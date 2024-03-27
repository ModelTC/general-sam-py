mod sam;
mod tokenizer;
mod trie;
mod utils;

use pyo3::prelude::*;

#[pymodule]
fn general_sam(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<trie::TrieNode>()?;
    m.add_class::<trie::Trie>()?;
    m.add_class::<sam::GeneralSamState>()?;
    m.add_class::<sam::GeneralSam>()?;
    m.add_class::<tokenizer::GreedyTokenizer>()?;
    Ok(())
}
