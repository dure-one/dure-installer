# ws - WebSocket and HTTP/2 Server

High-performance WebSocket and HTTP/2 server built on wtx + smol.

## Features

- HTTP/2 server with wtx (runtime-agnostic)
- WebSocket server (RFC 6455)
- TLS support via rustls
- Static file serving with directory traversal protection
- Middleware chain (CORS, compression, sessions)
- DDoS protection via fail2ban-rs (optional)
- Pure smol async runtime

## Usage

```rust
use ws::{ServerConfig, WsServer};

#[smol::main]
async fn main() {
    let config = ServerConfig::new("example.com");

    WsServer::new(config)
        .run()
        .await
        .unwrap();
}
```

## Features

- `chat`: Enable chat service integration

## Performance Targets

- 50,000+ HTTP/2 requests/second
- 10,000+ concurrent WebSocket connections
- <1ms p99 latency
