extern crate general_sam as general_sam_rs;

use crate::trie::Trie;

use std::{str::from_utf8, sync::Arc};

use either::{
    for_both, Either as CharOrByte,
    {Either::Left as CharSide, Either::Right as ByteSide},
};
use pyo3::{prelude::*, types::PyDict};

use general_sam_rs::{sam as sam_rs, trie as trie_rs, trie_alike::TravelEvent};

type RustGeneralSAM = CharOrByte<sam_rs::GeneralSAM<char>, sam_rs::GeneralSAM<u8>>;
type RustGeneralSAMState<'s> =
    CharOrByte<sam_rs::GeneralSAMState<'s, char>, sam_rs::GeneralSAMState<'s, u8>>;

#[pyclass]
pub struct GeneralSAM(pub Arc<RustGeneralSAM>);

#[pyclass]
#[derive(Clone)]
pub struct GeneralSAMState(pub Arc<RustGeneralSAM>, pub usize);

impl GeneralSAMState {
    pub fn get_state(&self) -> RustGeneralSAMState<'_> {
        self.0
            .as_ref()
            .as_ref()
            .map_either(|x| x.get_state(self.1), |x| x.get_state(self.1))
    }
}

#[pymethods]
impl GeneralSAMState {
    pub fn is_in_chars(&self) -> bool {
        self.0.is_left()
    }

    pub fn is_in_bytes(&self) -> bool {
        self.0.is_right()
    }

    pub fn get_node_id(&self) -> usize {
        self.1
    }

    pub fn is_nil(&self) -> bool {
        for_both!(self.get_state().as_ref(), x => x.is_nil())
    }

    pub fn is_root(&self) -> bool {
        for_both!(self.get_state().as_ref(), x => x.is_root())
    }

    pub fn is_accepting(&self) -> bool {
        for_both!(self.get_state().as_ref(), x => x.is_accepting())
    }

    pub fn get_trans(&self) -> PyObject {
        Python::with_gil(|py| {
            for_both!(self.get_state().as_ref(), state => {
                if let Some(node) = state.get_node() {
                    node.get_trans().clone().into_py(py)
                } else {
                    PyDict::new(py).into_py(py)
                }
            })
        })
    }

    pub fn get_suffix_parent_id(&self) -> usize {
        for_both!(self.get_state().as_ref() , x => {
            x.get_node()
                .map(|node| node.get_suffix_parent_id())
                .unwrap_or(sam_rs::SAM_NIL_NODE_ID)
        })
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn goto_suffix_parent(&mut self) {
        for_both!(self.get_state(), mut state => {
            state.goto_suffix_parent();
            self.1 = state.node_id;
        })
    }

    pub fn goto_char(&mut self, t: char) {
        let mut state = self.get_state().left().unwrap();
        state.goto(&t);
        self.1 = state.node_id;
    }

    pub fn goto_byte(&mut self, t: u8) {
        let mut state = self.get_state().right().unwrap();
        state.goto(&t);
        self.1 = state.node_id;
    }

    pub fn feed_chars(&mut self, s: &str) {
        match self.get_state() {
            CharSide(state_chars) => {
                let state_chars = state_chars.feed_chars(s);
                self.1 = state_chars.node_id;
            }
            ByteSide(state_bytes) => {
                let state_bytes = state_bytes.feed_ref_iter(s.as_bytes().iter());
                self.1 = state_bytes.node_id;
            }
        }
    }

    pub fn feed_bytes(&mut self, s: &[u8]) {
        match self.get_state() {
            CharSide(state_chars) => {
                let state_chars = state_chars.feed_iter(from_utf8(s).unwrap().chars());
                self.1 = state_chars.node_id;
            }
            ByteSide(state_bytes) => {
                let state_bytes = state_bytes.feed_ref_iter(s.iter());
                self.1 = state_bytes.node_id;
            }
        }
    }

    #[pyo3(signature = (trie, in_stack_callback, out_stack_callback, trie_node_id=None))]
    pub fn dfs_along(
        &self,
        trie: &Trie,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
        trie_node_id: Option<usize>,
    ) -> Result<(), PyErr> {
        assert!(trie.is_in_chars() == self.is_in_chars());
        let sam_state_and_trie = self.get_state().map_either(
            |x| (x, trie.0.as_ref().left().unwrap()),
            |x| (x, trie.0.as_ref().right().unwrap()),
        );
        for_both!(sam_state_and_trie, (sam_state, trie) => {
            let tn = trie.get_state(trie_node_id.unwrap_or(trie_rs::TRIE_ROOT_NODE_ID));
            sam_state.dfs_along(tn, |event| match event {
                TravelEvent::Push((st, tn), key_opt) => Python::with_gil(|py| {
                    in_stack_callback
                        .call1(
                            py,
                            (
                                GeneralSAMState(self.0.clone(), st.node_id),
                                tn.node_id,
                                key_opt,
                            ),
                        )
                        .map(|_| ())
                })
                .map(|_| ()),
                TravelEvent::Pop((st, tn)) => Python::with_gil(|py| {
                    out_stack_callback
                        .call1(
                            py,
                            (GeneralSAMState(self.0.clone(), st.node_id), tn.node_id),
                        )
                        .map(|_| ())
                }),
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
        assert!(trie.is_in_chars() == self.is_in_chars());
        let sam_state_and_trie = self.get_state().map_either(
            |x| (x, trie.0.as_ref().left().unwrap()),
            |x| (x, trie.0.as_ref().right().unwrap()),
        );
        for_both!(sam_state_and_trie, (sam_state, trie) => {
            let tn = trie.get_state(trie_node_id.unwrap_or(trie_rs::TRIE_ROOT_NODE_ID));
            sam_state.bfs_along(tn, |event| match event {
                TravelEvent::Push((st, tn), key_opt) => Python::with_gil(|py| {
                    in_stack_callback
                        .call1(
                            py,
                            (
                                GeneralSAMState(self.0.clone(), st.node_id),
                                tn.node_id,
                                key_opt,
                            ),
                        )
                        .map(|_| ())
                })
                .map(|_| ()),
                TravelEvent::Pop((st, tn)) => Python::with_gil(|py| {
                    out_stack_callback
                        .call1(
                            py,
                            (GeneralSAMState(self.0.clone(), st.node_id), tn.node_id),
                        )
                        .map(|_| ())
                }),
            })
        })
    }
}

#[pymethods]
impl GeneralSAM {
    #[staticmethod]
    pub fn construct_from_chars(s: &str) -> Self {
        GeneralSAM(Arc::new(CharSide(
            sam_rs::GeneralSAM::construct_from_chars(s.chars()),
        )))
    }

    #[staticmethod]
    pub fn construct_from_bytes(s: &[u8]) -> Self {
        GeneralSAM(Arc::new(ByteSide(
            sam_rs::GeneralSAM::construct_from_bytes(s),
        )))
    }

    #[staticmethod]
    pub fn construct_from_trie(trie: &Trie) -> Self {
        match trie.0.as_ref() {
            CharSide(trie_chars) => GeneralSAM(Arc::new(CharSide(
                sam_rs::GeneralSAM::construct_from_trie(trie_chars.get_root_state()),
            ))),
            ByteSide(trie_bytes) => GeneralSAM(Arc::new(ByteSide(
                sam_rs::GeneralSAM::construct_from_trie(trie_bytes.get_root_state()),
            ))),
        }
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
        GeneralSAMState(self.0.clone(), sam_rs::SAM_ROOT_NODE_ID)
    }

    pub fn get_state(&self, node_id: usize) -> GeneralSAMState {
        GeneralSAMState(self.0.clone(), node_id)
    }

    pub fn get_topo_order(&self) -> Vec<GeneralSAMState> {
        for_both!(self.0.as_ref(), x => {
            x.get_topo_sorted_node_ids()
                .iter()
                .map(|node_id| self.get_state(*node_id))
                .collect()
        })
    }
}
