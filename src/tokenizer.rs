use std::{str::from_utf8, sync::Arc};

use general_sam::{utils::tokenize as tokenize_rs, BoxBisectTable, TrieNodeID, TRIE_NIL_NODE_ID};
use ouroboros::self_referencing;
use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::{
    char_or_byte_type, for_both_and_wrap,
    sam::GeneralSAM,
    trie::Trie,
    utils::{get_char_or_byte_variant_name, ByteSide, CharSide, InconsistentCharOrByte},
};

pub(crate) type RustBoxBisectGreedyTokenizer<'s, T> =
    tokenize_rs::GreedyTokenizer<'s, BoxBisectTable<T>, TrieNodeID>;
pub(crate) type RustGreedyTokenizer<'s> = char_or_byte_type!(RustBoxBisectGreedyTokenizer; 's);

#[self_referencing]
pub struct SharedGreedyTokenizer {
    sam: GeneralSAM,
    #[borrows(sam)]
    #[covariant]
    inner: RustGreedyTokenizer<'this>,
}

impl SharedGreedyTokenizer {
    fn from_sam_and_trie(sam: &GeneralSAM, trie: &Trie) -> Result<Self, InconsistentCharOrByte> {
        Self::try_new(GeneralSAM(sam.0.clone()), |sam: &GeneralSAM| {
            for_both_and_wrap!(sam.0.as_ref(), trie.0.as_ref(); (sam, trie) => {
                RustBoxBisectGreedyTokenizer::build_from_trie(sam, trie.get_root_state())
            })
        })
    }
}

#[pyclass]
pub struct GreedyTokenizer(pub Arc<SharedGreedyTokenizer>);

#[pymethods]
impl GreedyTokenizer {
    pub fn get_sam(&self) -> GeneralSAM {
        GeneralSAM(self.0.borrow_sam().0.clone())
    }

    pub fn is_in_chars(&self) -> bool {
        self.0.borrow_sam().is_in_chars()
    }

    pub fn is_in_bytes(&self) -> bool {
        self.0.borrow_sam().is_in_bytes()
    }

    #[staticmethod]
    pub fn from_sam_and_trie(sam: &GeneralSAM, trie: &Trie) -> PyResult<Self> {
        SharedGreedyTokenizer::from_sam_and_trie(sam, trie)
            .map(|x| Self(Arc::new(x)))
            .map_err(|e| {
                PyTypeError::new_err(format!(
                    "{}, {} vs {}",
                    e,
                    get_char_or_byte_variant_name(sam.0.as_ref()),
                    get_char_or_byte_variant_name(&trie.0)
                ))
            })
    }

    #[pyo3(signature = (s, unk_token_id=None))]
    pub fn tokenize_str(
        &mut self,
        s: &str,
        unk_token_id: Option<TrieNodeID>,
    ) -> Vec<(TrieNodeID, usize)> {
        let unk_token_id = unk_token_id.unwrap_or(TRIE_NIL_NODE_ID);
        match self.0.borrow_inner() {
            CharSide(inner) => inner.tokenize(s.chars(), &unk_token_id),
            ByteSide(inner) => inner.tokenize(s.bytes(), &unk_token_id),
        }
    }

    #[pyo3(signature = (s, unk_token_id=None))]
    pub fn tokenize_bytes(
        &mut self,
        s: &[u8],
        unk_token_id: Option<TrieNodeID>,
    ) -> PyResult<Vec<(TrieNodeID, usize)>> {
        let unk_token_id = unk_token_id.unwrap_or(TRIE_NIL_NODE_ID);
        Ok(match self.0.borrow_inner() {
            CharSide(inner) => inner.tokenize(from_utf8(s)?.chars(), &unk_token_id),
            ByteSide(inner) => inner.tokenize(s.iter().copied(), &unk_token_id),
        })
    }
}
