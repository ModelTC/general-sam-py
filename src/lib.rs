pub mod sam;
pub mod tokenizer;
pub mod trie;
pub mod utils;

use pyo3::prelude::*;

#[pymodule]
fn general_sam(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<trie::TrieNode>()?;
    m.add_class::<trie::Trie>()?;
    m.add_class::<sam::GeneralSAMState>()?;
    m.add_class::<sam::GeneralSAM>()?;
    m.add_class::<tokenizer::GreedyTokenizer>()?;
    Ok(())
}
