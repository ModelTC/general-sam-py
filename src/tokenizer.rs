use std::{str::from_utf8, sync::Arc};

use general_sam::{utils::tokenize as tokenize_rs, BoxBisectTable, TrieNodeID, TRIE_NIL_NODE_ID};
use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::{
    char_or_byte_type, for_both_and_wrap, for_both_with_side,
    sam::{GeneralSAM, RustBoxBisectGeneralSAM},
    trie::Trie,
    utils::{get_char_or_byte_variant_name, ByteSide, CharSide},
};

pub(crate) type RustBoxBisectGreedyTokenizer<T> =
    tokenize_rs::GreedyTokenizer<BoxBisectTable<T>, TrieNodeID, RustBoxBisectGeneralSAM<T>>;
pub(crate) type RustGreedyTokenizer = char_or_byte_type!(RustBoxBisectGreedyTokenizer);

#[pyclass]
pub struct GreedyTokenizer(pub Arc<RustGreedyTokenizer>);

#[pymethods]
impl GreedyTokenizer {
    #[staticmethod]
    pub fn from_sam_and_trie(sam: &GeneralSAM, trie: &Trie) -> PyResult<Self> {
        Ok(Self(Arc::new(
            for_both_and_wrap!(sam.0.as_ref(), trie.0.as_ref(); (sam, trie) => {
                RustBoxBisectGreedyTokenizer::build_from_trie(sam.clone(), trie.get_root_state())
            })
            .map_err(|e| {
                PyTypeError::new_err(format!(
                    "{}, {} vs {}",
                    e,
                    get_char_or_byte_variant_name(&sam.0),
                    get_char_or_byte_variant_name(&trie.0)
                ))
            })?,
        )))
    }

    pub fn get_sam(&self) -> GeneralSAM {
        for_both_with_side!(self.0.as_ref(), side, x => {
            GeneralSAM(side(x.get_sam().clone()))
        })
    }

    pub fn is_in_chars(&self) -> bool {
        self.0.is_left()
    }

    pub fn is_in_bytes(&self) -> bool {
        self.0.is_right()
    }

    #[pyo3(signature = (s, unk_token_id=TRIE_NIL_NODE_ID))]
    pub fn tokenize_str(&mut self, s: &str, unk_token_id: TrieNodeID) -> Vec<(TrieNodeID, usize)> {
        match self.0.as_ref() {
            CharSide(inner) => inner.tokenize(s.chars(), &unk_token_id),
            ByteSide(inner) => inner.tokenize(s.bytes(), &unk_token_id),
        }
    }

    #[pyo3(signature = (s, unk_token_id=TRIE_NIL_NODE_ID))]
    pub fn tokenize_bytes(
        &mut self,
        s: &[u8],
        unk_token_id: TrieNodeID,
    ) -> PyResult<Vec<(TrieNodeID, usize)>> {
        Ok(match self.0.as_ref() {
            CharSide(inner) => inner.tokenize(from_utf8(s)?.chars(), &unk_token_id),
            ByteSide(inner) => inner.tokenize(s.iter().copied(), &unk_token_id),
        })
    }
}
