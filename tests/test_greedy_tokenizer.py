from general_sam import (
    GeneralSAM,
    GreedyTokenizer,
    build_trie_from_bytes,
    build_trie_from_chars,
)


def test_english_chars_tokenize():
    vocab = ["a", "ab", "b", "bc", "c", "d", "e", "f", "cd", "abcde"]
    trie, token_to_trie_node = build_trie_from_chars(vocab)

    trie_node_to_token = [-1] * trie.num_of_nodes()
    for i, j in enumerate(token_to_trie_node):
        trie_node_to_token[j] = i

    sam = GeneralSAM.from_trie(trie)
    tokenizer = GreedyTokenizer.from_sam_and_trie(sam, trie)
    assert tokenizer.is_in_chars()

    def tokenize(s: str):
        return [(trie_node_to_token[i], j) for i, j in tokenizer.tokenize_str(s)]

    assert tokenize("abcde") == [(9, 5)]
    assert tokenize("abcdf") == [(1, 2), (8, 2), (7, 1)]
    assert tokenize("abca") == [(1, 2), (4, 1), (0, 1)]


def test_chinese_chars_tokenize():
    vocab = ["歌曲", "聆听歌曲", "播放歌曲", "歌词", "查看歌词", "听歌", "曲折"]
    trie, token_to_trie_node = build_trie_from_chars(vocab)

    trie_node_to_token = [-1] * trie.num_of_nodes()
    for i, j in enumerate(token_to_trie_node):
        trie_node_to_token[j] = i

    sam = GeneralSAM.from_trie(trie)
    tokenizer = GreedyTokenizer.from_sam_and_trie(sam, trie)
    assert tokenizer.is_in_chars()

    def tokenize(s: str):
        return [(trie_node_to_token[i], j) for i, j in tokenizer.tokenize_str(s)]

    assert tokenize("歌曲折") == [(0, 2), (-1, 1)]
    assert tokenize("听歌曲") == [(5, 2), (-1, 1)]
    assert tokenize("听歌曲折") == [(5, 2), (6, 2)]
    assert tokenize("聆听歌曲折") == [(1, 4), (-1, 1)]
    assert tokenize("查看歌词歌曲") == [(4, 4), (0, 2)]
    assert tokenize("一起播放歌曲并共享歌词") == [(-1, 2), (2, 4), (-1, 3), (3, 2)]


def test_chinese_bytes_tokenize():
    vocab = ["歌曲", "聆听歌曲", "播放歌曲", "歌词", "查看歌词", "听歌", "曲折"]
    vocab = [i.encode() for i in vocab]
    trie, token_to_trie_node = build_trie_from_bytes(vocab)

    trie_node_to_token = [-1] * trie.num_of_nodes()
    for i, j in enumerate(token_to_trie_node):
        trie_node_to_token[j] = i

    sam = GeneralSAM.from_trie(trie)
    tokenizer = GreedyTokenizer.from_sam_and_trie(sam, trie)
    assert tokenizer.is_in_bytes()

    def tokenize_str(s: str):
        return [trie_node_to_token[i] for i, _ in tokenizer.tokenize_str(s)]

    def tokenize_bytes(s: str):
        return [trie_node_to_token[i] for i, _ in tokenizer.tokenize_bytes(s.encode())]

    def tokenize(s: str):
        a = tokenize_str(s)
        b = tokenize_bytes(s)
        assert a == b
        return a

    assert tokenize("歌曲折") == [0, -1]
    assert tokenize("听歌曲") == [5, -1]
    assert tokenize("听歌曲折") == [5, 6]
    assert tokenize("聆听歌曲折") == [1, -1]
    assert tokenize("查看歌词歌曲") == [4, 0]
    assert tokenize("一起播放歌曲并共享歌词") == [-1, 2, -1, 3]
