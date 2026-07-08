//! Embedded AsyncAPI HTML documentation.
//!
//! At **compile time** the contents of `crates/dure-asyncapi-gen/docs/api-docs/` are baked into
//! the binary via `include_bytes!`.  At **runtime** the server first looks for the
//! files on disk (convenient during development); if the directory is absent or the
//! file is missing it falls back to the embedded copy automatically.
//!
//! To regenerate the docs (e.g. after changing `docs/asyncapi.json`):
//!
//! ```sh
//! cd crates/dure-asyncapi-gen && asyncapi generate fromTemplate \
//!     ../../docs/asyncapi.json @asyncapi/html-template \
//!     -o docs/api-docs --force-write
//! ```
//!
//! Then `cargo build` picks them up — no extra steps needed.

// Paths are relative to this source file's location:
//   mobile/src/wss/server/asyncapi_docs.rs
//   → ../../../../crates/dure-asyncapi-gen/docs/api-docs/

static INDEX_HTML: &[u8] =
    include_bytes!("../../../../crates/dure-asyncapi-gen/docs/api-docs/index.html");
static ASYNCAPI_CSS: &[u8] =
    include_bytes!("../../../../crates/dure-asyncapi-gen/docs/api-docs/css/asyncapi.min.css");
static GLOBAL_CSS: &[u8] =
    include_bytes!("../../../../crates/dure-asyncapi-gen/docs/api-docs/css/global.min.css");
static APP_JS: &[u8] = include_bytes!("../../../../crates/dure-asyncapi-gen/docs/api-docs/js/app.js");
static ASYNCAPI_UI_JS: &[u8] =
    include_bytes!("../../../../crates/dure-asyncapi-gen/docs/api-docs/js/asyncapi-ui.min.js");

/// Serve an AsyncAPI docs file.
///
/// Strategy (in order):
/// 1. Filesystem — `docs_dir/file` if it exists (dev-mode reload without recompile).
/// 2. Embedded bytes baked in at compile time (production / CI).
///
/// Returns `(bytes, content_type)` or `None` if the file is unknown.
pub fn get(docs_dir: &std::path::Path, file: &str) -> Option<(Vec<u8>, &'static str)> {
    let file = if file.is_empty() { "index.html" } else { file };

    // 1 — Filesystem (development)
    let fs_path = docs_dir.join(file);
    if fs_path.exists() {
        if let Ok(bytes) = std::fs::read(&fs_path) {
            return Some((bytes, mime(file)));
        }
    }

    // 2 — Embedded (production)
    let bytes: Option<&'static [u8]> = match file {
        "index.html" => Some(INDEX_HTML),
        "css/asyncapi.min.css" => Some(ASYNCAPI_CSS),
        "css/global.min.css" => Some(GLOBAL_CSS),
        "js/app.js" => Some(APP_JS),
        "js/asyncapi-ui.min.js" => Some(ASYNCAPI_UI_JS),
        _ => None,
    };

    bytes.map(|b| (b.to_vec(), mime(file)))
}

fn mime(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json"
    } else {
        "application/octet-stream"
    }
}
