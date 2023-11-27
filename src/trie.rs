extern crate general_sam as general_sam_rs;

use std::{convert::Infallible, str::from_utf8};

use general_sam_rs::{trie as trie_rs, BTreeTransTable, TravelEvent, TrieNodeAlike};
use pyo3::prelude::*;

use crate::{
    for_both_with_side,
    utils::{char_or_byte_type, for_both, ByteSide, CharSide},
};

pub(crate) type RustBTreeTrie<T> = trie_rs::Trie<BTreeTransTable<T>>;
pub(crate) type RustBTreeTrieNode<T> = trie_rs::TrieNode<BTreeTransTable<T>>;
pub(crate) type RustTrie = char_or_byte_type!(RustBTreeTrie);
pub(crate) type RustTrieNode = char_or_byte_type!(RustBTreeTrieNode);

#[pyclass]
pub struct Trie(pub RustTrie);

#[pyclass]
pub struct TrieNode(pub usize, pub RustTrieNode);

#[pymethods]
impl TrieNode {
    pub fn is_in_chars(&self) -> bool {
        self.1.is_left()
    }

    pub fn is_in_bytes(&self) -> bool {
        self.1.is_right()
    }

    pub fn get_node_id(&self) -> usize {
        self.0
    }

    pub fn is_accepting(&self) -> bool {
        for_both!(self.1.as_ref(), x => x.accept)
    }

    pub fn get_trans(&self) -> PyObject {
        for_both!(self.1.as_ref(), x => {
            Python::with_gil(|py| x.get_trans().clone().into_py(py))
        })
    }

    pub fn get_parent(&self) -> usize {
        for_both!(self.1.as_ref(), x => x.get_parent())
    }
}

#[pymethods]
impl Trie {
    #[staticmethod]
    pub fn in_chars() -> Self {
        Trie(CharSide(Default::default()))
    }

    #[staticmethod]
    pub fn in_bytes() -> Self {
        Trie(ByteSide(Default::default()))
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

    pub fn insert_chars(&mut self, s: &str) -> usize {
        match self.0.as_mut() {
            CharSide(trie_chars) => trie_chars.insert_iter(s.chars()),
            ByteSide(trie_bytes) => trie_bytes.insert_ref_iter(s.as_bytes().iter()),
        }
    }

    pub fn insert_bytes(&mut self, b: &[u8]) -> PyResult<usize> {
        Ok(match self.0.as_mut() {
            CharSide(trie_chars) => trie_chars.insert_iter(from_utf8(b)?.chars()),
            ByteSide(trie_bytes) => trie_bytes.insert_ref_iter(b.iter()),
        })
    }

    pub fn get_bfs_order(&self) -> Vec<usize> {
        for_both!(self.0.as_ref(), trie => {
            let state = trie.get_root_state();
            let mut res = Vec::new();
            state
                .bfs_travel(|event| -> Result<(), Infallible> {
                    if let TravelEvent::Push(s, _, _) = event {
                        res.push(s.node_id);
                    }
                    Ok(())
                })
                .unwrap();
            res
        })
    }
    pub fn get_root(&self) -> TrieNode {
        self.get_node(trie_rs::TRIE_ROOT_NODE_ID).unwrap()
    }

    pub fn get_node(&self, node_id: usize) -> Option<TrieNode> {
        for_both_with_side!(self.0.as_ref(), side, trie => {
            trie.get_node(node_id)
                .map(|node| TrieNode(node_id, side(node.clone())))
        })
    }

    #[pyo3(signature = (in_stack_callback, out_stack_callback, root_node_id=None))]
    pub fn dfs_travel(
        &self,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
        root_node_id: Option<usize>,
    ) -> Result<(), PyErr> {
        for_both!(self.0.as_ref(), trie => {
            let root_state = trie.get_state(root_node_id.unwrap_or(trie_rs::TRIE_ROOT_NODE_ID));
            if root_state.is_nil() {
                return Ok(());
            }
            root_state.dfs_travel(|event| {
                match event {
                    TravelEvent::PushRoot(tn) => {
                        Python::with_gil(|py| in_stack_callback.call1(py, (tn.node_id, None::<()>)))
                    }
                    TravelEvent::Push(tn, _, key) => {
                        Python::with_gil(|py| in_stack_callback.call1(py, (tn.node_id, key)))
                    }
                    TravelEvent::Pop(tn, _) => {
                        Python::with_gil(|py| out_stack_callback.call1(py, (tn.node_id,)))
                    }
                }
                .map(|_| ())
            })
        })
    }

    #[pyo3(signature = (in_stack_callback, out_stack_callback, root_node_id=None))]
    pub fn bfs_travel(
        &self,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
        root_node_id: Option<usize>,
    ) -> Result<(), PyErr> {
        for_both!(self.0.as_ref(), trie => {
            let root_state = trie.get_state(root_node_id.unwrap_or(trie_rs::TRIE_ROOT_NODE_ID));
            if root_state.is_nil() {
                return Ok(());
            }
            root_state.bfs_travel(|event| {
                match event {
                    TravelEvent::PushRoot(tn) => {
                        Python::with_gil(|py| in_stack_callback.call1(py, (tn.node_id, None::<()>)))
                    }
                    TravelEvent::Push(tn, _, key) => {
                        Python::with_gil(|py| in_stack_callback.call1(py, (tn.node_id, key)))
                    }
                    TravelEvent::Pop(tn, _) => {
                        Python::with_gil(|py| out_stack_callback.call1(py, (tn.node_id,)))
                    }
                }
                .map(|_| ())
            })
        })
    }
}
