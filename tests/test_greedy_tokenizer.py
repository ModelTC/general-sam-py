from general_sam import GeneralSAM, GreedyTokenizer, build_trie_from_chars


def test_chinese_chars_tokenize():
    vocab = ['歌曲', '聆听歌曲', '播放歌曲', '歌词', '查看歌词', '听歌', '曲折']
    trie, token_to_trie_node = build_trie_from_chars(vocab)

    trie_node_to_token = [-1] * trie.num_of_nodes()
    for i, j in enumerate(token_to_trie_node):
        trie_node_to_token[j] = i

    sam = GeneralSAM.from_trie(trie)
    tokenizer = GreedyTokenizer.from_sam_and_trie(sam, trie)

    def tokenize(s: str):
        return [(trie_node_to_token[i], j) for i, j in tokenizer.tokenize_str(s)]

    assert tokenize('歌曲折') == [(0, 2), (-1, 1)]
    assert tokenize('听歌曲') == [(5, 2), (-1, 1)]
    assert tokenize('听歌曲折') == [(5, 2), (6, 2)]
    assert tokenize('聆听歌曲折') == [(1, 4), (-1, 1)]
    assert tokenize('查看歌词歌曲') == [(4, 4), (0, 2)]
    assert tokenize('一起播放歌曲并共享歌词') == [(-1, 2), (2, 4), (-1, 3), (3, 2)]
