//! I/O abstraction for cross-platform HTTP support

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::io::Read;

    pub async fn http_get(url: &str) -> anyhow::Result<Vec<u8>> {
        let response = ureq::get(url).call()?;
        let mut body = Vec::new();
        response.into_reader().read_to_end(&mut body)?;
        Ok(body)
    }

    pub async fn http_post(url: &str, body: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let response = ureq::post(url).send_bytes(&body)?;
        let mut response_body = Vec::new();
        response.into_reader().read_to_end(&mut response_body)?;
        Ok(response_body)
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    pub async fn http_get(url: &str) -> anyhow::Result<Vec<u8>> {
        let window = web_sys::window().ok_or_else(|| anyhow::anyhow!("no window"))?;
        let resp_value = JsFuture::from(window.fetch_with_str(url))
            .await
            .map_err(|_| anyhow::anyhow!("fetch failed"))?;
        let resp: Response = resp_value
            .dyn_into()
            .map_err(|_| anyhow::anyhow!("not a Response"))?;
        let array_buffer = JsFuture::from(
            resp.array_buffer()
                .map_err(|_| anyhow::anyhow!("array_buffer failed"))?,
        )
        .await
        .map_err(|_| anyhow::anyhow!("array_buffer promise failed"))?;
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        Ok(uint8_array.to_vec())
    }

    pub async fn http_post(url: &str, body: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.body(Some(&js_sys::Uint8Array::from(&body[..]).into()));

        let request = Request::new_with_str_and_init(url, &opts)
            .map_err(|_| anyhow::anyhow!("Request creation failed"))?;
        let window = web_sys::window().ok_or_else(|| anyhow::anyhow!("no window"))?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|_| anyhow::anyhow!("fetch failed"))?;
        let resp: Response = resp_value
            .dyn_into()
            .map_err(|_| anyhow::anyhow!("not a Response"))?;
        let array_buffer = JsFuture::from(
            resp.array_buffer()
                .map_err(|_| anyhow::anyhow!("array_buffer failed"))?,
        )
        .await
        .map_err(|_| anyhow::anyhow!("array_buffer promise failed"))?;
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        Ok(uint8_array.to_vec())
    }
}
