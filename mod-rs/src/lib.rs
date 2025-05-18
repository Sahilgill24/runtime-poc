use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Zinnia, js_name = log)]
    pub fn zinnia_log(s: &str);

    #[wasm_bindgen(js_namespace = Zinnia, js_name = sleep)]
    pub async fn zinnia_sleep(duration_in_ms: u64);

}

type ZinniaError = JsValue;

#[wasm_bindgen()]
pub async fn run() -> Result<(), ZinniaError> {
  zinnia_log("Good night...");
  zinnia_sleep(1000).await;
  zinnia_log("Good morning!");
  Ok(())
}