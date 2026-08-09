use aqkanji2koe::AqKanji2Koe;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Kanji2Koe {
    inner: AqKanji2Koe,
}

#[wasm_bindgen]
impl Kanji2Koe {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Kanji2Koe, JsError> {
        console_error_panic_hook::set_once();

        let inner = AqKanji2Koe::new().map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner })
    }

    pub fn convert(&self, text: &str) -> Result<String, JsError> {
        self.inner
            .convert(text)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = convertRoman)]
    pub fn convert_roman(&self, text: &str) -> Result<String, JsError> {
        self.inner
            .convert_roman(text)
            .map_err(|e| JsError::new(&e.to_string()))
    }
}
