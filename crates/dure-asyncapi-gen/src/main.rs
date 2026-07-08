//! Generate AsyncAPI specification files
//!
//! This binary generates the AsyncAPI 3.0 specification for the Dure WebSocket protocol
//! in both JSON and YAML formats.
//!
//! ## Usage
//!
//! ```sh
//! cargo run
//! ```
//!
//! ## Output
//!
//! - `../docs/asyncapi.json` - JSON format specification
//! - `../docs/asyncapi.yaml` - YAML format specification

use dure_asyncapi_gen::DureApi;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Collect all `$defs` entries from every message payload, merge them into
/// `components/schemas`, and rewrite every `$ref: "#/$defs/X"` to
/// `$ref: "#/components/schemas/X"` throughout the document.
///
/// `schemars` emits `$defs` local to each schema object, but the `$ref`
/// pointers it generates use JSON-Pointer paths from the document root
/// (`#/$defs/…`).  AsyncAPI tooling resolves those pointers against the root
/// and therefore cannot find the definitions unless they are promoted.
fn fix_schema_refs(mut doc: Value) -> Value {
    // 1. Gather every $defs entry from all message payloads.
    let mut schemas: BTreeMap<String, Value> = BTreeMap::new();

    if let Some(messages) = doc
        .pointer_mut("/components/messages")
        .and_then(Value::as_object_mut)
    {
        for msg in messages.values_mut() {
            if let Some(defs) = msg
                .pointer_mut("/payload/$defs")
                .and_then(Value::as_object_mut)
            {
                for (k, v) in defs.iter() {
                    schemas.entry(k.clone()).or_insert_with(|| v.clone());
                }
            }
            // Remove $defs and $schema from the payload – they are no longer needed there.
            if let Some(payload) = msg.pointer_mut("/payload").and_then(Value::as_object_mut) {
                payload.remove("$defs");
                payload.remove("$schema");
            }
        }
    }

    // 2. Insert collected schemas into components/schemas.
    if !schemas.is_empty() {
        let comp_schemas = doc
            .pointer_mut("/components")
            .and_then(Value::as_object_mut)
            .map(|c| {
                c.entry("schemas")
                    .or_insert_with(|| Value::Object(serde_json::Map::new()))
            });
        if let Some(Value::Object(map)) = comp_schemas {
            for (k, v) in schemas {
                map.entry(k).or_insert(v);
            }
        }
    }

    // 3. Rewrite all $ref values recursively.
    rewrite_refs(&mut doc);

    doc
}

fn rewrite_refs(val: &mut Value) {
    match val {
        Value::Object(map) => {
            if let Some(Value::String(r)) = map.get_mut("$ref") {
                if let Some(name) = r.strip_prefix("#/$defs/") {
                    *r = format!("#/components/schemas/{name}");
                }
            }
            for v in map.values_mut() {
                rewrite_refs(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                rewrite_refs(v);
            }
        }
        _ => {}
    }
}

fn main() -> anyhow::Result<()> {
    println!("🚀 Generating AsyncAPI specification for Dure WebSocket API...\n");

    // Generate the spec
    let spec = DureApi::asyncapi_spec();

    // Ensure docs directory exists (relative to workspace root)
    let docs_dir = Path::new("../docs");
    if !docs_dir.exists() {
        fs::create_dir_all(docs_dir)?;
        println!("✓ Created docs/ directory");
    }

    // Serialize to Value so we can post-process schema refs before writing.
    let raw: Value = serde_json::to_value(&spec)?;
    let fixed = fix_schema_refs(raw);

    // Generate JSON
    let json_path = docs_dir.join("asyncapi.json");
    let json = serde_json::to_string_pretty(&fixed)?;
    fs::write(&json_path, &json)?;
    println!("✓ Generated {}", json_path.display());
    println!("  {} bytes", json.len());

    // Generate YAML
    let yaml_path = docs_dir.join("asyncapi.yaml");
    let yaml = serde_yaml::to_string(&fixed)?;
    fs::write(&yaml_path, &yaml)?;
    println!("✓ Generated {}", yaml_path.display());
    println!("  {} bytes", yaml.len());

    println!("\n📊 Specification Summary:");
    println!("  Title: {}", spec.info.title);
    println!("  Version: {}", spec.info.version);
    println!("  AsyncAPI: {}", spec.asyncapi);

    if let Some(servers) = &spec.servers {
        println!("  Servers: {}", servers.len());
        for (name, _) in servers {
            println!("    - {}", name);
        }
    }

    if let Some(channels) = &spec.channels {
        println!("  Channels: {}", channels.len());
        for (name, _) in channels {
            println!("    - {}", name);
        }
    }

    if let Some(components) = &spec.components {
        if let Some(messages) = &components.messages {
            println!("  Messages: {}", messages.len());
        }
    }

    if let Some(schemas) = fixed.pointer("/components/schemas").and_then(Value::as_object) {
        println!("  Schemas promoted to components/schemas: {}", schemas.len());
    }

    println!("\n🎯 Next steps:");
    println!("  1. View in AsyncAPI Studio: https://studio.asyncapi.com/");
    println!("     - Upload: {}", json_path.display());
    println!("  2. Generate documentation:");
    println!("     - npm install -g @asyncapi/cli");
    println!("     - asyncapi generate fromTemplate {} @asyncapi/html-template -o docs/api-docs", json_path.display());
    println!("  3. Generate client code:");
    println!("     - asyncapi generate fromTemplate {} @asyncapi/ts-nats-template -o clients/typescript", json_path.display());

    Ok(())
}
