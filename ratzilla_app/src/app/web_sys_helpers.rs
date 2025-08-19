use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response, console};
use wasm_bindgen_futures::spawn_local;

pub async fn fetch_url(url: &str) -> Result<String, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    
    let request = Request::new_with_str(url)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        return Err(JsValue::from_str(&format!("Request failed: {}", resp.status())));
    }
    
    let text = JsFuture::from(resp.text()?).await?;
    text.as_string().ok_or_else(|| JsValue::from_str("Failed to convert to string"))
}
