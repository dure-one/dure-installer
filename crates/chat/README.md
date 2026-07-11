# chat - Chat Service Layer

Email-based chat service using deltachat-core with async-compat bridge.

## Features

- deltachat-core integration (SMTP/IMAP email chat)
- async-compat bridge (tokio ↔ smol)
- Event broadcasting with async-broadcast
- End-to-end encryption (rPGP, Autocrypt)

## Usage

```rust
use chat::ChatService;

smol::block_on(async {
    let service = ChatService::new("./chat.db".into())
        .await
        .unwrap();

    let mut events = service.subscribe_events();

    while let Ok(event) = events.recv().await {
        println!("Chat event: {:?}", event);
    }
});
```

## Architecture

- Primary runtime: smol
- deltachat-core runtime: tokio (bridged via async-compat)
- Event bus: async-broadcast
