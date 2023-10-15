extern crate general_sam as general_sam_rs;

use std::{convert::Infallible, str::from_utf8};

use either::{
    for_both, Either as CharOrByte,
    {Either::Left as CharSide, Either::Right as ByteSide},
};
use pyo3::prelude::*;

use general_sam_rs::{
    trie as trie_rs,
    trie_alike::{TravelEvent, TrieNodeAlike},
};

type RustTrie = CharOrByte<trie_rs::Trie<char>, trie_rs::Trie<u8>>;
type RustTrieNode = CharOrByte<trie_rs::TrieNode<char>, trie_rs::TrieNode<u8>>;

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
        Python::with_gil(|py| {
            for_both!(self.1.as_ref(), x => {
                x.get_trans().clone().into_py(py)
            })
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

    pub fn insert_bytes(&mut self, b: &[u8]) -> usize {
        match self.0.as_mut() {
            CharSide(trie_chars) => trie_chars.insert_iter(from_utf8(b).unwrap().chars()),
            ByteSide(trie_bytes) => trie_bytes.insert_ref_iter(b.iter()),
        }
    }

    pub fn get_bfs_order(&self) -> Vec<usize> {
        for_both!(self.0.as_ref(), trie => {
            let state = trie.get_root_state();
            let mut res = Vec::new();
            state
                .bfs_travel(|event| -> Result<(), Infallible> {
                    if let TravelEvent::Push(s, _) = event {
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
        match self.0.as_ref() {
            CharSide(trie) => trie
                .get_node(node_id)
                .map(|node| TrieNode(node_id, CharSide(node.clone()))),
            ByteSide(trie) => trie
                .get_node(node_id)
                .map(|node| TrieNode(node_id, ByteSide(node.clone()))),
        }
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
            root_state.dfs_travel(|event| match event {
                TravelEvent::Push(tn, key_opt) => Python::with_gil(|py| {
                    in_stack_callback.call1(py, (tn.node_id, key_opt))
                })
                .map(|_| ()),
                TravelEvent::Pop(tn) => {
                    Python::with_gil(|py| out_stack_callback.call1(py, (tn.node_id,))).map(|_| ())
                }
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
            root_state.bfs_travel(|event| match event {
                TravelEvent::Push(tn, key_opt) => Python::with_gil(|py| {
                    in_stack_callback.call1(py, (tn.node_id, key_opt))
                })
                .map(|_| ()),
                TravelEvent::Pop(tn) => {
                    Python::with_gil(|py| out_stack_callback.call1(py, (tn.node_id,))).map(|_| ())
                }
            })
        })
    }
}
