use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::Response;

use crate::{SymbolError, SymbolFont};

impl SymbolFont {
    /// Fetch a TrueType asset from a browser window and inject its bytes.
    ///
    /// Serve the file on the app's origin or enable CORS on the asset host.
    /// Each call loads one family. Retain the font or its SVGs for reuse.
    pub async fn fetch(url: &str) -> Result<Self, SymbolError> {
        let window = web_sys::window()
            .ok_or_else(|| SymbolError::Fetch("browser window is unavailable".into()))?;
        let response: Response = window
            .fetch_with_str(url)
            .await
            .and_then(JsCast::dyn_into)
            .map_err(fetch_error)?;
        if !response.ok() {
            return Err(SymbolError::HttpStatus(response.status()));
        }
        let buffer = response
            .array_buffer()
            .map_err(fetch_error)?
            .await
            .map_err(fetch_error)?;
        Self::from_bytes(Uint8Array::new(&buffer).to_vec())
    }
}

fn fetch_error(error: JsValue) -> SymbolError {
    SymbolError::Fetch(format!("{error:?}"))
}
