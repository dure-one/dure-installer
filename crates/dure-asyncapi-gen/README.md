# Dure AsyncAPI Specification Generator

This standalone crate generates AsyncAPI 3.0 specifications for the Dure WebSocket protocol.

## Purpose

This is a **standalone** crate (not part of the main Dure workspace) that:
1. Defines all WebSocket message types for the Dure platform
2. Generates AsyncAPI 3.0 specifications in JSON and YAML formats
3. Provides a single source of truth for the WebSocket API documentation

## Why Standalone?

The generator is standalone rather than a binary in the main `mobile` crate because:
- **Faster builds**: Doesn't require egui, tungstenite, or other heavy dependencies
- **CI/CD friendly**: Can run in minimal Docker containers
- **Clean separation**: Documentation generation doesn't depend on application code
- **Avoids compilation issues**: Main crate has incomplete modules being migrated from beads_rust

## Usage

```bash
# Build and run
cargo run --release

# Or build separately
cargo build --release
./target/release/dure-asyncapi-gen
```

## Output

The generator creates two files in the `../docs` directory:

- `../docs/asyncapi.json` - JSON format (3.8 MB)
- `../docs/asyncapi.yaml` - YAML format (2.7 MB)

## Generated Specification

### Statistics
- **AsyncAPI Version**: 3.0.0
- **Message Types**: 67
- **Servers**: 2 (production, development)
- **Channels**: 2 (client_commands, server_responses)

### Message Categories
- Authentication (login, logout)
- Hosting management
- Member/directory operations
- Channel management
- Messaging
- Product catalog
- Order processing
- Payment integration
- Review management

## Project Structure

```
asyncapi-gen/
├── src/
│   ├── main.rs              # Generator binary
│   ├── lib.rs               # Library exports
│   ├── asyncapi_spec.rs     # AsyncAPI specification definition
│   └── messages/            # WebSocket message types
│       ├── mod.rs           # ClientMessage & ServerMessage enums
│       ├── auth.rs          # Authentication messages
│       ├── hosting.rs       # Hosting management
│       ├── member.rs        # Member operations
│       ├── channel.rs       # Channel management
│       ├── message.rs       # Messaging
│       ├── product.rs       # Product catalog
│       ├── order.rs         # Order processing
│       ├── payment.rs       # Payment integration
│       └── review.rs        # Reviews
├── Cargo.toml               # Dependencies (standalone workspace)
└── README.md                # This file
```

## Dependencies

```toml
asyncapi-rust = "0.2"           # AsyncAPI spec generation
schemars = "1.1"                # JSON Schema derivation
serde = "1.0"                   # Serialization
serde_json = "1.0"              # JSON output
serde_yaml = "0.9"              # YAML output
chrono = "0.4"                  # DateTime support
anyhow = "1.0"                  # Error handling
```

## How It Works

### 1. Define Message Types

```rust
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "auth.login")]
    AuthLogin(AuthLoginRequest),
    // ... more messages
}
```

### 2. Create AsyncAPI Spec

```rust
#[derive(AsyncApi)]
#[asyncapi(
    title = "Dure WebSocket API",
    version = "1.0.0",
    description = "Distributed e-commerce WebSocket protocol"
)]
#[asyncapi_server(
    name = "production",
    host = "{domain}",
    protocol = "wss",
    pathname = "/api/ws"
)]
#[asyncapi_messages(ClientMessage, ServerMessage)]
pub struct DureApi;
```

### 3. Generate Specifications

```rust
fn main() -> anyhow::Result<()> {
    let spec = DureApi::asyncapi_spec();

    let json = serde_json::to_string_pretty(&spec)?;
    fs::write("../docs/asyncapi.json", json)?;

    let yaml = serde_yaml::to_string(&spec)?;
    fs::write("../docs/asyncapi.yaml", yaml)?;

    Ok(())
}
```

## Using the Generated Spec

### View in AsyncAPI Studio

1. Visit https://studio.asyncapi.com/
2. Upload `../docs/asyncapi.json`
3. Explore the interactive documentation

### Generate HTML Documentation

```bash
npm install -g @asyncapi/cli
asyncapi generate fromTemplate ../docs/asyncapi.json @asyncapi/html-template -o ../docs/api-docs
```

### Generate Client Code

```bash
# TypeScript client
asyncapi generate fromTemplate ../docs/asyncapi.json @asyncapi/ts-nats-template -o ../clients/typescript

# Python client
asyncapi generate fromTemplate ../docs/asyncapi.json @asyncapi/python-paho-template -o ../clients/python
```

## Integration with Main Project

Once the main `mobile` crate compilation issues are resolved, the message types can be used directly:

```rust
use dure::messages::{ClientMessage, ServerMessage};
use tungstenite::{Message, WebSocket};

// Receiving client messages
if let Ok(Message::Text(text)) = socket.read() {
    let msg: ClientMessage = serde_json::from_str(&text)?;
    match msg {
        ClientMessage::AuthLogin(req) => {
            // Handle authentication
        }
        ClientMessage::MessageSend(req) => {
            // Handle message sending
        }
        // ... other message types
    }
}

// Sending server messages
let response = ServerMessage::AuthResponse(AuthResponse {
    success: true,
    session_id: Some("session_123".to_string()),
    // ...
});

let json = serde_json::to_string(&response)?;
socket.write(Message::Text(json))?;
```

## CI/CD Integration

Add to your GitHub Actions workflow:

```yaml
name: Generate API Documentation
on: [push]
jobs:
  generate-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Generate AsyncAPI Spec
        run: |
          cd asyncapi-gen
          cargo run --release
      - name: Upload Spec
        uses: actions/upload-artifact@v2
        with:
          name: asyncapi-spec
          path: docs/asyncapi.*
```

## Development

### Adding New Message Types

1. Add message struct to appropriate module (e.g., `messages/auth.rs`)
2. Add variant to `ClientMessage` or `ServerMessage` enum in `messages/mod.rs`
3. Run `cargo run --release` to regenerate specifications
4. Verify in AsyncAPI Studio

### Message Type Guidelines

- Use `#[serde(tag = "type")]` for type discriminator
- Use `#[serde(rename = "...")]` for message type names
- Include doc comments - they appear in generated schemas
- Use `DateTime<Utc>` for timestamps
- Use `Option<T>` for optional fields
- Derive: `Serialize`, `Deserialize`, `JsonSchema`, `Clone`, `Debug`

## Troubleshooting

### Build fails with workspace errors

Ensure `Cargo.toml` contains:
```toml
[workspace]
# This is a standalone crate
```

### Generated files are empty

Check that:
1. Message types have `#[derive(ToAsyncApiMessage)]`
2. Enum has `#[serde(tag = "type")]`
3. `#[asyncapi_messages(...)]` includes your message enums

### Schemas are missing fields

Ensure all fields derive `JsonSchema` and nested types also derive it.

## License

Dual-licensed under MIT OR Apache-2.0 (same as main Dure project).

## References

- [Dure AsyncAPI Integration Docs](../docs/ASYNCAPI_INTEGRATION.md)
- [AsyncAPI Specification](https://www.asyncapi.com/docs/reference/specification/v3.0.0)
- [asyncapi-rust](https://github.com/mlilback/asyncapi-rust)
- [Main Dure Project](../README.md)
