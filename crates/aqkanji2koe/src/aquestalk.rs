use crate::mora::AQUESTALK_KANA_1_8_READINGS;

const MAX_READINGS_PER_PHRASE: usize = 255;

// AquesTalk 1.8 の「無声化読み記号」。左が実際の表記、右が通常読み。
const DEVOICED_READINGS: &[(&str, &str)] = &[
    ("_フィ", "ふぃ"),
    ("_シュ", "しゅ"),
    ("_スィ", "すぃ"),
    ("_ティ", "てぃ"),
    ("_トゥ", "とぅ"),
    ("_チュ", "ちゅ"),
    ("_ツィ", "つぃ"),
    ("_キ", "き"),
    ("_ク", "く"),
    ("_ヒ", "ひ"),
    ("_フ", "ふ"),
    ("_シ", "し"),
    ("_ス", "す"),
    ("_ピ", "ぴ"),
    ("_プ", "ぷ"),
    ("_チ", "ち"),
    ("_ツ", "つ"),
];

#[derive(Clone, Debug)]
struct KanaToken {
    normal: String,
    rendered: String,
    devoiced: bool,
    accent_after: bool,
}

impl KanaToken {
    fn normal(reading: &str) -> Self {
        Self {
            normal: reading.to_string(),
            rendered: reading.to_string(),
            devoiced: false,
            accent_after: false,
        }
    }

    fn devoiced(rendered: &str, normal: &str) -> Self {
        Self {
            normal: normal.to_string(),
            rendered: rendered.to_string(),
            devoiced: true,
            accent_after: false,
        }
    }

    fn disable_devoicing(&mut self) {
        self.devoiced = false;
        self.rendered = self.normal.clone();
    }
}

fn take_delimiter(input: &str) -> Option<(&'static str, usize)> {
    for delimiter in ["。", "？", "、", ",", ";", "/", "+"] {
        if input.starts_with(delimiter) {
            return Some((delimiter, delimiter.len()));
        }
    }
    None
}

fn take_devoiced(input: &str) -> Option<(&'static str, &'static str, usize)> {
    DEVOICED_READINGS
        .iter()
        .filter(|(rendered, _)| input.starts_with(*rendered))
        .max_by_key(|(rendered, _)| rendered.len())
        .map(|(rendered, normal)| (*rendered, *normal, rendered.len()))
}

fn take_reading(input: &str) -> Option<(&'static str, usize)> {
    AQUESTALK_KANA_1_8_READINGS
        .iter()
        .copied()
        .filter(|reading| input.starts_with(*reading))
        .max_by_key(|reading| reading.len())
        .map(|reading| (reading, reading.len()))
}

fn forbidden_after_devoiced(reading: &str) -> bool {
    matches!(
        reading,
        "ー"
            | "あ"
            | "い"
            | "う"
            | "え"
            | "お"
            | "ん"
            | "や"
            | "ゆ"
            | "いぇ"
            | "よ"
            | "わ"
            | "を"
            | "だ"
            | "で"
            | "ど"
            | "ば"
            | "び"
            | "ぶ"
            | "べ"
            | "ぼ"
            | "うぃ"
            | "うぇ"
            | "うぉ"
            | "びゃ"
            | "びゅ"
            | "びぇ"
            | "びょ"
            | "でぃ"
            | "どぅ"
            | "でゅ"
            | "が"
            | "ぎ"
            | "ぐ"
            | "げ"
            | "ご"
            | "ぎゃ"
            | "ぎゅ"
            | "ぎぇ"
            | "ぎょ"
    )
}

fn normalize_phrase(tokens: &[KanaToken]) -> Result<String, String> {
    let mut normalized: Vec<KanaToken> = Vec::with_capacity(tokens.len());

    for token in tokens.iter().cloned() {
        // AquesTalk 1.8 はアクセント句先頭の長音を受け付けない。前の母音が
        // 存在しないので、ここでは長音自体を落とす。
        if normalized.is_empty() && token.normal == "ー" {
            continue;
        }

        // 促音の連続は 1 個へ畳む。アクセントが後側に付いていた場合は
        // 残す促音へ移す。
        if token.normal == "っ"
            && normalized
                .last()
                .map(|previous| previous.normal == "っ")
                .unwrap_or(false)
        {
            if token.accent_after {
                if let Some(previous) = normalized.last_mut() {
                    previous.accent_after = true;
                }
            }
            continue;
        }

        // 「っー」は禁止。長音には伸ばす母音が無いため長音を落とす。
        if token.normal == "ー"
            && normalized
                .last()
                .map(|previous| previous.normal == "っ")
                .unwrap_or(false)
        {
            if token.accent_after {
                if let Some(previous) = normalized.last_mut() {
                    previous.accent_after = true;
                }
            }
            continue;
        }

        normalized.push(token);
    }

    if normalized.is_empty() {
        return Err("AquesTalk 1.8 で有効な読み記号がありません".to_string());
    }

    // 句末促音は旧 DLL で記号列エラーになるため、通常の「つ」に開く。
    if let Some(last) = normalized.last_mut() {
        if last.normal == "っ" {
            last.normal = "つ".to_string();
            last.rendered = "つ".to_string();
            last.devoiced = false;
        }
    }

    // 手動無声化の直後に禁止されている読みが来る場合は、読みそのものを
    // 変えず、手動無声化だけを解除する。
    for index in 0..normalized.len().saturating_sub(1) {
        if normalized[index].devoiced
            && forbidden_after_devoiced(&normalized[index + 1].normal)
        {
            normalized[index].disable_devoicing();
        }
    }

    if normalized.len() > MAX_READINGS_PER_PHRASE {
        return Err(format!(
            "AquesTalk 1.8 の1アクセント句あたりの読み記号上限を超えています: {} > {}",
            normalized.len(), MAX_READINGS_PER_PHRASE
        ));
    }

    if normalized.iter().filter(|token| token.accent_after).count() > 1 {
        return Err("1つのアクセント句に複数のアクセント記号があります".to_string());
    }

    let mut out = String::new();
    for token in normalized {
        out.push_str(&token.rendered);
        if token.accent_after {
            out.push('\'');
        }
    }
    Ok(out)
}

/// AqKanji2Koe が生成したかな音声記号列を、AquesTalk 1.8 の書式と
/// 読み記号並びの制限に合わせて検証・正規化する。
///
/// 未定義読みやタグをそのまま DLL に渡さず、102/105/106/107/108 の
/// 原因になる列はこの段階で排除する。長すぎる1アクセント句は勝手に
/// 分割せず、変換エラーとして返す。
pub(crate) fn sanitize_kana(input: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut phrase: Vec<KanaToken> = Vec::new();
    let mut rest = input;

    while !rest.is_empty() {
        if let Some((delimiter, bytes)) = take_delimiter(rest) {
            if phrase.is_empty() {
                return Err("句切記号の前に有効な読み記号がありません".to_string());
            }
            out.push_str(&normalize_phrase(&phrase)?);
            out.push_str(delimiter);
            phrase.clear();
            rest = &rest[bytes..];
            continue;
        }

        if rest.starts_with('\'') {
            let Some(last) = phrase.last_mut() else {
                return Err("アクセント記号の前に読み記号がありません".to_string());
            };
            if phrase.iter().any(|token| token.accent_after) {
                return Err("1つのアクセント句に複数のアクセント記号があります".to_string());
            }
            last.accent_after = true;
            rest = &rest[1..];
            continue;
        }

        if let Some((rendered, normal, bytes)) = take_devoiced(rest) {
            phrase.push(KanaToken::devoiced(rendered, normal));
            rest = &rest[bytes..];
            continue;
        }

        if let Some((reading, bytes)) = take_reading(rest) {
            phrase.push(KanaToken::normal(reading));
            rest = &rest[bytes..];
            continue;
        }

        return Err(format!(
            "AquesTalk 1.8 に未定義の読み記号またはタグが含まれています: {rest}"
        ));
    }

    if !phrase.is_empty() {
        return Err("音声記号列の末尾に句切記号がありません".to_string());
    }
    if out.is_empty() {
        return Err("AquesTalk 1.8 で有効な読み記号がありません".to_string());
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::sanitize_kana;

    #[test]
    fn leaves_valid_output_unchanged() {
        assert_eq!(
            sanitize_kana("これわ/おんせ'ーきごーです。").unwrap(),
            "これわ/おんせ'ーきごーです。"
        );
        assert_eq!(sanitize_kana("_キく、て'_スと。").unwrap(), "_キく、て'_スと。");
    }

    #[test]
    fn normalizes_old_dll_sequence_restrictions() {
        assert_eq!(sanitize_kana("えっ。").unwrap(), "えつ。");
        assert_eq!(sanitize_kana("えっっと。").unwrap(), "えっと。");
        assert_eq!(sanitize_kana("ーか。").unwrap(), "か。");
        assert_eq!(sanitize_kana("えっーか。").unwrap(), "えっか。");
        assert_eq!(sanitize_kana("_キー。").unwrap(), "きー。");
        assert_eq!(sanitize_kana("_キあ。").unwrap(), "きあ。");
    }

    #[test]
    fn rejects_undefined_readings_and_tags_before_the_dll() {
        assert!(sanitize_kana("ぢ。").is_err());
        assert!(sanitize_kana("ゔぁ。").is_err());
        assert!(sanitize_kana("<NUM VAL=12>。").is_err());
    }

    #[test]
    fn rejects_multiple_accents_in_one_phrase() {
        assert!(sanitize_kana("あ'いう'。").is_err());
    }

    #[test]
    fn refuses_overlong_phrase_without_splitting_or_truncating() {
        let valid = format!("{}。", "あ".repeat(255));
        assert_eq!(sanitize_kana(&valid).unwrap(), valid);

        let invalid = format!("{}。", "あ".repeat(256));
        assert!(sanitize_kana(&invalid).is_err());
    }

    #[test]
    fn rejects_empty_or_unterminated_output() {
        assert!(sanitize_kana("。").is_err());
        assert!(sanitize_kana("あ").is_err());
    }
}
