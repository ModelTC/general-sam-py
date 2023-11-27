extern crate general_sam as general_sam_rs;

use std::{str::from_utf8, sync::Arc};

use general_sam_rs::{
    sam as sam_rs, trie as trie_rs, BTreeTransTable, BoxBisectTable, TransitionTable, TravelEvent,
    SAM_ROOT_NODE_ID,
};
use pyo3::exceptions::PyTypeError;
use pyo3::{prelude::*, types::PyDict};

use crate::trie::Trie;
use crate::utils::{
    char_or_byte_type, for_both, get_char_or_byte_variant_name, ByteSide, CharSide,
};
use crate::{for_both_and_wrap, for_both_with_side};

pub(crate) type RustBoxBisectGeneralSAM<T> = Arc<sam_rs::GeneralSAM<BoxBisectTable<T>>>;
pub(crate) type RustBoxBisectGeneralSAMState<T> =
    sam_rs::GeneralSAMState<BoxBisectTable<T>, RustBoxBisectGeneralSAM<T>>;
pub(crate) type RustGeneralSAM = char_or_byte_type!(RustBoxBisectGeneralSAM);
pub(crate) type RustGeneralSAMState = char_or_byte_type!(RustBoxBisectGeneralSAMState);

#[pyclass]
pub struct GeneralSAM(pub RustGeneralSAM);

#[pyclass]
#[derive(Clone)]
pub struct GeneralSAMState(pub RustGeneralSAMState);

#[pymethods]
impl GeneralSAMState {
    pub fn is_in_chars(&self) -> bool {
        self.0.is_left()
    }

    pub fn is_in_bytes(&self) -> bool {
        self.0.is_right()
    }

    pub fn get_node_id(&self) -> usize {
        for_both!(self.0.as_ref(), x => x.node_id)
    }

    pub fn is_nil(&self) -> bool {
        for_both!(self.0.as_ref(), x => x.is_nil())
    }

    pub fn is_root(&self) -> bool {
        for_both!(self.0.as_ref(), x => x.is_root())
    }

    pub fn is_accepting(&self) -> bool {
        for_both!(self.0.as_ref(), x => x.is_accepting())
    }

    pub fn get_trans(&self) -> PyObject {
        for_both!(self.0.as_ref(), state => {
            Python::with_gil(|py| {
                if let Some(node) = state.get_node() {
                    BTreeTransTable::from_kv_iter(node.get_trans().iter()).into_py(py)
                } else {
                    PyDict::new(py).into_py(py)
                }
            })
        })
    }

    pub fn get_suffix_parent_id(&self) -> usize {
        for_both!(self.0.as_ref(), x => {
            x.get_node()
                .map(|node| node.get_suffix_parent_id())
                .unwrap_or(sam_rs::SAM_NIL_NODE_ID)
        })
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn goto_suffix_parent(&mut self) {
        for_both!(self.0.as_mut(), state => {
            state.goto_suffix_parent();
        })
    }

    pub fn goto_char(&mut self, t: char) {
        let state = self.0.as_mut().left().unwrap();
        state.goto(&t);
    }

    pub fn goto_byte(&mut self, t: u8) {
        let state = self.0.as_mut().right().unwrap();
        state.goto(&t);
    }

    pub fn feed_chars(&mut self, s: &str) {
        match self.0.as_mut() {
            CharSide(state_chars) => {
                *state_chars = state_chars.clone().feed_chars(s);
            }
            ByteSide(state_bytes) => {
                *state_bytes = state_bytes.clone().feed_ref_iter(s.as_bytes().iter());
            }
        }
    }

    pub fn feed_bytes(&mut self, s: &[u8]) -> PyResult<()> {
        match self.0.as_mut() {
            CharSide(state_chars) => {
                *state_chars = state_chars.clone().feed_iter(from_utf8(s)?.chars());
            }
            ByteSide(state_bytes) => {
                *state_bytes = state_bytes.clone().feed_ref_iter(s.iter());
            }
        }
        Ok(())
    }

    #[pyo3(signature = (trie, in_stack_callback, out_stack_callback, trie_node_id=None))]
    pub fn dfs_along(
        &self,
        trie: &Trie,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
        trie_node_id: Option<usize>,
    ) -> PyResult<()> {
        let sam_state_and_trie = for_both_and_wrap!(self.0.as_ref(), &trie.0; (s, t) => (s, t))
            .map_err(|e| {
                PyTypeError::new_err(format!(
                    "{}, {} vs {}",
                    e,
                    get_char_or_byte_variant_name(&self.0),
                    get_char_or_byte_variant_name(&trie.0)
                ))
            })?;
        for_both_with_side!(sam_state_and_trie, side, (sam_state, trie) => {
            let tn = trie.get_state(trie_node_id.unwrap_or(trie_rs::TRIE_ROOT_NODE_ID));
            sam_state.dfs_along(tn, |event| {
                match event {
                    TravelEvent::PushRoot((st, tn)) => Python::with_gil(|py| {
                        in_stack_callback.call1(
                            py,
                            (GeneralSAMState(side(st.clone())), tn.node_id, None::<()>),
                        )
                    }),
                    TravelEvent::Push((st, tn), _, key) => Python::with_gil(|py| {
                        in_stack_callback
                            .call1(py, (GeneralSAMState(side(st.clone())), tn.node_id, key))
                    }),
                    TravelEvent::Pop((st, tn), _) => Python::with_gil(|py| {
                        out_stack_callback
                            .call1(py, (GeneralSAMState(side(st.clone())), tn.node_id))
                    }),
                }
                .map(|_| ())
            })
        })
    }

    #[pyo3(signature = (trie, in_stack_callback, out_stack_callback, trie_node_id=None))]
    pub fn bfs_along(
        &self,
        trie: &Trie,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
        trie_node_id: Option<usize>,
    ) -> Result<(), PyErr> {
        let sam_state_and_trie = for_both_and_wrap!(self.0.as_ref(), &trie.0; (s, t) => (s, t))
            .map_err(|e| {
                PyTypeError::new_err(format!(
                    "{}, {} vs {}",
                    e,
                    get_char_or_byte_variant_name(&self.0),
                    get_char_or_byte_variant_name(&trie.0)
                ))
            })?;
        for_both_with_side!(sam_state_and_trie, side, (sam_state, trie) => {
            let tn = trie.get_state(trie_node_id.unwrap_or(trie_rs::TRIE_ROOT_NODE_ID));
            sam_state.bfs_along(tn, |event| {
                match event {
                    TravelEvent::PushRoot((st, tn)) => Python::with_gil(|py| {
                        in_stack_callback.call1(
                            py,
                            (GeneralSAMState(side(st.clone())), tn.node_id, None::<()>),
                        )
                    }),
                    TravelEvent::Push((st, tn), _, key) => Python::with_gil(|py| {
                        in_stack_callback
                            .call1(py, (GeneralSAMState(side(st.clone())), tn.node_id, key))
                    }),
                    TravelEvent::Pop((st, tn), _) => Python::with_gil(|py| {
                        out_stack_callback
                            .call1(py, (GeneralSAMState(side(st.clone())), tn.node_id))
                    }),
                }
                .map(|_| ())
            })
        })
    }
}

#[pymethods]
impl GeneralSAM {
    #[staticmethod]
    pub fn from_chars(s: &str) -> Self {
        GeneralSAM(CharSide(Arc::new(
            sam_rs::GeneralSAM::<BTreeTransTable<_>>::from_chars(s.chars())
                .alter_trans_table_into(),
        )))
    }

    #[staticmethod]
    pub fn from_bytes(s: &[u8]) -> Self {
        GeneralSAM(ByteSide(Arc::new(
            sam_rs::GeneralSAM::<BTreeTransTable<_>>::from_bytes(s).alter_trans_table_into(),
        )))
    }

    #[staticmethod]
    pub fn from_trie(trie: &Trie) -> Self {
        for_both_with_side!(trie.0.as_ref(), side, trie => {
            GeneralSAM(side(Arc::new(
                sam_rs::GeneralSAM::<BTreeTransTable<_>>::from_trie(trie.get_root_state())
                    .alter_trans_table_into(),
            )))
        })
    }

    pub fn is_in_chars(&self) -> bool {
        self.0.is_left()
    }

    pub fn is_in_bytes(&self) -> bool {
        self.0.is_right()
    }

    pub fn num_of_nodes(&self) -> usize {
        for_both!(self.0.as_ref(), x => x.num_of_nodes())
    }

    pub fn get_root_state(&self) -> GeneralSAMState {
        GeneralSAMState(for_both_with_side!(
            self.0.as_ref(),
            side,
            x => side(RustBoxBisectGeneralSAMState {
                sam: x.clone(),
                node_id: SAM_ROOT_NODE_ID,
            })
        ))
    }

    pub fn get_state(&self, node_id: usize) -> GeneralSAMState {
        GeneralSAMState(for_both_with_side!(
            self.0.as_ref(),
            side,
            x => side(RustBoxBisectGeneralSAMState {
                sam: x.clone(),
                node_id,
            })
        ))
    }

    pub fn get_topo_and_suf_len_sorted_states(&self) -> Vec<GeneralSAMState> {
        for_both!(self.0.as_ref(), x => {
            x.get_topo_and_suf_len_sorted_node_ids()
                .iter()
                .map(|node_id| self.get_state(*node_id))
                .collect()
        })
    }
}
