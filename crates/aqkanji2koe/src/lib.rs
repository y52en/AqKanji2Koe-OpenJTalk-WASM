//! # aqkanji2koe
//!
//! 漢字かな交じりテキストを AquesTalk 用音声記号列に変換するライブラリ。
//!
//! 内部で [jpreprocess](https://docs.rs/jpreprocess) (OpenJTalk の Rust 実装) と
//! NAIST-JDic を使用して形態素解析・アクセント推定を行う。
//!
//! ## 出力形式
//!
//! - **かな記法**: ひらがな + 半角記号 (AquesTalk 標準入力形式、UTF-8)
//! - **ローマ字記法**: ASCII のみ (AquesTalk pico 準拠)
//!
//! ## 基本的な使い方
//!
//! ```rust
//! use aqkanji2koe::AqKanji2Koe;
//!
//! let converter = AqKanji2Koe::new().expect("初期化失敗");
//!
//! let kana   = converter.convert("日本語のテキスト").unwrap();
//! let roman  = converter.convert_roman("日本語のテキスト").unwrap();
//!
//! println!("かな: {kana}");
//! println!("ローマ字: {roman}");
//! ```

mod aquestalk;
pub mod converter;
mod english;
pub mod error;
pub mod mora;
pub mod phoneme;

pub use converter::OutputFormat;
pub use error::{Error, Result};

use converter::{nodes_to_phoneme, NodeData};
use e2k::C2k;
use jpreprocess::kind::JPreprocessDictionaryKind;
use jpreprocess::{JPreprocess, SystemDictionaryConfig};

type ProcessFn = dyn Fn(&str) -> Result<Vec<NodeData>> + Send + Sync;

/// 漢字かな交じりテキストを AquesTalk 音声記号列に変換する。
///
/// jpreprocess (OpenJTalk) と NAIST-JDic をバンドルして使用する。
/// スレッド間で共有可能 (`Send + Sync`)。
pub struct AqKanji2Koe {
    process: Box<ProcessFn>,
}

impl AqKanji2Koe {
    /// バンドル済み NAIST-JDic を使って変換器を初期化する。
    ///
    /// # Errors
    ///
    /// 辞書の読み込みに失敗した場合に [`Error::Init`] を返す。
    pub fn new() -> Result<Self> {
        let system = SystemDictionaryConfig::Bundled(JPreprocessDictionaryKind::NaistJdic)
            .load()
            .map_err(|e| Error::Init(e.to_string()))?;

        let jp = JPreprocess::with_dictionaries(system, None);
        let english_to_katakana = C2k::new(64);

        let process = Box::new(move |text: &str| -> Result<Vec<NodeData>> {
            let normalized_text = english::katakanize_ascii_words(text, &english_to_katakana);
            // preprocess() でアクセント句連結と無声化フラグを確定させる。
            let mut njd = jp
                .text_to_njd(&normalized_text)
                .map_err(|e| Error::Processing(e.to_string()))?;
            njd.preprocess();

            let nodes = njd
                .nodes
                .iter()
                .map(|node| {
                    let pron = node.get_pron();
                    // jpreprocess は無声化モーラの末尾に U+2019 を付けるので除去する。
                    let pron_moras: Vec<(String, bool)> = pron
                        .moras()
                        .iter()
                        .map(|m| {
                            let rendered = m.to_string();
                            let katakana = rendered
                                .strip_suffix('\u{2019}')
                                .unwrap_or(&rendered)
                                .to_string();
                            (katakana, m.is_voiced)
                        })
                        .collect();
                    NodeData {
                        original: node.get_string().to_string(),
                        pron_moras,
                        accent: pron.accent(),
                        chain_with_prev: node.get_chain_flag().unwrap_or(false),
                        is_pron_empty: pron.is_empty(),
                        is_touten: pron.is_touten(),
                        is_question: pron.is_question(),
                    }
                })
                .collect();

            Ok(nodes)
        });

        Ok(Self { process })
    }

    fn convert_with_format(&self, text: &str, format: OutputFormat) -> Result<String> {
        let nodes = (self.process)(text)?;
        let phonemes = nodes_to_phoneme(&nodes, format);
        if matches!(phonemes.as_str(), "。" | ".") {
            return Ok(String::new());
        }
        if format == OutputFormat::Kana {
            aquestalk::sanitize_kana(&phonemes).map_err(Error::Processing)
        } else {
            Ok(phonemes)
        }
    }

    /// 漢字かな交じりテキストを **かな音声記号列** (UTF-8) に変換する。
    ///
    /// 出力例: `"これわ/おんせ'ーきごーです。"`
    ///
    /// AquesTalk 1.8 の未定義読み・禁止された読み並びを DLL に渡さないよう、
    /// 出力直前に検証と安全な正規化を行う。1アクセント句が長すぎる場合は
    /// 勝手に分割せず [`Error::Processing`] を返す。
    pub fn convert(&self, text: &str) -> Result<String> {
        self.convert_with_format(text, OutputFormat::Kana)
    }

    /// 漢字かな交じりテキストを **ローマ字音声記号列** (ASCII) に変換する。
    ///
    /// 出力例: `"korewa/onse'-kigo-desu."`
    ///
    /// # Errors
    ///
    /// jpreprocess の処理に失敗した場合に [`Error::Processing`] を返す。
    pub fn convert_roman(&self, text: &str) -> Result<String> {
        self.convert_with_format(text, OutputFormat::Roman)
    }
}

impl std::fmt::Debug for AqKanji2Koe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AqKanji2Koe").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::AqKanji2Koe;

    #[test]
    fn noisy_and_colloquial_inputs_convert_without_errors() {
        let converter = AqKanji2Koe::new().unwrap();
        let inputs = [
            " 基本的な動作確認 ",
            " (sample value) ",
            "  ",
            " ☆彡 おはようー ",
            " 今日は晴れだねーまたねー ",
            " ケーキーーー！！！ ",
            " 学校へ行くよーーーー ",
            " 明日も会おうねーーーー ",
            " 記号wサンプル できたよーー ",
            " Hello world ",
        ];

        for input in inputs {
            converter
                .convert(input)
                .unwrap_or_else(|error| panic!("かな変換に失敗: {input:?}: {error}"));
            converter
                .convert_roman(input)
                .unwrap_or_else(|error| panic!("ローマ字変換に失敗: {input:?}: {error}"));
        }
    }

    #[test]
    fn empty_and_symbol_only_inputs_return_empty_output() {
        let converter = AqKanji2Koe::new().unwrap();

        assert_eq!(converter.convert("").unwrap(), "");
        assert_eq!(converter.convert("  ").unwrap(), "");
        assert_eq!(converter.convert("!!!").unwrap(), "");
        assert_eq!(converter.convert_roman("  ").unwrap(), "");
    }

    #[test]
    fn english_words_are_katakanized_before_analysis() {
        let converter = AqKanji2Koe::new().unwrap();

        let kana = converter.convert("constants").unwrap();
        assert_eq!(kana, "こん_スた'んつ。");
    }
}
