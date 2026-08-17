/// AquesTalk 音声記号列仕様 1.8 で定義されているかな読み記号。
///
/// AqKanji2Koe が出力するかな音声記号列は、旧 AquesTalk1 DLL でも
/// 未定義読み (102/105) にならないよう、この集合だけを使用する。
pub(crate) const AQUESTALK_KANA_1_8_READINGS: &[&str] = &[
    "あ", "か", "さ", "た", "な", "は", "ま", "や", "ら", "わ", "ん", "が", "ざ", "だ", "ば", "ぱ",
    "い", "き", "し", "ち", "に", "ひ", "み", "り", "っ", "ぎ", "じ", "び", "ぴ",
    "う", "く", "す", "つ", "ぬ", "ふ", "む", "ゆ", "る", "ー", "ぐ", "ず", "ぶ", "ぷ",
    "え", "け", "せ", "て", "ね", "へ", "め", "いぇ", "れ", "げ", "ぜ", "で", "べ", "ぺ",
    "お", "こ", "そ", "と", "の", "ほ", "も", "よ", "ろ", "を", "ご", "ぞ", "ど", "ぼ", "ぽ",
    "きゃ", "しゃ", "ちゃ", "にゃ", "ひゃ", "みゃ", "りゃ", "ぎゃ", "じゃ", "つぁ", "ふぁ", "びゃ", "ぴゃ", "すぃ", "とぅ",
    "きゅ", "しゅ", "ちゅ", "にゅ", "ひゅ", "みゅ", "りゅ", "ぎゅ", "じゅ", "うぃ", "つぃ", "ふぃ", "びゅ", "ぴゅ", "てぃ", "どぅ",
    "きぇ", "しぇ", "ちぇ", "にぇ", "ひぇ", "みぇ", "りぇ", "ぎぇ", "じぇ", "うぇ", "つぇ", "ふぇ", "びぇ", "ぴぇ", "ずぃ", "でゅ",
    "きょ", "しょ", "ちょ", "にょ", "ひょ", "みょ", "りょ", "ぎょ", "じょ", "うぉ", "つぉ", "ふぉ", "びょ", "ぴょ", "でぃ", "てゅ",
];

#[inline]
pub(crate) fn is_aquestalk_kana_1_8_reading(reading: &str) -> bool {
    AQUESTALK_KANA_1_8_READINGS.contains(&reading)
}

/// カタカナ 1 文字をひらがなに変換する。
///
/// ー (U+30FC) および変換対象外の文字はそのまま返す。
#[inline]
pub fn katakana_char_to_hiragana(c: char) -> char {
    let code = c as u32;
    // カタカナ U+30A1〜U+30F6 はひらがな U+3041〜U+3096 に対応 (オフセット -0x60)
    if (0x30A1..=0x30F6).contains(&code) {
        char::from_u32(code - 0x60).unwrap_or(c)
    } else {
        c
    }
}

fn fallback_char_to_aquestalk(c: char) -> &'static str {
    match c {
        // AquesTalk 1.8 で未定義の歴史的かな。
        'ぢ' => "じ",
        'づ' => "ず",
        'ゐ' => "い",
        'ゑ' => "え",
        // 「ヴ」系は 1.8 に無いため、定義済み読みの連結へ落とす。
        'ゔ' => "ぶ",
        // 単独の小書きかなは読み記号ではない。対応する大書きかなへ寄せる。
        'ぁ' => "あ",
        'ぃ' => "い",
        'ぅ' => "う",
        'ぇ' => "え",
        'ぉ' => "お",
        'ゃ' => "や",
        'ゅ' => "ゆ",
        'ょ' => "よ",
        'ゎ' => "わ",
        'ゕ' => "か",
        'ゖ' => "け",
        // jpreprocess の既知モーラ以外が将来追加されても、未定義記号を
        // AquesTalk DLL に渡さない。通常この分岐へは到達しない。
        _ => "ん",
    }
}

/// カタカナのモーラ文字列を AquesTalk 1.8 互換のかな読みに変換する。
///
/// 直接定義されているモーラはそのままひらがな化する。OpenJTalk が
/// `ヂ` / `ヅ` / `ヴァ` / 単独小書きかな等の未定義モーラを返した場合は、
/// AquesTalk 1.8 で定義済みの読み記号の連結へ安全に正規化する。
pub fn mora_katakana_to_hiragana(mora: &str) -> String {
    let hiragana: String = mora.chars().map(katakana_char_to_hiragana).collect();
    if is_aquestalk_kana_1_8_reading(&hiragana) {
        return hiragana;
    }

    let mut out = String::new();
    for c in hiragana.chars() {
        let single = c.to_string();
        if is_aquestalk_kana_1_8_reading(&single) {
            out.push(c);
        } else {
            out.push_str(fallback_char_to_aquestalk(c));
        }
    }

    if out.is_empty() {
        // jpreprocess のモーラは空にならないが、将来の変更に対する最後の安全網。
        "ん".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{is_aquestalk_kana_1_8_reading, mora_katakana_to_hiragana};

    #[test]
    fn preserves_defined_aquestalk_1_8_moras() {
        for (source, expected) in [
            ("カ", "か"),
            ("キャ", "きゃ"),
            ("イェ", "いぇ"),
            ("ツァ", "つぁ"),
            ("スィ", "すぃ"),
            ("ディ", "でぃ"),
            ("ー", "ー"),
        ] {
            assert_eq!(mora_katakana_to_hiragana(source), expected);
        }
    }

    #[test]
    fn normalizes_every_known_jpreprocess_mora_missing_from_aquestalk_1_8() {
        let cases = [
            ("ヴョ", "ぶよ"),
            ("ヴュ", "ぶゆ"),
            ("ヴャ", "ぶや"),
            ("ヴォ", "ぶお"),
            ("ヴェ", "ぶえ"),
            ("ヴィ", "ぶい"),
            ("ヴァ", "ぶあ"),
            ("ヴ", "ぶ"),
            ("ヱ", "え"),
            ("ヰ", "い"),
            ("ョ", "よ"),
            ("ュ", "ゆ"),
            ("ャ", "や"),
            ("デョ", "でよ"),
            ("デャ", "でや"),
            ("テョ", "てよ"),
            ("テャ", "てや"),
            ("ヅ", "ず"),
            ("ヂ", "じ"),
            ("ォ", "お"),
            ("ェ", "え"),
            ("ゥ", "う"),
            ("ィ", "い"),
            ("ァ", "あ"),
            ("グヮ", "ぐわ"),
            ("クヮ", "くわ"),
            ("ヮ", "わ"),
            ("ヶ", "け"),
        ];

        for (source, expected) in cases {
            let actual = mora_katakana_to_hiragana(source);
            assert_eq!(actual, expected, "source mora: {source}");
            // Fallbacks are intentionally sequences of single defined readings.
            for c in actual.chars() {
                assert!(
                    is_aquestalk_kana_1_8_reading(&c.to_string()),
                    "{source} produced undefined reading {c}"
                );
            }
        }
    }
}
