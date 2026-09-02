use e2k::C2k;

/// ASCII 英単語を英語らしいカタカナ読みに置換する。
pub(crate) fn katakanize_ascii_words(text: &str, converter: &C2k) -> String {
    let mut out = String::with_capacity(text.len());
    let mut word = String::new();

    let flush_word = |out: &mut String, word: &mut String| {
        if word.is_empty() {
            return;
        }

        let lower = word.to_ascii_lowercase();
        let katakana = converter.infer(&lower);
        if katakana.is_empty() {
            out.push_str(word);
        } else {
            out.push_str(&katakana);
        }
        word.clear();
    };

    for ch in text.chars() {
        if ch.is_ascii_alphabetic() {
            word.push(ch);
        } else {
            flush_word(&mut out, &mut word);
            out.push(ch);
        }
    }
    flush_word(&mut out, &mut word);

    out
}

#[cfg(test)]
mod tests {
    use super::katakanize_ascii_words;

    #[test]
    fn replaces_ascii_words_and_preserves_other_text() {
        let converter = e2k::C2k::new(64);

        assert_eq!(
            katakanize_ascii_words("I love Rust! 日本語", &converter),
            "イ ラブ ラスト! 日本語"
        );
        assert_eq!(katakanize_ascii_words("123", &converter), "123");
    }
}
