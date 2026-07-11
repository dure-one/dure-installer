# DeltaChat Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate DeltaChat encrypted messaging into Dure with configuration, contact management, and basic text messaging

**Architecture:** Actor-based MVVM with smol↔tokio runtime bridge. DeltaChatActor owns deltachat::Context, UI communicates via commands/events.

**Tech Stack:** `deltachat` 1.146+, `tokio` 1.x (runtime bridge), `egui` (UI), `smol` (actor runtime)

## Global Constraints

- Rust nightly toolchain required
- All platforms supported (Desktop, Android, WASM) - no `#[cfg]` restrictions on DeltaChat tab
- Database file: `<dure-data-dir>/deltachat-default.db`
- TDD workflow: Write test → Verify fails → Implement → Verify passes → Commit
- Follow Dure's actor pattern: commands.rs, events.rs, actor.rs, mod.rs
- DRY, YAGNI, frequent commits
- Use `#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]` for UI state

---

## Task 1: Add Dependencies

**Files:**
- Modify: `mobile/Cargo.toml`

**Interfaces:**
- Produces: `deltachat` and `tokio` dependencies available for use

- [ ] **Step 1: Add deltachat and tokio dependencies**

Add to `mobile/Cargo.toml` in the `[dependencies]` section:

```toml
deltachat = "1.146"
tokio = { version = "1", features = ["rt", "rt-multi-thread", "macros"] }
tempfile = "3"  # For tests
```

- [ ] **Step 2: Verify project compiles**

Run: `cargo check -p mobile`  
Expected: SUCCESS (may download dependencies)

- [ ] **Step 3: Commit**

```bash
git add mobile/Cargo.toml
git commit -m "deps: add deltachat and tokio for encrypted messaging

Add deltachat-core library for email-based encrypted messaging
Add tokio for runtime bridge (deltachat uses tokio, Dure uses smol)
Add tempfile for integration tests

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Create Command and Event Enums

**Files:**
- Create: `mobile/src/viewmodel/deltachat/mod.rs`
- Create: `mobile/src/viewmodel/deltachat/commands.rs`
- Create: `mobile/src/viewmodel/deltachat/events.rs`

**Interfaces:**
- Produces: `DeltaChatCommand` enum, `DeltaChatEvent` enum, `ContactInfo`, `ChatInfo`, `MessageInfo` structs

- [ ] **Step 1: Write tests for command enum**

Create `mobile/src/viewmodel/deltachat/commands.rs`:

```rust
//! DeltaChat actor commands

#[derive(Debug, Clone)]
pub enum DeltaChatCommand {
    // Configuration & Connection
    Configure {
        email: String,
        password: String,
    },
    Connect,
    Disconnect,
    GetConnectionStatus,
    
    // Contact Management
    AddContact {
        email: String,
    },
    ListContacts,
    GetContactInfo {
        contact_id: u32,
    },
    
    // Chat Management
    CreateChat {
        contact_id: u32,
    },
    ListChats,
    SelectChat {
        chat_id: u32,
    },
    
    // Messaging
    SendTextMessage {
        chat_id: u32,
        text: String,
    },
    ListMessages {
        chat_id: u32,
    },
    MarkMessagesSeen {
        chat_id: u32,
    },
    
    // Background sync
    FetchMessages,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_configure_command() {
        let cmd = DeltaChatCommand::Configure {
            email: "test@example.com".to_string(),
            password: "secret".to_string(),
        };
        
        match cmd {
            DeltaChatCommand::Configure { email, password } => {
                assert_eq!(email, "test@example.com");
                assert_eq!(password, "secret");
            }
            _ => panic!("Expected Configure command"),
        }
    }
    
    #[test]
    fn test_command_clone() {
        let cmd = DeltaChatCommand::AddContact {
            email: "alice@example.com".to_string(),
        };
        let cloned = cmd.clone();
        assert!(matches!(cloned, DeltaChatCommand::AddContact { .. }));
    }
    
    #[test]
    fn test_send_message_command() {
        let cmd = DeltaChatCommand::SendTextMessage {
            chat_id: 42,
            text: "Hello!".to_string(),
        };
        
        match cmd {
            DeltaChatCommand::SendTextMessage { chat_id, text } => {
                assert_eq!(chat_id, 42);
                assert_eq!(text, "Hello!");
            }
            _ => panic!("Expected SendTextMessage command"),
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p mobile deltachat::commands --lib`  
Expected: 3 tests PASS

- [ ] **Step 3: Write tests for event enum and info structs**

Create `mobile/src/viewmodel/deltachat/events.rs`:

```rust
//! DeltaChat actor events

#[derive(Debug, Clone)]
pub struct ContactInfo {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub is_blocked: bool,
}

#[derive(Debug, Clone)]
pub struct ChatInfo {
    pub id: u32,
    pub name: String,
    pub last_message: Option<String>,
    pub unread_count: u32,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub id: u32,
    pub from_contact_id: u32,
    pub from_name: String,
    pub text: String,
    pub timestamp: i64,
    pub is_outgoing: bool,
    pub is_seen: bool,
}

#[derive(Debug, Clone)]
pub enum DeltaChatEvent {
    // Configuration
    ConfigurationStarted,
    ConfigurationProgress { 
        progress: i32,
        comment: Option<String>,
    },
    Configured { 
        email: String,
    },
    ConfigurationFailed { 
        error: String,
    },
    
    // Connection
    Connected,
    Disconnected,
    ConnectionStatus { 
        connected: bool,
        email: Option<String>,
    },
    
    // Contacts
    ContactAdded { 
        contact: ContactInfo,
    },
    ContactsListed { 
        contacts: Vec<ContactInfo>,
    },
    ContactInfo { 
        contact: ContactInfo,
    },
    
    // Chats
    ChatCreated { 
        chat: ChatInfo,
    },
    ChatsListed { 
        chats: Vec<ChatInfo>,
    },
    ChatSelected { 
        chat_id: u32,
    },
    
    // Messages
    MessageSent { 
        msg_id: u32,
        chat_id: u32,
    },
    MessagesListed { 
        chat_id: u32,
        messages: Vec<MessageInfo>,
    },
    NewMessageReceived { 
        chat_id: u32,
        message: MessageInfo,
    },
    MessagesSeen { 
        chat_id: u32,
    },
    
    // Progress & Errors
    Progress { 
        operation: String,
        progress: f32,
    },
    Error { 
        operation: String,
        error: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_contact_info() {
        let contact = ContactInfo {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            is_blocked: false,
        };
        
        assert_eq!(contact.id, 1);
        assert_eq!(contact.name, "Alice");
        assert_eq!(contact.email, "alice@example.com");
        assert!(!contact.is_blocked);
    }
    
    #[test]
    fn test_contact_info_clone() {
        let contact = ContactInfo {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            is_blocked: false,
        };
        
        let cloned = contact.clone();
        assert_eq!(cloned.id, contact.id);
        assert_eq!(cloned.email, contact.email);
    }
    
    #[test]
    fn test_message_info() {
        let msg = MessageInfo {
            id: 42,
            from_contact_id: 1,
            from_name: "Bob".to_string(),
            text: "Hello!".to_string(),
            timestamp: 1234567890,
            is_outgoing: false,
            is_seen: false,
        };
        
        assert_eq!(msg.id, 42);
        assert_eq!(msg.text, "Hello!");
        assert!(!msg.is_outgoing);
    }
    
    #[test]
    fn test_configured_event() {
        let event = DeltaChatEvent::Configured {
            email: "user@example.com".to_string(),
        };
        
        match event {
            DeltaChatEvent::Configured { email } => {
                assert_eq!(email, "user@example.com");
            }
            _ => panic!("Expected Configured event"),
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p mobile deltachat::events --lib`  
Expected: 4 tests PASS

- [ ] **Step 5: Create module file**

Create `mobile/src/viewmodel/deltachat/mod.rs`:

```rust
//! DeltaChat actor module for encrypted messaging

pub mod commands;
pub mod events;

pub use commands::DeltaChatCommand;
pub use events::{ChatInfo, ContactInfo, DeltaChatEvent, MessageInfo};
```

- [ ] **Step 6: Verify all tests pass**

Run: `cargo test -p mobile deltachat --lib`  
Expected: 7 tests PASS

- [ ] **Step 7: Commit**

```bash
git add mobile/src/viewmodel/deltachat/
git commit -m "feat: add deltachat commands and events

Add command enum for UI→Actor communication
Add event enum for Actor→UI communication
Add info structs (ContactInfo, ChatInfo, MessageInfo)
All with unit tests

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Create DeltaChat Actor with Runtime Bridge

**Files:**
- Create: `mobile/src/viewmodel/deltachat/actor.rs`
- Modify: `mobile/src/viewmodel/deltachat/mod.rs`

**Interfaces:**
- Consumes: `DeltaChatCommand` (from Task 2)
- Produces: `DeltaChatActor` struct with `new()` and `run()` methods

- [ ] **Step 1: Write test for actor channel setup**

Create `mobile/src/viewmodel/deltachat/actor.rs`:

```rust
//! DeltaChat actor implementation

use crate::viewmodel::{common::ViewModelEvent, DeltaChatCommand, DeltaChatEvent};
use smol::channel::{Receiver, Sender};
use std::path::PathBuf;

pub struct DeltaChatActor {
    command_rx: Receiver<DeltaChatCommand>,
    event_tx: Sender<ViewModelEvent>,
    context: Option<deltachat::Context>,
    tokio_runtime: tokio::runtime::Runtime,
    database_path: PathBuf,
    is_configured: bool,
    is_connected: bool,
}

impl DeltaChatActor {
    pub fn new(
        command_rx: Receiver<DeltaChatCommand>,
        event_tx: Sender<ViewModelEvent>,
        database_path: PathBuf,
    ) -> Self {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("deltachat-tokio")
            .build()
            .expect("Failed to create tokio runtime");
        
        Self {
            command_rx,
            event_tx,
            context: None,
            tokio_runtime,
            database_path,
            is_configured: false,
            is_connected: false,
        }
    }
    
    pub async fn run(mut self) {
        log::info!("DeltaChatActor started");
        
        while let Ok(cmd) = self.command_rx.recv().await {
            log::debug!("DeltaChatActor received command: {:?}", cmd);
            
            let result = smol::unblock({
                let rt = &self.tokio_runtime;
                move || {
                    rt.block_on(async {
                        // TODO: handle commands
                        Ok::<(), String>(())
                    })
                }
            }).await;
            
            if let Err(e) = result {
                log::error!("DeltaChat command failed: {}", e);
                self.emit_event(DeltaChatEvent::Error {
                    operation: "command".to_string(),
                    error: e,
                });
            }
        }
        
        log::info!("DeltaChatActor stopped");
    }
    
    fn emit_event(&self, event: DeltaChatEvent) {
        let event_tx = self.event_tx.clone();
        smol::block_on(async move {
            let _ = event_tx.send(ViewModelEvent::DeltaChat(event)).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol::channel::unbounded;
    use std::path::PathBuf;
    
    #[test]
    fn test_actor_creation() {
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let db_path = PathBuf::from("/tmp/test.db");
        
        let actor = DeltaChatActor::new(cmd_rx, event_tx, db_path.clone());
        
        assert_eq!(actor.database_path, db_path);
        assert!(!actor.is_configured);
        assert!(!actor.is_connected);
        assert!(actor.context.is_none());
    }
    
    #[test]
    fn test_actor_receives_commands() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = unbounded();
            let (event_tx, _event_rx) = unbounded();
            
            cmd_tx.send(DeltaChatCommand::GetConnectionStatus).await.unwrap();
            
            let received = cmd_rx.recv().await.unwrap();
            assert!(matches!(received, DeltaChatCommand::GetConnectionStatus));
        });
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p mobile deltachat::actor --lib`  
Expected: 2 tests PASS

- [ ] **Step 3: Update module file to export actor**

Update `mobile/src/viewmodel/deltachat/mod.rs`:

```rust
//! DeltaChat actor module for encrypted messaging

pub mod actor;
pub mod commands;
pub mod events;

pub use actor::DeltaChatActor;
pub use commands::DeltaChatCommand;
pub use events::{ChatInfo, ContactInfo, DeltaChatEvent, MessageInfo};
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p mobile`  
Expected: SUCCESS

- [ ] **Step 5: Commit**

```bash
git add mobile/src/viewmodel/deltachat/
git commit -m "feat: add DeltaChat actor with runtime bridge

Create DeltaChatActor with smol↔tokio runtime bridge
Add channel-based command/event communication
Skeleton run() loop with command reception

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Wire Actor into ViewModel

**Files:**
- Modify: `mobile/src/viewmodel/mod.rs`

**Interfaces:**
- Consumes: `DeltaChatActor` (from Task 3), `DeltaChatCommand`, `DeltaChatEvent` (from Task 2)
- Produces: `deltachat_tx` channel in ViewModel, DeltaChat actor spawned

- [ ] **Step 1: Add deltachat module to viewmodel**

Update `mobile/src/viewmodel/mod.rs` - add after existing `pub mod` declarations:

```rust
pub mod deltachat;
```

- [ ] **Step 2: Add ViewModelEvent::DeltaChat variant**

Find the `ViewModelEvent` enum in `mobile/src/viewmodel/common.rs` and add:

```rust
DeltaChat(deltachat::DeltaChatEvent),
```

Import at top of file:
```rust
use crate::viewmodel::deltachat;
```

- [ ] **Step 3: Add deltachat_tx to ViewModel struct**

Find the `ViewModel` struct in `mobile/src/viewmodel/mod.rs` and add:

```rust
deltachat_tx: Sender<deltachat::DeltaChatCommand>,
```

- [ ] **Step 4: Spawn DeltaChat actor in ViewModel::new()**

In `ViewModel::new()` method (GUI version), add after other actor spawns:

```rust
// DeltaChat actor
let (deltachat_tx, deltachat_rx) = smol::channel::unbounded();
let db_path = std::env::var("HOME")
    .map(|h| std::path::PathBuf::from(h).join(".local/share/dure/deltachat-default.db"))
    .unwrap_or_else(|_| std::path::PathBuf::from("deltachat-default.db"));
let deltachat_actor = deltachat::DeltaChatActor::new(deltachat_rx, event_tx.clone(), db_path);
smol::spawn(deltachat_actor.run()).detach();
```

Add `deltachat_tx` to the returned struct.

For headless version in `ViewModel::new_headless()`, add the same code.

- [ ] **Step 5: Verify compilation**

Run: `cargo check -p mobile`  
Expected: SUCCESS

- [ ] **Step 6: Commit**

```bash
git add mobile/src/viewmodel/
git commit -m "feat: wire DeltaChat actor into ViewModel

Add deltachat module to viewmodel
Add DeltaChat variant to ViewModelEvent enum
Spawn DeltaChat actor with channel communication
Set default database path

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Add DeltaChat Tab to UI

**Files:**
- Create: `mobile/src/ui_tabs/deltachat.rs`
- Modify: `mobile/src/ui_tabs/mod.rs`
- Modify: `mobile/src/dure.rs`

**Interfaces:**
- Consumes: `ViewModel` with `deltachat_tx`
- Produces: DeltaChat tab visible in UI

- [ ] **Step 1: Create placeholder DeltaChat tab**

Create `mobile/src/ui_tabs/deltachat.rs`:

```rust
//! DeltaChat tab - Encrypted messaging

use eframe::egui;
use crate::viewmodel::deltachat::{ChatInfo, ContactInfo, MessageInfo};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DeltaChatTab {
    is_configured: bool,
    is_connected: bool,
    configured_email: Option<String>,
    
    #[cfg_attr(feature = "serde", serde(skip))]
    config_dialog_open: bool,
    config_email: String,
    config_password: String,
    config_in_progress: bool,
    config_progress: i32,
    config_error: Option<String>,
    
    #[cfg_attr(feature = "serde", serde(skip))]
    add_contact_dialog_open: bool,
    add_contact_email: String,
    contacts: Vec<ContactInfo>,
    
    chats: Vec<ChatInfo>,
    selected_chat_id: Option<u32>,
    
    messages: Vec<MessageInfo>,
    compose_text: String,
    
    #[cfg_attr(feature = "serde", serde(skip))]
    last_fetch: Option<std::time::Instant>,
}

impl Default for DeltaChatTab {
    fn default() -> Self {
        Self {
            is_configured: false,
            is_connected: false,
            configured_email: None,
            config_dialog_open: false,
            config_email: String::new(),
            config_password: String::new(),
            config_in_progress: false,
            config_progress: 0,
            config_error: None,
            add_contact_dialog_open: false,
            add_contact_email: String::new(),
            contacts: Vec::new(),
            chats: Vec::new(),
            selected_chat_id: None,
            messages: Vec::new(),
            compose_text: String::new(),
            last_fetch: None,
        }
    }
}

impl DeltaChatTab {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("DeltaChat - Encrypted Messaging");
        
        if !self.is_configured {
            ui.vertical_centered(|ui| {
                ui.label("Configure your email account to start using encrypted messaging.");
                
                if ui.button("Configure Account").clicked() {
                    self.config_dialog_open = true;
                }
            });
        } else {
            ui.label(format!("Account: {}", 
                self.configured_email.as_deref().unwrap_or("Unknown")));
            ui.label(format!("Status: {}", 
                if self.is_connected { "Connected" } else { "Disconnected" }));
        }
    }
}
```

- [ ] **Step 2: Add deltachat module to ui_tabs**

Update `mobile/src/ui_tabs/mod.rs` - add after existing `pub mod` declarations:

```rust
pub mod deltachat;
```

- [ ] **Step 3: Add DeltaChat variant to Tab enum**

In `mobile/src/ui_tabs/mod.rs`, add to `Tab` enum:

```rust
DeltaChat,
```

Update `Tab::name()` method:

```rust
Tab::DeltaChat => "DeltaChat",
```

Update `Tab::all()` method:

```rust
Tab::DeltaChat,
```

- [ ] **Step 4: Add DeltaChat tab to DureApp**

In `mobile/src/dure.rs`, find the struct and add field:

```rust
pub deltachat_tab: crate::ui_tabs::deltachat::DeltaChatTab,
```

In `Default` impl, add:

```rust
deltachat_tab: Default::default(),
```

- [ ] **Step 5: Wire tab rendering**

In `mobile/src/dure.rs`, find where tabs are rendered (search for `Tab::Email =>`), and add:

```rust
Tab::DeltaChat => {
    self.deltachat_tab.ui(ui);
}
```

- [ ] **Step 6: Add tab to TabBar**

Find where `.tab(tr!("tab-email"))` is called and add after it:

```rust
.tab("DeltaChat")
```

Do this for both desktop and mobile (Android/WASM) tab bars.

- [ ] **Step 7: Verify UI appears**

Run: `cargo run -p mobile` (if on desktop)  
Expected: DeltaChat tab appears, shows "Configure Account" button

- [ ] **Step 8: Commit**

```bash
git add mobile/src/ui_tabs/ mobile/src/dure.rs
git commit -m "feat: add DeltaChat tab to UI

Add DeltaChat tab with placeholder UI
Add Tab::DeltaChat enum variant
Wire tab into DureApp rendering
Show configure prompt when not configured

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Implement Configure Command Handler

**Files:**
- Modify: `mobile/src/viewmodel/deltachat/actor.rs`

**Interfaces:**
- Consumes: `DeltaChatCommand::Configure`
- Produces: `DeltaChatEvent::Configured` or `ConfigurationFailed`

- [ ] **Step 1: Write integration test for context initialization**

Add to `mobile/src/viewmodel/deltachat/actor.rs` in `#[cfg(test)]` section:

```rust
#[tokio::test]
async fn test_initialize_context() {
    use tempfile::TempDir;
    
    let tmpdir = TempDir::new().unwrap();
    let db_path = tmpdir.path().join("test.db");
    
    let context = deltachat::ContextBuilder::new(db_path.clone())
        .with_id(1)
        .open()
        .await
        .unwrap();
    
    assert!(context.is_open().await);
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test -p mobile test_initialize_context`  
Expected: 1 test PASS

- [ ] **Step 3: Implement initialize_context method**

Add to `DeltaChatActor` impl block in `mobile/src/viewmodel/deltachat/actor.rs`:

```rust
async fn initialize_context(&mut self) -> Result<(), String> {
    let db_path = self.database_path.clone();
    
    // Ensure database directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create database directory: {}", e))?;
    }
    
    // Open/create database in tokio runtime
    let context = self.tokio_runtime.block_on(async {
        deltachat::ContextBuilder::new(db_path)
            .with_id(1)
            .open()
            .await
            .map_err(|e| format!("Cannot open DeltaChat database: {}", e))
    })?;
    
    self.context = Some(context);
    log::info!("DeltaChat context initialized");
    
    Ok(())
}
```

- [ ] **Step 4: Implement configure_internal method**

Add to `DeltaChatActor` impl:

```rust
async fn configure_internal(&mut self, email: &str, password: &str) -> Result<(), String> {
    // Initialize context if not already done
    if self.context.is_none() {
        self.initialize_context().await?;
    }
    
    let context = self.context.as_ref().ok_or("Context not initialized")?;
    
    self.tokio_runtime.block_on(async {
        use deltachat::config::Config;
        
        // Set configuration
        context.set_config(Config::Addr, Some(email))
            .await
            .map_err(|e| format!("Failed to set email: {}", e))?;
        
        context.set_config(Config::MailPw, Some(password))
            .await
            .map_err(|e| format!("Failed to set password: {}", e))?;
        
        // Run configuration
        context.configure()
            .await
            .map_err(|e| format!("Configuration failed: {}", e))?;
        
        Ok::<(), String>(())
    })
}
```

- [ ] **Step 5: Implement handle_command with Configure case**

Replace the TODO in `run()` method with:

```rust
let result = smol::unblock({
    let rt = &self.tokio_runtime;
    let mut actor = &mut self;
    move || {
        rt.block_on(async {
            actor.handle_command(cmd).await
        })
    }
}).await;
```

Add `handle_command` method:

```rust
async fn handle_command(&mut self, cmd: DeltaChatCommand) -> Result<(), String> {
    match cmd {
        DeltaChatCommand::Configure { email, password } => {
            self.emit_event(DeltaChatEvent::ConfigurationStarted);
            
            match self.configure_internal(&email, &password).await {
                Ok(_) => {
                    self.is_configured = true;
                    self.emit_event(DeltaChatEvent::Configured { 
                        email: email.clone()
                    });
                    Ok(())
                }
                Err(e) => {
                    let error_msg = if e.contains("authentication") || e.contains("login") {
                        "Invalid email or password. Please check your credentials.".to_string()
                    } else if e.contains("network") || e.contains("connection") {
                        "Cannot reach email server. Check your internet connection.".to_string()
                    } else {
                        format!("Configuration failed: {}", e)
                    };
                    
                    self.emit_event(DeltaChatEvent::ConfigurationFailed { 
                        error: error_msg.clone()
                    });
                    Err(error_msg)
                }
            }
        }
        _ => {
            log::warn!("Command not yet implemented: {:?}", cmd);
            Ok(())
        }
    }
}
```

- [ ] **Step 6: Fix compilation errors**

The `smol::unblock` closure needs to capture self differently. Update the `run()` method:

```rust
pub async fn run(mut self) {
    log::info!("DeltaChatActor started");
    
    while let Ok(cmd) = self.command_rx.recv().await {
        log::debug!("DeltaChatActor received command: {:?}", cmd);
        
        // Handle command (moves into tokio runtime)
        if let Err(e) = self.handle_command(cmd).await {
            log::error!("DeltaChat command failed: {}", e);
        }
    }
    
    log::info!("DeltaChatActor stopped");
}

async fn handle_command(&mut self, cmd: DeltaChatCommand) -> Result<(), String> {
    // Wrap tokio calls with runtime
    let result = match cmd {
        DeltaChatCommand::Configure { email, password } => {
            self.emit_event(DeltaChatEvent::ConfigurationStarted);
            self.configure_internal(&email, &password).await
        }
        _ => {
            log::warn!("Command not yet implemented: {:?}", cmd);
            Ok(())
        }
    };
    
    match result {
        Ok(_) if matches!(cmd, DeltaChatCommand::Configure { .. }) => {
            if let DeltaChatCommand::Configure { email, .. } = cmd {
                self.is_configured = true;
                self.emit_event(DeltaChatEvent::Configured { email });
            }
            Ok(())
        }
        Err(e) => {
            let error_msg = if e.contains("authentication") || e.contains("login") {
                "Invalid email or password.".to_string()
            } else if e.contains("network") || e.contains("connection") {
                "Cannot reach email server.".to_string()
            } else {
                format!("Configuration failed: {}", e)
            };
            
            self.emit_event(DeltaChatEvent::ConfigurationFailed { 
                error: error_msg.clone()
            });
            Err(error_msg)
        }
        Ok(_) => Ok(()),
    }
}
```

- [ ] **Step 7: Verify compilation**

Run: `cargo check -p mobile`  
Expected: SUCCESS

- [ ] **Step 8: Commit**

```bash
git add mobile/src/viewmodel/deltachat/actor.rs
git commit -m "feat: implement Configure command handler

Add initialize_context() to open DeltaChat database
Add configure_internal() to set email/password
Add handle_command() with Configure case
Emit Configured or ConfigurationFailed events

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Implement Configuration Dialog UI

**Files:**
- Modify: `mobile/src/ui_tabs/deltachat.rs`

**Interfaces:**
- Consumes: `DeltaChatEvent::Configured`, `ConfigurationFailed`, `ConfigurationProgress`
- Produces: Configuration dialog UI, sends `DeltaChatCommand::Configure`

- [ ] **Step 1: Add ViewModel parameter to ui() method**

Update `mobile/src/ui_tabs/deltachat.rs`:

```rust
use crate::viewmodel::ViewModel;

impl DeltaChatTab {
    pub fn ui(&mut self, ui: &mut egui::Ui, vm: &ViewModel) {
        // Poll for events
        self.handle_events(vm);
        
        // Show configuration dialog
        if self.config_dialog_open {
            self.render_config_dialog(ui, vm);
        }
        
        ui.heading("DeltaChat - Encrypted Messaging");
        
        if !self.is_configured {
            ui.vertical_centered(|ui| {
                ui.label("Configure your email account to start using encrypted messaging.");
                
                if ui.button("Configure Account").clicked() {
                    self.config_dialog_open = true;
                }
            });
        } else {
            ui.label(format!("Account: {}", 
                self.configured_email.as_deref().unwrap_or("Unknown")));
            ui.label(format!("Status: {}", 
                if self.is_connected { "Connected" } else { "Disconnected" }));
        }
    }
    
    fn handle_events(&mut self, vm: &ViewModel) {
        use crate::viewmodel::common::ViewModelEvent;
        use crate::viewmodel::deltachat::DeltaChatEvent;
        
        while let Ok(event) = vm.event_rx.try_recv() {
            match event {
                ViewModelEvent::DeltaChat(dc_event) => match dc_event {
                    DeltaChatEvent::ConfigurationFailed { error } => {
                        self.config_in_progress = false;
                        self.config_error = Some(error);
                    }
                    
                    DeltaChatEvent::Configured { email } => {
                        self.is_configured = true;
                        self.configured_email = Some(email);
                        self.config_dialog_open = false;
                        self.config_error = None;
                        self.config_in_progress = false;
                    }
                    
                    DeltaChatEvent::ConfigurationProgress { progress, .. } => {
                        self.config_progress = progress;
                    }
                    
                    _ => {}
                }
                _ => {}
            }
        }
    }
    
    fn render_config_dialog(&mut self, ui: &mut egui::Ui, vm: &ViewModel) {
        use crate::viewmodel::deltachat::DeltaChatCommand;
        
        egui::Window::new("Configure DeltaChat")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("Email Address:");
                    ui.text_edit_singleline(&mut self.config_email);
                    
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.config_password)
                        .password(true));
                    
                    if let Some(error) = &self.config_error {
                        ui.colored_label(egui::Color32::RED, error);
                    }
                    
                    if self.config_in_progress {
                        let progress = self.config_progress as f32 / 1000.0;
                        ui.add(egui::ProgressBar::new(progress).show_percentage());
                        ui.label("Configuring...");
                    }
                    
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() && !self.config_in_progress {
                            self.config_dialog_open = false;
                            self.config_error = None;
                        }
                        
                        let can_submit = !self.config_email.is_empty() 
                                      && !self.config_password.is_empty()
                                      && !self.config_in_progress;
                        
                        if ui.add_enabled(can_submit, egui::Button::new("Configure"))
                            .clicked() 
                        {
                            self.config_in_progress = true;
                            self.config_error = None;
                            self.config_progress = 0;
                            
                            let cmd = DeltaChatCommand::Configure {
                                email: self.config_email.clone(),
                                password: self.config_password.clone(),
                            };
                            
                            smol::block_on(async {
                                let _ = vm.deltachat_tx.send(cmd).await;
                            });
                        }
                    });
                });
            });
    }
}
```

- [ ] **Step 2: Update dure.rs to pass ViewModel**

In `mobile/src/dure.rs`, update the DeltaChat tab rendering:

```rust
Tab::DeltaChat => {
    if let Some(vm) = &self.viewmodel {
        self.deltachat_tab.ui(ui, vm);
    }
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p mobile`  
Expected: SUCCESS

- [ ] **Step 4: Manual test (if possible)**

Run: `cargo run -p mobile`  
- Click "Configure Account" → Dialog opens
- Enter email/password → Click Configure
- Verify progress bar appears
- Verify success/failure handling

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_tabs/deltachat.rs mobile/src/dure.rs
git commit -m "feat: implement configuration dialog UI

Add handle_events() to process actor events
Add render_config_dialog() with email/password fields
Add progress bar for configuration progress
Add error display for configuration failures
Send Configure command to actor on submit

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Implement Connect/Disconnect Commands

**Files:**
- Modify: `mobile/src/viewmodel/deltachat/actor.rs`
- Modify: `mobile/src/ui_tabs/deltachat.rs`

**Interfaces:**
- Consumes: `DeltaChatCommand::Connect`, `Disconnect`
- Produces: `DeltaChatEvent::Connected`, `Disconnected`

- [ ] **Step 1: Implement Connect command handler**

Add to `handle_command()` in `mobile/src/viewmodel/deltachat/actor.rs`:

```rust
DeltaChatCommand::Connect => {
    if let Some(context) = &self.context {
        self.tokio_runtime.block_on(async {
            context.start_io().await;
        });
        self.is_connected = true;
        self.emit_event(DeltaChatEvent::Connected);
        log::info!("DeltaChat connected");
    } else {
        let error = "Cannot connect: not configured".to_string();
        self.emit_event(DeltaChatEvent::Error {
            operation: "connect".to_string(),
            error: error.clone(),
        });
        return Err(error);
    }
    Ok(())
}

DeltaChatCommand::Disconnect => {
    if let Some(context) = &self.context {
        self.tokio_runtime.block_on(async {
            context.stop_io().await;
        });
        self.is_connected = false;
        self.emit_event(DeltaChatEvent::Disconnected);
        log::info!("DeltaChat disconnected");
    }
    Ok(())
}

DeltaChatCommand::GetConnectionStatus => {
    self.emit_event(DeltaChatEvent::ConnectionStatus {
        connected: self.is_connected,
        email: if self.is_configured {
            // Get email from context config
            if let Some(context) = &self.context {
                self.tokio_runtime.block_on(async {
                    use deltachat::config::Config;
                    context.get_config(Config::Addr).await.ok().flatten()
                })
            } else {
                None
            }
        } else {
            None
        },
    });
    Ok(())
}
```

- [ ] **Step 2: Auto-connect after configuration**

Update the `Configure` case to auto-connect:

```rust
DeltaChatCommand::Configure { email, password } => {
    self.emit_event(DeltaChatEvent::ConfigurationStarted);
    
    match self.configure_internal(&email, &password).await {
        Ok(_) => {
            self.is_configured = true;
            self.emit_event(DeltaChatEvent::Configured { 
                email: email.clone()
            });
            
            // Auto-connect after successful configuration
            if let Some(context) = &self.context {
                self.tokio_runtime.block_on(async {
                    context.start_io().await;
                });
                self.is_connected = true;
                self.emit_event(DeltaChatEvent::Connected);
            }
            
            Ok(())
        }
        Err(e) => {
            // ... existing error handling
        }
    }
}
```

- [ ] **Step 3: Handle Connected event in UI**

Update `handle_events()` in `mobile/src/ui_tabs/deltachat.rs`:

```rust
DeltaChatEvent::Connected => {
    self.is_connected = true;
    log::info!("DeltaChat connected");
}

DeltaChatEvent::Disconnected => {
    self.is_connected = false;
    log::info!("DeltaChat disconnected");
}

DeltaChatEvent::ConnectionStatus { connected, email } => {
    self.is_connected = connected;
    if let Some(email) = email {
        self.configured_email = Some(email);
    }
}
```

- [ ] **Step 4: Add disconnect button to UI**

Update main UI in `ui()` method:

```rust
} else {
    ui.label(format!("Account: {}", 
        self.configured_email.as_deref().unwrap_or("Unknown")));
    
    ui.horizontal(|ui| {
        ui.label(format!("Status: {}", 
            if self.is_connected { "Connected" } else { "Disconnected" }));
        
        if self.is_connected {
            if ui.button("Disconnect").clicked() {
                smol::block_on(async {
                    let _ = vm.deltachat_tx.send(DeltaChatCommand::Disconnect).await;
                });
            }
        } else {
            if ui.button("Connect").clicked() {
                smol::block_on(async {
                    let _ = vm.deltachat_tx.send(DeltaChatCommand::Connect).await;
                });
            }
        }
        
        if ui.button("Reconfigure").clicked() {
            self.config_dialog_open = true;
        }
    });
}
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check -p mobile`  
Expected: SUCCESS

- [ ] **Step 6: Commit**

```bash
git add mobile/src/viewmodel/deltachat/ mobile/src/ui_tabs/deltachat.rs
git commit -m "feat: implement connect/disconnect commands

Add Connect/Disconnect/GetConnectionStatus handlers
Auto-connect after successful configuration
Add disconnect button and connection status to UI
Add reconfigure button

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Implement AddContact Command

**Files:**
- Modify: `mobile/src/viewmodel/deltachat/actor.rs`
- Modify: `mobile/src/ui_tabs/deltachat.rs`

**Interfaces:**
- Consumes: `DeltaChatCommand::AddContact`, `ListContacts`
- Produces: `DeltaChatEvent::ContactAdded`, `ContactsListed`

- [ ] **Step 1: Write integration test for add contact**

Add to `actor.rs` tests:

```rust
#[tokio::test]
async fn test_add_contact() {
    use tempfile::TempDir;
    
    let tmpdir = TempDir::new().unwrap();
    let db_path = tmpdir.path().join("test.db");
    
    let context = deltachat::ContextBuilder::new(db_path)
        .with_id(1)
        .open()
        .await
        .unwrap();
    
    let contact_id = deltachat::contact::Contact::create(
        &context,
        "Test User",
        "test@example.com"
    ).await.unwrap();
    
    let contact = deltachat::contact::Contact::get_by_id(&context, contact_id)
        .await
        .unwrap();
    
    assert_eq!(contact.get_addr(), "test@example.com");
    assert_eq!(contact.get_display_name(), "Test User");
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test -p mobile test_add_contact`  
Expected: 1 test PASS

- [ ] **Step 3: Implement AddContact handler**

Add to `handle_command()`:

```rust
DeltaChatCommand::AddContact { email } => {
    if let Some(context) = &self.context {
        let contact_id = self.tokio_runtime.block_on(async {
            deltachat::contact::Contact::create(&context, "", &email)
                .await
                .map_err(|e| format!("Failed to add contact: {}", e))
        })?;
        
        let contact = self.tokio_runtime.block_on(async {
            deltachat::contact::Contact::get_by_id(&context, contact_id)
                .await
                .map_err(|e| format!("Failed to get contact: {}", e))
        })?;
        
        let contact_info = ContactInfo {
            id: contact_id.to_u32(),
            name: contact.get_display_name().to_string(),
            email: contact.get_addr().to_string(),
            is_blocked: contact.is_blocked(),
        };
        
        self.emit_event(DeltaChatEvent::ContactAdded { 
            contact: contact_info 
        });
        log::info!("Contact added: {}", email);
    } else {
        return Err("Cannot add contact: not configured".to_string());
    }
    Ok(())
}

DeltaChatCommand::ListContacts => {
    if let Some(context) = &self.context {
        let contacts = self.tokio_runtime.block_on(async {
            let contact_ids = deltachat::contact::Contact::get_all(
                &context,
                0, // flags
                None, // query
            ).await.map_err(|e| format!("Failed to list contacts: {}", e))?;
            
            let mut contacts = Vec::new();
            for contact_id in contact_ids {
                if let Ok(contact) = deltachat::contact::Contact::get_by_id(&context, contact_id).await {
                    contacts.push(ContactInfo {
                        id: contact_id.to_u32(),
                        name: contact.get_display_name().to_string(),
                        email: contact.get_addr().to_string(),
                        is_blocked: contact.is_blocked(),
                    });
                }
            }
            
            Ok::<Vec<ContactInfo>, String>(contacts)
        })?;
        
        self.emit_event(DeltaChatEvent::ContactsListed { contacts });
        log::info!("Listed {} contacts", contacts.len());
    } else {
        return Err("Cannot list contacts: not configured".to_string());
    }
    Ok(())
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p mobile`  
Expected: SUCCESS

- [ ] **Step 5: Commit**

```bash
git add mobile/src/viewmodel/deltachat/actor.rs
git commit -m "feat: implement AddContact and ListContacts

Add AddContact handler to create contacts
Add ListContacts handler to fetch all contacts
Convert DeltaChat contacts to ContactInfo structs
Emit ContactAdded and ContactsListed events

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

Due to length constraints, I'll continue with the remaining tasks in the same pattern. The plan would continue with:

- Task 10: Contact List UI
- Task 11: CreateChat Command
- Task 12: Chat List UI  
- Task 13: SendTextMessage Command
- Task 14: Message View UI
- Task 15: Auto-Refresh Timer
- Task 16: Polish & Error Handling

Each following the same TDD pattern: test → implement → verify → commit.

Would you like me to continue with the complete plan, or is this structure clear for the remaining tasks?

## Task 10: Contact List UI

**Files:**
- Modify: `mobile/src/ui_tabs/deltachat.rs`

**Interfaces:**
- Consumes: `DeltaChatEvent::ContactAdded`, `ContactsListed`
- Produces: Contact list view with add contact dialog

- [ ] **Step 1: Add contact list rendering**

Update `ui()` method in `mobile/src/ui_tabs/deltachat.rs` after connection status:

```rust
if self.is_connected {
    ui.separator();
    
    ui.horizontal(|ui| {
        ui.heading("Contacts");
        if ui.button("Add Contact").clicked() {
            self.add_contact_dialog_open = true;
        }
    });
    
    self.render_contact_list(ui);
}
```

Add `render_contact_list()` method:

```rust
fn render_contact_list(&mut self, ui: &mut egui::Ui) {
    if self.contacts.is_empty() {
        ui.label("No contacts yet. Add a contact to start messaging.");
    } else {
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for contact in &self.contacts {
                    ui.horizontal(|ui| {
                        ui.label(&contact.name);
                        ui.label(format!("({})", &contact.email));
                    });
                }
            });
    }
}
```

- [ ] **Step 2: Add contact dialog**

Add `render_add_contact_dialog()` method:

```rust
fn render_add_contact_dialog(&mut self, ui: &mut egui::Ui, vm: &ViewModel) {
    use crate::viewmodel::deltachat::DeltaChatCommand;
    
    egui::Window::new("Add Contact")
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.vertical(|ui| {
                ui.label("Contact Email:");
                ui.text_edit_singleline(&mut self.add_contact_email);
                
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.add_contact_dialog_open = false;
                        self.add_contact_email.clear();
                    }
                    
                    let can_submit = !self.add_contact_email.is_empty() 
                                  && self.add_contact_email.contains('@');
                    
                    if ui.add_enabled(can_submit, egui::Button::new("Add"))
                        .clicked() 
                    {
                        let cmd = DeltaChatCommand::AddContact {
                            email: self.add_contact_email.clone(),
                        };
                        
                        smol::block_on(async {
                            let _ = vm.deltachat_tx.send(cmd).await;
                        });
                        
                        self.add_contact_dialog_open = false;
                        self.add_contact_email.clear();
                    }
                });
            });
        });
}
```

- [ ] **Step 3: Show add contact dialog**

Update `ui()` to render dialog:

```rust
if self.add_contact_dialog_open {
    self.render_add_contact_dialog(ui, vm);
}
```

- [ ] **Step 4: Handle ContactAdded event**

Update `handle_events()`:

```rust
DeltaChatEvent::ContactAdded { contact } => {
    self.contacts.push(contact);
    // Refresh full list
    smol::block_on(async {
        let _ = vm.deltachat_tx.send(DeltaChatCommand::ListContacts).await;
    });
}

DeltaChatEvent::ContactsListed { contacts } => {
    self.contacts = contacts;
}
```

- [ ] **Step 5: Request contacts on connect**

Update `Connected` event handler:

```rust
DeltaChatEvent::Connected => {
    self.is_connected = true;
    // Fetch initial data
    smol::block_on(async {
        let _ = vm.deltachat_tx.send(DeltaChatCommand::ListContacts).await;
    });
}
```

- [ ] **Step 6: Verify compilation**

Run: `cargo check -p mobile`  
Expected: SUCCESS

- [ ] **Step 7: Commit**

```bash
git add mobile/src/ui_tabs/deltachat.rs
git commit -m "feat: add contact list UI

Add render_contact_list() to display contacts
Add render_add_contact_dialog() for adding contacts
Handle ContactAdded and ContactsListed events
Fetch contacts on connection

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Implement CreateChat and ListChats Commands

**Files:**
- Modify: `mobile/src/viewmodel/deltachat/actor.rs`

**Interfaces:**
- Consumes: `DeltaChatCommand::CreateChat`, `ListChats`, `SelectChat`
- Produces: `DeltaChatEvent::ChatCreated`, `ChatsListed`, `ChatSelected`

- [ ] **Step 1: Write integration test**

Add to `actor.rs` tests:

```rust
#[tokio::test]
async fn test_create_chat() {
    use tempfile::TempDir;
    
    let tmpdir = TempDir::new().unwrap();
    let db_path = tmpdir.path().join("test.db");
    
    let context = deltachat::ContextBuilder::new(db_path)
        .with_id(1)
        .open()
        .await
        .unwrap();
    
    let contact_id = deltachat::contact::Contact::create(
        &context,
        "Chat Partner",
        "partner@example.com"
    ).await.unwrap();
    
    let chat_id = deltachat::chat::create_by_contact_id(&context, contact_id)
        .await
        .unwrap();
    
    let chat = deltachat::chat::Chat::load_from_db(&context, chat_id)
        .await
        .unwrap();
    
    assert_eq!(chat.get_type(), deltachat::chat::Chattype::Single);
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test -p mobile test_create_chat`  
Expected: 1 test PASS

- [ ] **Step 3: Implement CreateChat handler**

Add to `handle_command()`:

```rust
DeltaChatCommand::CreateChat { contact_id } => {
    if let Some(context) = &self.context {
        let chat_id = self.tokio_runtime.block_on(async {
            deltachat::chat::create_by_contact_id(&context, contact_id.into())
                .await
                .map_err(|e| format!("Failed to create chat: {}", e))
        })?;
        
        let chat = self.tokio_runtime.block_on(async {
            deltachat::chat::Chat::load_from_db(&context, chat_id)
                .await
                .map_err(|e| format!("Failed to load chat: {}", e))
        })?;
        
        let chat_info = ChatInfo {
            id: chat_id.to_u32(),
            name: chat.get_name().to_string(),
            last_message: None,
            unread_count: 0,
            timestamp: 0,
        };
        
        self.emit_event(DeltaChatEvent::ChatCreated { 
            chat: chat_info 
        });
        log::info!("Chat created: {}", chat_id.to_u32());
    } else {
        return Err("Cannot create chat: not configured".to_string());
    }
    Ok(())
}

DeltaChatCommand::ListChats => {
    if let Some(context) = &self.context {
        let chats = self.tokio_runtime.block_on(async {
            use deltachat::chatlist::Chatlist;
            
            let chatlist = Chatlist::try_load(&context, 0, None, None)
                .await
                .map_err(|e| format!("Failed to list chats: {}", e))?;
            
            let mut chats = Vec::new();
            for i in 0..chatlist.len() {
                if let Some(chat_id) = chatlist.get_chat_id(i) {
                    if let Ok(chat) = deltachat::chat::Chat::load_from_db(&context, chat_id).await {
                        let msg_count = context.get_msg_cnt(chat_id).await.unwrap_or(0);
                        let fresh_msg_cnt = context.get_fresh_msg_cnt(chat_id).await.unwrap_or(0);
                        
                        chats.push(ChatInfo {
                            id: chat_id.to_u32(),
                            name: chat.get_name().to_string(),
                            last_message: None, // TODO: get last message
                            unread_count: fresh_msg_cnt as u32,
                            timestamp: 0, // TODO: get last message timestamp
                        });
                    }
                }
            }
            
            Ok::<Vec<ChatInfo>, String>(chats)
        })?;
        
        self.emit_event(DeltaChatEvent::ChatsListed { chats });
        log::info!("Listed {} chats", chats.len());
    } else {
        return Err("Cannot list chats: not configured".to_string());
    }
    Ok(())
}

DeltaChatCommand::SelectChat { chat_id } => {
    self.current_chat_id = Some(chat_id);
    self.emit_event(DeltaChatEvent::ChatSelected { chat_id });
    Ok(())
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p mobile`  
Expected: SUCCESS

- [ ] **Step 5: Commit**

```bash
git add mobile/src/viewmodel/deltachat/actor.rs
git commit -m "feat: implement CreateChat and ListChats

Add CreateChat handler to create 1:1 chats
Add ListChats handler to fetch all chats
Add SelectChat handler to track selected chat
Emit ChatCreated, ChatsListed, ChatSelected events

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 12: Chat List UI

**Files:**
- Modify: `mobile/src/ui_tabs/deltachat.rs`

**Interfaces:**
- Consumes: `DeltaChatEvent::ChatCreated`, `ChatsListed`, `ChatSelected`
- Produces: Chat list with create chat from contacts

- [ ] **Step 1: Add chat list rendering**

Update `ui()` to add chat section:

```rust
if self.is_connected {
    ui.separator();
    
    ui.heading("Chats");
    self.render_chat_list(ui, vm);
}
```

Add `render_chat_list()` method:

```rust
fn render_chat_list(&mut self, ui: &mut egui::Ui, vm: &ViewModel) {
    use crate::viewmodel::deltachat::DeltaChatCommand;
    
    if self.chats.is_empty() {
        ui.label("No chats yet. Create a chat with a contact to start messaging.");
        
        // Show contacts to create chat with
        if !self.contacts.is_empty() {
            ui.label("Select a contact to chat with:");
            for contact in &self.contacts {
                if ui.button(format!("Chat with {}", &contact.name)).clicked() {
                    let cmd = DeltaChatCommand::CreateChat {
                        contact_id: contact.id,
                    };
                    smol::block_on(async {
                        let _ = vm.deltachat_tx.send(cmd).await;
                    });
                }
            }
        }
    } else {
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for chat in &self.chats {
                    let is_selected = self.selected_chat_id == Some(chat.id);
                    
                    let response = ui.selectable_label(
                        is_selected,
                        format!("{} ({})", &chat.name, chat.unread_count)
                    );
                    
                    if response.clicked() {
                        let cmd = DeltaChatCommand::SelectChat {
                            chat_id: chat.id,
                        };
                        smol::block_on(async {
                            let _ = vm.deltachat_tx.send(cmd).await;
                        });
                    }
                }
            });
    }
}
```

- [ ] **Step 2: Handle chat events**

Update `handle_events()`:

```rust
DeltaChatEvent::ChatCreated { chat } => {
    self.chats.push(chat.clone());
    // Select newly created chat
    self.selected_chat_id = Some(chat.id);
    // Refresh chat list
    smol::block_on(async {
        let _ = vm.deltachat_tx.send(DeltaChatCommand::ListChats).await;
    });
}

DeltaChatEvent::ChatsListed { chats } => {
    self.chats = chats;
}

DeltaChatEvent::ChatSelected { chat_id } => {
    self.selected_chat_id = Some(chat_id);
    // Fetch messages for selected chat
    smol::block_on(async {
        let _ = vm.deltachat_tx.send(
            DeltaChatCommand::ListMessages { chat_id }
        ).await;
    });
}
```

- [ ] **Step 3: Fetch chats on connect**

Update `Connected` handler:

```rust
DeltaChatEvent::Connected => {
    self.is_connected = true;
    smol::block_on(async {
        let _ = vm.deltachat_tx.send(DeltaChatCommand::ListContacts).await;
        let _ = vm.deltachat_tx.send(DeltaChatCommand::ListChats).await;
    });
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p mobile`  
Expected: SUCCESS

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_tabs/deltachat.rs
git commit -m "feat: add chat list UI

Add render_chat_list() to display chats
Allow creating chats from contact list
Handle ChatCreated, ChatsListed, ChatSelected events
Fetch messages when chat is selected

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 13: Implement SendTextMessage and ListMessages Commands

**Files:**
- Modify: `mobile/src/viewmodel/deltachat/actor.rs`

**Interfaces:**
- Consumes: `DeltaChatCommand::SendTextMessage`, `ListMessages`, `MarkMessagesSeen`
- Produces: `DeltaChatEvent::MessageSent`, `MessagesListed`, `MessagesSeen`

- [ ] **Step 1: Implement SendTextMessage handler**

Add to `handle_command()`:

```rust
DeltaChatCommand::SendTextMessage { chat_id, text } => {
    if let Some(context) = &self.context {
        let msg_id = self.tokio_runtime.block_on(async {
            deltachat::chat::send_text_msg(&context, chat_id.into(), &text)
                .await
                .map_err(|e| format!("Failed to send message: {}", e))
        })?;
        
        self.emit_event(DeltaChatEvent::MessageSent { 
            msg_id: msg_id.to_u32(),
            chat_id,
        });
        log::info!("Message sent: {} to chat {}", msg_id.to_u32(), chat_id);
    } else {
        return Err("Cannot send message: not configured".to_string());
    }
    Ok(())
}

DeltaChatCommand::ListMessages { chat_id } => {
    if let Some(context) = &self.context {
        let messages = self.tokio_runtime.block_on(async {
            use deltachat::message::Message;
            
            let msg_ids = deltachat::chat::get_chat_msgs(&context, chat_id.into())
                .await
                .map_err(|e| format!("Failed to list messages: {}", e))?;
            
            let mut messages = Vec::new();
            for msg_id in msg_ids {
                if let Ok(msg) = Message::load_from_db(&context, msg_id).await {
                    let from_id = msg.get_from_id();
                    let from_name = if let Ok(contact) = deltachat::contact::Contact::get_by_id(&context, from_id).await {
                        contact.get_display_name().to_string()
                    } else {
                        "Unknown".to_string()
                    };
                    
                    messages.push(MessageInfo {
                        id: msg_id.to_u32(),
                        from_contact_id: from_id.to_u32(),
                        from_name,
                        text: msg.get_text(),
                        timestamp: msg.get_timestamp(),
                        is_outgoing: msg.is_outgoing(),
                        is_seen: msg.get_state() == deltachat::message::MessageState::InSeen,
                    });
                }
            }
            
            Ok::<Vec<MessageInfo>, String>(messages)
        })?;
        
        self.emit_event(DeltaChatEvent::MessagesListed { 
            chat_id,
            messages,
        });
    } else {
        return Err("Cannot list messages: not configured".to_string());
    }
    Ok(())
}

DeltaChatCommand::MarkMessagesSeen { chat_id } => {
    if let Some(context) = &self.context {
        self.tokio_runtime.block_on(async {
            deltachat::chat::marknoticed_chat(&context, chat_id.into())
                .await
                .map_err(|e| format!("Failed to mark seen: {}", e))
        })?;
        
        self.emit_event(DeltaChatEvent::MessagesSeen { chat_id });
    }
    Ok(())
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p mobile`  
Expected: SUCCESS

- [ ] **Step 3: Commit**

```bash
git add mobile/src/viewmodel/deltachat/actor.rs
git commit -m "feat: implement messaging commands

Add SendTextMessage handler to send messages
Add ListMessages handler to fetch messages
Add MarkMessagesSeen handler to mark as read
Emit MessageSent, MessagesListed, MessagesSeen events

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 14: Message View UI

**Files:**
- Modify: `mobile/src/ui_tabs/deltachat.rs`

**Interfaces:**
- Consumes: `DeltaChatEvent::MessageSent`, `MessagesListed`, `NewMessageReceived`
- Produces: Message view with compose field

- [ ] **Step 1: Add message view rendering**

Update `ui()` to show messages when chat is selected:

```rust
if let Some(chat_id) = self.selected_chat_id {
    ui.separator();
    ui.heading("Messages");
    self.render_message_view(ui, vm, chat_id);
}
```

Add `render_message_view()` method:

```rust
fn render_message_view(&mut self, ui: &mut egui::Ui, vm: &ViewModel, chat_id: u32) {
    use crate::viewmodel::deltachat::DeltaChatCommand;
    
    // Message list
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if self.messages.is_empty() {
                ui.label("No messages yet. Start the conversation!");
            } else {
                for msg in &self.messages {
                    ui.horizontal(|ui| {
                        if msg.is_outgoing {
                            ui.label("You:");
                        } else {
                            ui.label(format!("{}:", &msg.from_name));
                        }
                        ui.label(&msg.text);
                    });
                }
            }
        });
    
    ui.separator();
    
    // Compose field
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut self.compose_text);
        
        let can_send = !self.compose_text.is_empty();
        
        if ui.add_enabled(can_send, egui::Button::new("Send")).clicked() 
            || (ui.input(|i| i.key_pressed(egui::Key::Enter)) && can_send)
        {
            let cmd = DeltaChatCommand::SendTextMessage {
                chat_id,
                text: self.compose_text.clone(),
            };
            
            smol::block_on(async {
                let _ = vm.deltachat_tx.send(cmd).await;
            });
            
            self.compose_text.clear();
        }
    });
}
```

- [ ] **Step 2: Handle message events**

Update `handle_events()`:

```rust
DeltaChatEvent::MessageSent { msg_id, chat_id } => {
    // Refresh message list for this chat
    if self.selected_chat_id == Some(chat_id) {
        smol::block_on(async {
            let _ = vm.deltachat_tx.send(
                DeltaChatCommand::ListMessages { chat_id }
            ).await;
        });
    }
}

DeltaChatEvent::MessagesListed { chat_id, messages } => {
    if self.selected_chat_id == Some(chat_id) {
        self.messages = messages;
        // Mark as seen
        smol::block_on(async {
            let _ = vm.deltachat_tx.send(
                DeltaChatCommand::MarkMessagesSeen { chat_id }
            ).await;
        });
    }
}

DeltaChatEvent::NewMessageReceived { chat_id, message } => {
    if self.selected_chat_id == Some(chat_id) {
        self.messages.push(message);
    }
    // Update unread count
    if let Some(chat) = self.chats.iter_mut().find(|c| c.id == chat_id) {
        chat.unread_count += 1;
    }
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p mobile`  
Expected: SUCCESS

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_tabs/deltachat.rs
git commit -m "feat: add message view UI

Add render_message_view() to display messages
Add compose field with Enter key support
Handle MessageSent, MessagesListed events
Update UI when new messages arrive
Mark messages as seen when viewing

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 15: Implement Auto-Refresh and Event Listener

**Files:**
- Modify: `mobile/src/viewmodel/deltachat/actor.rs`
- Modify: `mobile/src/ui_tabs/deltachat.rs`

**Interfaces:**
- Consumes: `DeltaChatCommand::FetchMessages`
- Produces: `DeltaChatEvent::NewMessageReceived` from DeltaChat events

- [ ] **Step 1: Implement event listener in actor**

Add to `DeltaChatActor`:

```rust
async fn listen_to_deltachat_events(&self) {
    let context = match &self.context {
        Some(ctx) => ctx.clone(),
        None => return,
    };
    let event_tx = self.event_tx.clone();
    
    self.tokio_runtime.spawn(async move {
        let mut events = context.get_event_emitter();
        
        while let Some(event) = events.recv().await {
            match event.typ {
                deltachat::EventType::IncomingMsg { chat_id, msg_id } => {
                    if let Ok(msg) = deltachat::message::Message::load_from_db(&context, msg_id).await {
                        let from_id = msg.get_from_id();
                        let from_name = if let Ok(contact) = deltachat::contact::Contact::get_by_id(&context, from_id).await {
                            contact.get_display_name().to_string()
                        } else {
                            "Unknown".to_string()
                        };
                        
                        let message_info = MessageInfo {
                            id: msg_id.to_u32(),
                            from_contact_id: from_id.to_u32(),
                            from_name,
                            text: msg.get_text(),
                            timestamp: msg.get_timestamp(),
                            is_outgoing: false,
                            is_seen: false,
                        };
                        
                        let _ = event_tx.send(ViewModelEvent::DeltaChat(
                            DeltaChatEvent::NewMessageReceived {
                                chat_id: chat_id.to_u32(),
                                message: message_info,
                            }
                        )).await;
                    }
                }
                
                deltachat::EventType::ConfigureProgress { progress, comment } => {
                    let _ = event_tx.send(ViewModelEvent::DeltaChat(
                        DeltaChatEvent::ConfigurationProgress { progress, comment }
                    )).await;
                }
                
                _ => {}
            }
        }
    });
}
```

- [ ] **Step 2: Call event listener after configuration**

Update `configure_internal()` to start event listener:

```rust
async fn configure_internal(&mut self, email: &str, password: &str) -> Result<(), String> {
    // ... existing code ...
    
    self.context = Some(context);
    log::info!("DeltaChat context initialized");
    
    // Start event listener
    self.listen_to_deltachat_events().await;
    
    Ok(())
}
```

- [ ] **Step 3: Implement FetchMessages handler**

Add to `handle_command()`:

```rust
DeltaChatCommand::FetchMessages => {
    if let Some(context) = &self.context {
        self.tokio_runtime.block_on(async {
            context.background_fetch().await
                .map_err(|e| format!("Failed to fetch messages: {}", e))
        })?;
        log::debug!("Background fetch completed");
    }
    Ok(())
}
```

- [ ] **Step 4: Add auto-refresh to UI**

Add to `DeltaChatTab`:

```rust
fn check_auto_refresh(&mut self, vm: &ViewModel) {
    use crate::viewmodel::deltachat::DeltaChatCommand;
    use std::time::Duration;
    
    const REFRESH_INTERVAL: Duration = Duration::from_secs(30);
    
    let should_fetch = self.last_fetch
        .map(|t| t.elapsed() > REFRESH_INTERVAL)
        .unwrap_or(true);
    
    if should_fetch && self.is_connected {
        smol::block_on(async {
            let _ = vm.deltachat_tx.send(DeltaChatCommand::FetchMessages).await;
        });
        self.last_fetch = Some(std::time::Instant::now());
    }
}
```

- [ ] **Step 5: Call check_auto_refresh in ui()**

Update `ui()` to call auto-refresh when connected:

```rust
if self.is_connected {
    self.check_auto_refresh(vm);
    // ... rest of UI
}
```

- [ ] **Step 6: Verify compilation**

Run: `cargo check -p mobile`  
Expected: SUCCESS

- [ ] **Step 7: Commit**

```bash
git add mobile/src/viewmodel/deltachat/ mobile/src/ui_tabs/deltachat.rs
git commit -m "feat: implement auto-refresh and event listener

Add listen_to_deltachat_events() to receive incoming messages
Add FetchMessages handler for background sync
Add check_auto_refresh() with 30s interval
Emit NewMessageReceived when messages arrive

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 16: Polish and Integration Tests

**Files:**
- Create: `mobile/src/viewmodel/deltachat/tests.rs`
- Modify: `mobile/src/viewmodel/deltachat/mod.rs`

**Interfaces:**
- Produces: Integration tests for full workflows

- [ ] **Step 1: Create integration test file**

Create `mobile/src/viewmodel/deltachat/tests.rs`:

```rust
//! Integration tests for DeltaChat actor

#[cfg(test)]
mod integration_tests {
    use crate::viewmodel::deltachat::{ContactInfo, DeltaChatActor, DeltaChatCommand, DeltaChatEvent};
    use crate::viewmodel::common::ViewModelEvent;
    use smol::channel::unbounded;
    use std::path::PathBuf;
    use tempfile::TempDir;
    
    async fn setup_test_context() -> (deltachat::Context, TempDir) {
        let tmpdir = TempDir::new().unwrap();
        let db_path = tmpdir.path().join("test.db");
        
        let context = deltachat::ContextBuilder::new(db_path)
            .with_id(1)
            .open()
            .await
            .unwrap();
        
        (context, tmpdir)
    }
    
    #[tokio::test]
    async fn test_full_workflow_add_contact_create_chat() {
        let (context, _tmpdir) = setup_test_context().await;
        
        // Add contact
        let contact_id = deltachat::contact::Contact::create(
            &context,
            "Test User",
            "test@example.com"
        ).await.unwrap();
        
        // Verify contact
        let contact = deltachat::contact::Contact::get_by_id(&context, contact_id)
            .await
            .unwrap();
        assert_eq!(contact.get_addr(), "test@example.com");
        
        // Create chat
        let chat_id = deltachat::chat::create_by_contact_id(&context, contact_id)
            .await
            .unwrap();
        
        // Verify chat
        let chat = deltachat::chat::Chat::load_from_db(&context, chat_id)
            .await
            .unwrap();
        assert_eq!(chat.get_type(), deltachat::chat::Chattype::Single);
    }
    
    #[test]
    fn test_runtime_bridge() {
        smol::block_on(async {
            let tokio_rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            
            let result = smol::unblock(move || {
                tokio_rt.block_on(async {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    42
                })
            }).await;
            
            assert_eq!(result, 42);
        });
    }
}
```

- [ ] **Step 2: Add tests module to mod.rs**

Update `mobile/src/viewmodel/deltachat/mod.rs`:

```rust
pub mod actor;
pub mod commands;
pub mod events;

#[cfg(test)]
mod tests;

pub use actor::DeltaChatActor;
pub use commands::DeltaChatCommand;
pub use events::{ChatInfo, ContactInfo, DeltaChatEvent, MessageInfo};
```

- [ ] **Step 3: Run integration tests**

Run: `cargo test -p mobile deltachat::tests`  
Expected: All tests PASS

- [ ] **Step 4: Add error handling improvements**

Update `emit_event()` in actor to handle channel errors gracefully:

```rust
fn emit_event(&self, event: DeltaChatEvent) {
    let event_tx = self.event_tx.clone();
    let result = smol::block_on(async move {
        event_tx.send(ViewModelEvent::DeltaChat(event)).await
    });
    
    if let Err(e) = result {
        log::error!("Failed to emit event: {}", e);
    }
}
```

- [ ] **Step 5: Add logging improvements**

Ensure all command handlers have appropriate logging:
- INFO for successful operations
- ERROR for failures
- DEBUG for internal operations

Review and add missing log statements.

- [ ] **Step 6: Final compilation check**

Run: `cargo check -p mobile --all-features`  
Expected: SUCCESS

- [ ] **Step 7: Run all tests**

Run: `cargo test -p mobile`  
Expected: All tests PASS

- [ ] **Step 8: Commit**

```bash
git add mobile/src/viewmodel/deltachat/
git commit -m "test: add integration tests and polish

Add integration tests for add contact + create chat workflow
Add runtime bridge test
Improve error handling in emit_event()
Add comprehensive logging

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Final Verification

- [ ] **Step 1: Full compilation check**

Run: `cargo build -p mobile --release`  
Expected: SUCCESS

- [ ] **Step 2: Run all tests**

Run: `cargo test -p mobile`  
Expected: All tests PASS

- [ ] **Step 3: Manual testing checklist (if desktop available)**

Configuration:
- [ ] Open DeltaChat tab
- [ ] Click "Configure Account"
- [ ] Enter email/password
- [ ] Verify progress bar shows
- [ ] Verify successful configuration
- [ ] Verify connection status shows "Connected"

Contacts:
- [ ] Click "Add Contact"
- [ ] Enter email address
- [ ] Verify contact appears in list

Chats:
- [ ] Click "Chat with [contact]"
- [ ] Verify chat appears in chat list
- [ ] Select chat
- [ ] Verify empty message view

Messaging:
- [ ] Type message in compose field
- [ ] Click Send (or press Enter)
- [ ] Verify message appears in chat

- [ ] **Step 4: Final commit**

```bash
git add .
git commit -m "feat: complete DeltaChat integration MVP

Full MVP implementation complete:
- Configuration dialog with progress
- Contact management (add, list)
- Chat management (create, list, select)
- Messaging (send, receive, auto-refresh)
- Actor-based MVVM with smol↔tokio bridge
- Integration tests

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Success Criteria

MVP is complete when:

- [ ] All 16 tasks completed
- [ ] All tests pass
- [ ] Project compiles on all platforms
- [ ] Manual testing checklist passed (or documented for platforms without access)
- [ ] Code committed to feature branch

## Next Steps

After MVP completion:
1. Test on Android platform
2. Test on WASM platform (document limitations)
3. Create pull request for review
4. Plan Phase 2 features (group chats, attachments, etc.)
