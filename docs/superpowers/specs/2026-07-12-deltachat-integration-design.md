# DeltaChat Integration Design

**Date:** 2026-07-12  
**Author:** Claude Sonnet 4.5  
**Status:** Approved  
**Target Release:** MVP - Minimal Viable Product

## Executive Summary

This document specifies the integration of DeltaChat encrypted messaging into the Dure application. DeltaChat provides end-to-end encrypted email-based messaging using the Autocrypt standard, allowing users to send encrypted messages over standard email infrastructure.

**Scope:** Minimal MVP covering configuration, contact management, and basic text messaging.

**Approach:** Direct integration with `deltachat-core` library using Dure's actor-based MVVM architecture with a runtime bridge to handle async runtime differences (DeltaChat uses tokio, Dure uses smol).

## Design Decisions

### Key Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Database** | Separate SQLite file | DeltaChat library expects to own its database; separate file avoids schema conflicts |
| **Database Location** | Dure's data directory | Co-located with Dure's main database for unified backups |
| **Platform Support** | All platforms (Desktop/Android/WASM) | Align with Dure's cross-platform goals |
| **MVP Scope** | Configuration + Contacts + Basic Messaging | Incremental development; can add features later |
| **Account Support** | Single account, designed for multi-account | Start simple, but future-proof architecture |
| **Configuration UI** | Dialog-based workflow | Clean UX; matches user's original request |
| **Message Sync** | Auto-sync when tab active | Balance freshness vs. resource usage |
| **Integration Approach** | Direct deltachat-core with runtime bridge | Full feature support, maintained library, proper encryption |

### Database Strategy

- **File:** `<dure-data-dir>/deltachat-default.db`
  - Desktop Linux: `~/.local/share/dure/deltachat-default.db`
  - Desktop macOS: `~/Library/Application Support/dure/deltachat-default.db`
  - Android: App's private data directory
  - WASM: IndexedDB (via sqlite-wasm)
- **Schema:** Managed by DeltaChat library (automatic migrations)
- **Future:** Multi-account support via `deltachat-<account-name>.db`

### Platform Support

All platforms supported:
- Desktop (Linux x86_64/aarch64, macOS, Windows)
- Android (native-activity)
- WASM (browser)

No platform-specific `#[cfg(...)]` restrictions on the DeltaChat tab.

---

## Architecture Overview

### High-Level Structure

The DeltaChat integration follows Dure's actor-based MVVM pattern with a runtime bridge to handle async runtime differences:

```
┌─────────────────────────────────────────────────────────┐
│ UI Layer (egui)                                         │
│  └─ mobile/src/ui_tabs/deltachat.rs                    │
│      - Configuration dialog                             │
│      - Contact list view                                │
│      - Chat list view                                   │
│      - Message view & compose                           │
└────────────────┬────────────────────────────────────────┘
                 │ Commands (send)
                 │ Events (receive)
                 ▼
┌─────────────────────────────────────────────────────────┐
│ ViewModel Layer                                         │
│  └─ mobile/src/viewmodel/mod.rs                        │
│      - deltachat_tx: Sender<DeltaChatCommand>          │
│      - event_rx: Receiver<ViewModelEvent>              │
└────────────────┬────────────────────────────────────────┘
                 │ Channel communication
                 ▼
┌─────────────────────────────────────────────────────────┐
│ Actor Layer (smol runtime)                              │
│  └─ mobile/src/viewmodel/deltachat/actor.rs           │
│      - DeltaChatActor                                   │
│      - Processes commands                               │
│      - Emits events                                     │
└────────────────┬────────────────────────────────────────┘
                 │ Runtime bridge (spawn_blocking)
                 ▼
┌─────────────────────────────────────────────────────────┐
│ DeltaChat Core (tokio runtime)                         │
│  └─ deltachat::Context                                 │
│      - Database: dure-data-dir/deltachat-default.db    │
│      - IMAP/SMTP connections                            │
│      - Encryption/Autocrypt                             │
└─────────────────────────────────────────────────────────┘
```

### Module Structure

```
mobile/src/
├── ui_tabs/
│   ├── mod.rs                    # Add Tab::DeltaChat enum variant
│   └── deltachat.rs              # NEW: DeltaChat tab UI
├── viewmodel/
│   ├── mod.rs                    # Add deltachat_tx channel, spawn deltachat actor
│   └── deltachat/                # NEW: DeltaChat actor module
│       ├── mod.rs                # Public API, re-exports
│       ├── actor.rs              # DeltaChatActor implementation
│       ├── commands.rs           # Command enum
│       ├── events.rs             # Event enum
│       └── runtime_bridge.rs     # smol↔tokio utilities (optional, may inline)
└── dure.rs                       # Add DeltaChat tab to TabBar
```

### Dependencies to Add

In `mobile/Cargo.toml`:

```toml
[dependencies]
deltachat = "1.146"  # Latest stable version as of design date
tokio = { version = "1", features = ["rt", "rt-multi-thread", "macros"] }
```

**Note:** Verify latest `deltachat` version at implementation time.

---

## Component Details

### Commands (`mobile/src/viewmodel/deltachat/commands.rs`)

Commands the UI sends to the DeltaChat actor:

```rust
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
    
    // Background sync (internal, triggered by UI timer)
    FetchMessages,
}
```

**Design Notes:**
- All commands are `Clone` for flexibility
- `contact_id` and `chat_id` use `u32` to match DeltaChat's `ContactId` and `ChatId` types
- `FetchMessages` is internal command triggered by UI timer when tab is active

### Events (`mobile/src/viewmodel/deltachat/events.rs`)

Events the actor emits back to the UI:

```rust
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
        progress: i32,       // 0-1000 (per-mille, DeltaChat convention)
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
```

**Design Notes:**
- Info structs (`ContactInfo`, `ChatInfo`, `MessageInfo`) are separate to decouple from DeltaChat's internal types
- All events are `Clone` for broadcast to UI
- `ConfigurationProgress` uses i32 (0-1000) to match DeltaChat's per-mille convention
- `timestamp` fields use `i64` (Unix timestamp)

### Actor State (`mobile/src/viewmodel/deltachat/actor.rs`)

```rust
pub struct DeltaChatActor {
    // Communication channels
    command_rx: Receiver<DeltaChatCommand>,
    event_tx: Sender<ViewModelEvent>,
    
    // DeltaChat context (wrapped in Option for lazy initialization)
    context: Option<deltachat::Context>,
    
    // Runtime bridge
    tokio_runtime: tokio::runtime::Runtime,
    
    // State
    database_path: PathBuf,
    is_configured: bool,
    is_connected: bool,
    current_chat_id: Option<u32>,
}
```

**Key Methods:**

```rust
impl DeltaChatActor {
    pub fn new(
        command_rx: Receiver<DeltaChatCommand>,
        event_tx: Sender<ViewModelEvent>,
        database_path: PathBuf,
    ) -> Self;
    
    pub async fn run(mut self);
    
    async fn initialize_context(&mut self) -> Result<(), String>;
    
    async fn handle_command(&mut self, cmd: DeltaChatCommand) -> Result<(), String>;
    
    fn emit_event(&self, event: DeltaChatEvent);
    
    async fn listen_to_deltachat_events(&self);
}
```

**Initialization Pattern:**

The actor initializes lazily:
1. Actor starts with `context: None`
2. First `Configure` command triggers `initialize_context()`
3. Context persists for actor lifetime

**Event Listening:**

DeltaChat emits its own events (e.g., `IncomingMsg`, `MsgsChanged`). The actor spawns a background task to listen and forward relevant events to the UI.

### UI Tab State (`mobile/src/ui_tabs/deltachat.rs`)

```rust
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DeltaChatTab {
    // Connection state (from events)
    is_configured: bool,
    is_connected: bool,
    configured_email: Option<String>,
    
    // Configuration dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    config_dialog_open: bool,
    config_email: String,
    config_password: String,
    config_in_progress: bool,
    config_progress: i32,
    config_error: Option<String>,
    
    // Contact list state
    #[cfg_attr(feature = "serde", serde(skip))]
    add_contact_dialog_open: bool,
    add_contact_email: String,
    contacts: Vec<ContactInfo>,
    
    // Chat list state
    chats: Vec<ChatInfo>,
    selected_chat_id: Option<u32>,
    
    // Message view state
    messages: Vec<MessageInfo>,
    compose_text: String,
    
    // Auto-refresh timer
    #[cfg_attr(feature = "serde", serde(skip))]
    last_fetch: Option<std::time::Instant>,
}
```

**Key Methods:**

```rust
impl DeltaChatTab {
    pub fn ui(&mut self, ui: &mut egui::Ui, vm: &ViewModel);
    
    fn render_config_dialog(&mut self, ui: &mut egui::Ui, vm: &ViewModel);
    fn render_add_contact_dialog(&mut self, ui: &mut egui::Ui, vm: &ViewModel);
    fn render_contact_list(&mut self, ui: &mut egui::Ui);
    fn render_chat_list(&mut self, ui: &mut egui::Ui, vm: &ViewModel);
    fn render_message_view(&mut self, ui: &mut egui::Ui, vm: &ViewModel);
    
    fn handle_events(&mut self, vm: &ViewModel);
    fn check_auto_refresh(&mut self, vm: &ViewModel);
}
```

---

## Data Flow

### 3.1 Configuration Flow (First-Time Setup)

```
User clicks "Configure Account" button
    │
    ▼
UI opens dialog, user enters email/password
    │
    ▼
UI sends: DeltaChatCommand::Configure { email, password }
    │
    ▼
Actor receives command in smol runtime
    │
    ▼
Actor spawns tokio task via runtime bridge:
    tokio_runtime.block_on(async {
        // Initialize context if needed
        if context.is_none() {
            context = Some(initialize_context().await?);
        }
        
        // Set configuration
        context.set_config(Config::Addr, email).await?;
        context.set_config(Config::MailPw, password).await?;
        
        // Start configuration
        context.configure().await?;
    })
    │
    ▼
DeltaChat emits progress events (via get_event_emitter())
    EventType::ConfigureProgress { progress, comment }
    │
    ▼
Actor listens to event stream, forwards to UI:
    DeltaChatEvent::ConfigurationProgress { 500, "Checking IMAP..." }
    │
    ▼
UI updates progress bar in dialog
    │
    ▼
Configuration completes (or fails)
    │
    ▼
Actor emits: DeltaChatEvent::Configured { email }
    (or DeltaChatEvent::ConfigurationFailed { error })
    │
    ▼
UI closes dialog on success, shows error on failure
    │
    ▼
Auto-connect: UI sends DeltaChatCommand::Connect
    │
    ▼
Actor: context.start_io().await → DeltaChatEvent::Connected
```

### 3.2 Message Sync Flow (When Tab Active)

```
User switches to DeltaChat tab
    │
    ▼
UI starts timer (check every 30 seconds)
    │
    ▼
Timer fires → UI sends: DeltaChatCommand::FetchMessages
    │
    ▼
Actor: 
    tokio_runtime.block_on(async {
        context.background_fetch().await
    })
    │
    ▼
DeltaChat polls IMAP server for new messages
    │
    ▼
If new messages arrive:
    DeltaChat emits EventType::IncomingMsg { chat_id, msg_id }
    │
    ▼
Actor catches event in event listener task:
    - Fetch message details: Message::load_from_db(&context, msg_id).await
    - Fetch sender info: Contact::get_by_id(&context, msg.get_from_id()).await
    - Build MessageInfo struct
    │
    ▼
Actor emits: DeltaChatEvent::NewMessageReceived { chat_id, message }
    │
    ▼
UI receives event, updates message list if chat_id matches selected chat
    │
    ▼
User switches to another tab
    │
    ▼
Timer stops → No more FetchMessages commands
```

**Auto-Refresh Logic:**

```rust
fn check_auto_refresh(&mut self, vm: &ViewModel) {
    const REFRESH_INTERVAL: Duration = Duration::from_secs(30);
    
    let should_fetch = self.last_fetch
        .map(|t| t.elapsed() > REFRESH_INTERVAL)
        .unwrap_or(true);
    
    if should_fetch && self.is_connected {
        vm.send_command(DeltaChatCommand::FetchMessages);
        self.last_fetch = Some(Instant::now());
    }
}
```

### 3.3 Sending Message Flow

```
User types message in compose field, clicks Send button
    │
    ▼
UI validates: chat selected, message not empty
    │
    ▼
UI sends: DeltaChatCommand::SendTextMessage { chat_id, text }
    │
    ▼
Actor receives command:
    tokio_runtime.block_on(async {
        let chat = Chat::load_from_db(&context, chat_id).await?;
        let msg_id = chat::send_text_msg(&context, chat_id, text).await?;
        msg_id
    })
    │
    ▼
DeltaChat queues message for SMTP send (background job)
    │
    ▼
Actor emits: DeltaChatEvent::MessageSent { msg_id, chat_id }
    │
    ▼
UI receives event:
    - Clears compose field
    - Adds optimistic message to message list (mark as sending)
    │
    ▼
DeltaChat sends message via SMTP in background
    │
    ▼
DeltaChat emits EventType::MsgsChanged { chat_id, msg_id }
    (message state changed: Sending → Sent → Delivered)
    │
    ▼
Actor re-fetches message, emits update event
    │
    ▼
UI updates message status (show checkmark)
```

### 3.4 Runtime Bridge Pattern

**Challenge:** DeltaChat uses `tokio`, Dure uses `smol`.

**Solution:** Dedicated tokio runtime in actor, use `smol::unblock` to bridge:

```rust
impl DeltaChatActor {
    pub fn new(
        command_rx: Receiver<DeltaChatCommand>, 
        event_tx: Sender<ViewModelEvent>,
        database_path: PathBuf,
    ) -> Self {
        // Create dedicated tokio runtime for DeltaChat
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
            current_chat_id: None,
        }
    }
    
    pub async fn run(mut self) {
        log::info!("DeltaChatActor started");
        
        // Main command loop runs in smol
        while let Ok(cmd) = self.command_rx.recv().await {
            // Execute tokio code via blocking call
            let result = smol::unblock(|| {
                // Move into tokio runtime
                self.tokio_runtime.block_on(async {
                    self.handle_command_tokio(cmd).await
                })
            }).await;
            
            // Handle result
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
    
    async fn handle_command_tokio(&mut self, cmd: DeltaChatCommand) -> Result<(), String> {
        // This runs in tokio runtime
        match cmd {
            DeltaChatCommand::Configure { email, password } => {
                self.configure_internal(&email, &password).await?;
            }
            // ... other commands
        }
        Ok(())
    }
}
```

**Key Points:**
- `tokio_runtime` is created once in `new()`
- Main actor loop (`run()`) runs in smol
- Each command is executed via `smol::unblock(() => tokio_runtime.block_on(async { ... }))`
- This bridges the two async runtimes without conflicts

**Event Listener Bridge:**

DeltaChat's event emitter is tokio-based. We spawn a tokio task to listen and forward events to smol:

```rust
async fn listen_to_deltachat_events(&self) {
    let context = self.context.as_ref().unwrap().clone();
    let event_tx = self.event_tx.clone();
    
    // Spawn tokio task to listen to DeltaChat events
    self.tokio_runtime.spawn(async move {
        let mut events = context.get_event_emitter();
        
        while let Some(event) = events.recv().await {
            match event.typ {
                EventType::IncomingMsg { chat_id, msg_id } => {
                    // Fetch message details
                    let msg = Message::load_from_db(&context, msg_id).await.ok()?;
                    let contact = Contact::get_by_id(&context, msg.get_from_id()).await.ok()?;
                    
                    // Build MessageInfo
                    let message_info = MessageInfo {
                        id: msg_id.to_u32(),
                        from_contact_id: contact.get_id().to_u32(),
                        from_name: contact.get_display_name().to_string(),
                        text: msg.get_text(),
                        timestamp: msg.get_timestamp(),
                        is_outgoing: false,
                        is_seen: false,
                    };
                    
                    // Forward to smol via channel (channels are Send+Sync)
                    let _ = event_tx.send(ViewModelEvent::DeltaChat(
                        DeltaChatEvent::NewMessageReceived {
                            chat_id: chat_id.to_u32(),
                            message: message_info,
                        }
                    )).await;
                }
                EventType::ConfigureProgress { progress, comment } => {
                    let _ = event_tx.send(ViewModelEvent::DeltaChat(
                        DeltaChatEvent::ConfigurationProgress { progress, comment }
                    )).await;
                }
                // ... handle other events
                _ => {}
            }
        }
    });
}
```

---

## Error Handling

### Error Categories

1. **Configuration Errors:**
   - Invalid email format
   - Wrong password
   - Unsupported email provider
   - Network connectivity issues

2. **Connection Errors:**
   - IMAP/SMTP server unreachable
   - Authentication expired/revoked
   - Firewall/port blocking

3. **Operation Errors:**
   - Contact already exists
   - Chat doesn't exist
   - Message send failed
   - Database errors

### Error Handling Strategy

**At Actor Level:**

```rust
async fn handle_command_tokio(&mut self, cmd: DeltaChatCommand) -> Result<(), String> {
    match cmd {
        DeltaChatCommand::Configure { email, password } => {
            // Emit start event
            self.emit_event(DeltaChatEvent::ConfigurationStarted);
            
            match self.configure_internal(&email, &password).await {
                Ok(_) => {
                    self.is_configured = true;
                    self.emit_event(DeltaChatEvent::Configured { email });
                    Ok(())
                }
                Err(e) => {
                    // Convert deltachat error to user-friendly message
                    let error_msg = match e.to_string().as_str() {
                        s if s.contains("authentication") || s.contains("login") => 
                            "Invalid email or password. Please check your credentials.".to_string(),
                        s if s.contains("network") || s.contains("connection") => 
                            "Cannot reach email server. Check your internet connection.".to_string(),
                        s if s.contains("timeout") => 
                            "Connection timed out. Server may be slow or unreachable.".to_string(),
                        _ => format!("Configuration failed: {}", e),
                    };
                    
                    self.emit_event(DeltaChatEvent::ConfigurationFailed { 
                        error: error_msg.clone()
                    });
                    Err(error_msg)
                }
            }
        }
        
        DeltaChatCommand::SendTextMessage { chat_id, text } => {
            match chat::send_text_msg(&self.context.as_ref().unwrap(), chat_id.into(), &text).await {
                Ok(msg_id) => {
                    self.emit_event(DeltaChatEvent::MessageSent { 
                        msg_id: msg_id.to_u32(),
                        chat_id,
                    });
                    Ok(())
                }
                Err(e) => {
                    let error_msg = format!("Failed to send message: {}", e);
                    self.emit_event(DeltaChatEvent::Error {
                        operation: "send_message".to_string(),
                        error: error_msg.clone(),
                    });
                    Err(error_msg)
                }
            }
        }
        
        // ... other commands
    }
}
```

**At UI Level:**

```rust
impl DeltaChatTab {
    pub fn ui(&mut self, ui: &mut egui::Ui, vm: &ViewModel) {
        // Poll for events from actor
        self.handle_events(vm);
        
        // Show configuration dialog if needed
        if self.config_dialog_open {
            self.render_config_dialog(ui, vm);
        }
        
        // Render main UI based on state
        if !self.is_configured {
            ui.vertical_centered(|ui| {
                ui.heading("DeltaChat Not Configured");
                ui.label("Configure your email account to start using encrypted messaging.");
                
                if ui.button("Configure Account").clicked() {
                    self.config_dialog_open = true;
                }
            });
        } else if !self.is_connected {
            ui.vertical_centered(|ui| {
                ui.heading("Disconnected");
                ui.label(format!("Account: {}", self.configured_email.as_ref().unwrap()));
                
                if ui.button("Connect").clicked() {
                    vm.send_command(DeltaChatCommand::Connect);
                }
            });
        } else {
            // Show main messaging UI
            self.render_chat_list(ui, vm);
            self.render_message_view(ui, vm);
            
            // Auto-refresh when tab is active
            self.check_auto_refresh(vm);
        }
    }
    
    fn handle_events(&mut self, vm: &ViewModel) {
        while let Ok(event) = vm.try_recv_event() {
            match event {
                ViewModelEvent::DeltaChat(dc_event) => match dc_event {
                    DeltaChatEvent::ConfigurationFailed { error } => {
                        self.config_in_progress = false;
                        self.config_error = Some(error);
                        // Dialog stays open to show error
                    }
                    
                    DeltaChatEvent::Configured { email } => {
                        self.is_configured = true;
                        self.configured_email = Some(email);
                        self.config_dialog_open = false;
                        self.config_error = None;
                        self.config_in_progress = false;
                        
                        // Auto-connect
                        vm.send_command(DeltaChatCommand::Connect);
                    }
                    
                    DeltaChatEvent::ConfigurationProgress { progress, comment } => {
                        self.config_progress = progress;
                        // Optionally show comment in dialog
                    }
                    
                    DeltaChatEvent::Connected => {
                        self.is_connected = true;
                        // Fetch initial data
                        vm.send_command(DeltaChatCommand::ListContacts);
                        vm.send_command(DeltaChatCommand::ListChats);
                    }
                    
                    DeltaChatEvent::ContactsListed { contacts } => {
                        self.contacts = contacts;
                    }
                    
                    DeltaChatEvent::ChatsListed { chats } => {
                        self.chats = chats;
                    }
                    
                    DeltaChatEvent::NewMessageReceived { chat_id, message } => {
                        // Add to messages if this chat is selected
                        if self.selected_chat_id == Some(chat_id) {
                            self.messages.push(message);
                        }
                        
                        // Update chat's unread count
                        if let Some(chat) = self.chats.iter_mut().find(|c| c.id == chat_id) {
                            chat.unread_count += 1;
                        }
                    }
                    
                    DeltaChatEvent::Error { operation, error } => {
                        // Show toast notification or log error
                        log::error!("DeltaChat error in {}: {}", operation, error);
                        // TODO: Show user-facing error notification
                    }
                    
                    // ... handle other events
                    _ => {}
                }
                _ => {}
            }
        }
    }
    
    fn render_config_dialog(&mut self, ui: &mut egui::Ui, vm: &ViewModel) {
        egui::Window::new("Configure DeltaChat")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("Email Address:");
                    ui.text_edit_singleline(&mut self.config_email);
                    
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.config_password).password(true));
                    
                    // Show error if any
                    if let Some(error) = &self.config_error {
                        ui.colored_label(egui::Color32::RED, error);
                    }
                    
                    // Show progress if configuring
                    if self.config_in_progress {
                        let progress = self.config_progress as f32 / 1000.0; // 0-1000 → 0.0-1.0
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
                        
                        if ui.add_enabled(can_submit, egui::Button::new("Configure")).clicked() {
                            self.config_in_progress = true;
                            self.config_error = None;
                            self.config_progress = 0;
                            
                            vm.send_command(DeltaChatCommand::Configure {
                                email: self.config_email.clone(),
                                password: self.config_password.clone(),
                            });
                        }
                    });
                });
            });
    }
}
```

### Recovery Strategies

**Automatic Recovery:**
- **Connection drops:** Actor auto-reconnects on next `FetchMessages` command
- **Transient network errors:** Retry with exponential backoff (built into deltachat-core)

**User-Initiated Recovery:**
- **Invalid credentials:** User clicks "Reconfigure" button, opens config dialog again
- **Persistent errors:** "Disconnect" button to reset state, try again

**Graceful Degradation:**
- If fetch fails, keep showing cached messages
- Disable send button if disconnected
- Show connection status indicator in tab (dot icon: green=connected, red=disconnected, yellow=connecting)

### Database Errors

**Initialization:**

```rust
async fn initialize_context(&mut self) -> Result<(), String> {
    let db_path = self.database_path.clone();
    
    // Ensure database directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create database directory: {}", e))?;
    }
    
    // Open/create database
    let context = deltachat::ContextBuilder::new(db_path)
        .with_id(1)
        .open()
        .await
        .map_err(|e| format!("Cannot open DeltaChat database: {}", e))?;
    
    self.context = Some(context);
    
    // Start event listener
    self.listen_to_deltachat_events();
    
    Ok(())
}
```

**Migration Errors:**
- DeltaChat handles schema migrations automatically on `.open()`
- If migration fails, emit error event with instructions
- User may need to delete corrupt database (show file path in error message)

---

## Testing Strategy

### Unit Tests

**Commands & Events (Pure Data):**

Test all command and event variants to ensure serialization and pattern matching work:

```rust
// mobile/src/viewmodel/deltachat/commands.rs
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
}
```

```rust
// mobile/src/viewmodel/deltachat/events.rs
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
    }
    
    #[test]
    fn test_message_info_clone() {
        let msg = MessageInfo {
            id: 42,
            from_contact_id: 1,
            from_name: "Bob".to_string(),
            text: "Hello!".to_string(),
            timestamp: 1234567890,
            is_outgoing: false,
            is_seen: false,
        };
        
        let cloned = msg.clone();
        assert_eq!(cloned.id, msg.id);
        assert_eq!(cloned.text, msg.text);
    }
}
```

### Actor Tests (Channel Communication)

Test that the actor can receive commands and emit events:

```rust
// mobile/src/viewmodel/deltachat/actor.rs
#[cfg(test)]
mod tests {
    use super::*;
    use smol::channel::unbounded;
    
    #[test]
    fn test_actor_channel_setup() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = unbounded();
            let (event_tx, event_rx) = unbounded();
            
            // Send a command
            cmd_tx.send(DeltaChatCommand::GetConnectionStatus).await.unwrap();
            
            // Verify actor can receive it
            let received = cmd_rx.recv().await.unwrap();
            assert!(matches!(received, DeltaChatCommand::GetConnectionStatus));
        });
    }
    
    #[test]
    fn test_event_emission() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = unbounded();
            let (event_tx, event_rx) = unbounded();
            
            // Simulate emitting an event
            let event = DeltaChatEvent::Connected;
            event_tx.send(ViewModelEvent::DeltaChat(event)).await.unwrap();
            
            // Verify event was emitted
            let received = event_rx.recv().await.unwrap();
            assert!(matches!(received, ViewModelEvent::DeltaChat(DeltaChatEvent::Connected)));
        });
    }
}
```

### Integration Tests (Real DeltaChat)

Test with real DeltaChat context using temporary database:

```rust
// mobile/src/viewmodel/deltachat/tests.rs
#[cfg(test)]
mod integration_tests {
    use super::*;
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
    async fn test_context_initialization() {
        let (context, _tmpdir) = setup_test_context().await;
        
        // Verify context is open
        assert!(context.is_open().await);
    }
    
    #[tokio::test]
    async fn test_add_contact() {
        let (context, _tmpdir) = setup_test_context().await;
        
        // Add a contact
        let contact_id = deltachat::contact::Contact::create(
            &context, 
            "Test User", 
            "test@example.com"
        ).await.unwrap();
        
        // Verify contact was added
        let contact = deltachat::contact::Contact::get_by_id(&context, contact_id)
            .await
            .unwrap();
        
        assert_eq!(contact.get_addr(), "test@example.com");
        assert_eq!(contact.get_display_name(), "Test User");
    }
    
    #[tokio::test]
    async fn test_create_chat() {
        let (context, _tmpdir) = setup_test_context().await;
        
        // Add contact first
        let contact_id = deltachat::contact::Contact::create(
            &context, 
            "Chat Partner", 
            "partner@example.com"
        ).await.unwrap();
        
        // Create 1:1 chat
        let chat_id = deltachat::chat::create_by_contact_id(&context, contact_id)
            .await
            .unwrap();
        
        // Verify chat was created
        let chat = deltachat::chat::Chat::load_from_db(&context, chat_id)
            .await
            .unwrap();
        
        assert_eq!(chat.get_type(), deltachat::chat::Chattype::Single);
    }
    
    // Note: Testing actual email send/receive requires email server setup
    // For MVP, we'll rely on manual testing with real email accounts
}
```

### Runtime Bridge Tests

Verify smol↔tokio bridge works correctly:

```rust
#[cfg(test)]
mod runtime_bridge_tests {
    use super::*;
    
    #[test]
    fn test_smol_can_call_tokio() {
        smol::block_on(async {
            let tokio_rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            
            let result = smol::unblock(move || {
                tokio_rt.block_on(async {
                    // Simulate async work in tokio
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    42
                })
            }).await;
            
            assert_eq!(result, 42);
        });
    }
    
    #[test]
    fn test_tokio_runtime_thread_count() {
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        
        // Verify runtime was created (no panics)
        drop(tokio_rt);
    }
}
```

### UI Tests

**For MVP:** Manual testing is sufficient. Create a test checklist:

**Configuration Tests:**
- [ ] Open config dialog
- [ ] Enter invalid email → Error shown
- [ ] Enter valid credentials → Progress bar updates
- [ ] Configuration succeeds → Dialog closes, shows connected status
- [ ] Configuration fails → Error message shown, can retry

**Contact Tests:**
- [ ] Add contact with valid email → Appears in list
- [ ] Add duplicate contact → Error handling
- [ ] Contact list updates when new contact added

**Messaging Tests:**
- [ ] Create chat with contact → Chat appears in list
- [ ] Send message → Message appears in chat
- [ ] Receive message → New message notification, appears in chat
- [ ] Switch tabs → Auto-refresh stops
- [ ] Switch back to DeltaChat tab → Auto-refresh resumes

**Error Handling Tests:**
- [ ] Disconnect network during fetch → Error shown, UI still usable
- [ ] Send message while disconnected → Error shown
- [ ] Reconnect → Can send again

### Test Coverage Goals

**Minimum for MVP:**
- ✅ All command/event structs have basic tests
- ✅ Actor channel communication works (send command, receive event)
- ✅ Runtime bridge pattern works (smol↔tokio)
- ✅ At least 3 integration tests: initialize context, add contact, create chat
- ✅ Manual UI testing checklist completed

**Future Enhancements:**
- Mock IMAP/SMTP servers for network-isolated tests
- Property-based testing with `proptest` for message parsing
- Performance tests (how many messages before UI lags?)
- Automated UI tests with `egui_test_harness`

---

## Implementation Phases

### Phase 1: Foundation (Core Infrastructure)

**Goal:** Set up actor infrastructure, runtime bridge, basic tab UI

**Files to Create:**
- `mobile/src/viewmodel/deltachat/mod.rs`
- `mobile/src/viewmodel/deltachat/commands.rs`
- `mobile/src/viewmodel/deltachat/events.rs`
- `mobile/src/viewmodel/deltachat/actor.rs`
- `mobile/src/ui_tabs/deltachat.rs`

**Files to Modify:**
- `mobile/Cargo.toml` - Add `deltachat` and `tokio` dependencies
- `mobile/src/viewmodel/mod.rs` - Add `pub mod deltachat;`, spawn deltachat actor
- `mobile/src/ui_tabs/mod.rs` - Add `pub mod deltachat;`, add `Tab::DeltaChat` enum
- `mobile/src/dure.rs` - Add DeltaChat tab to TabBar

**Tasks:**
1. Add dependencies to Cargo.toml
2. Create command/event enums with unit tests
3. Create basic DeltaChatActor with runtime bridge
4. Create DeltaChatTab with placeholder UI
5. Wire actor into ViewModel
6. Add tab to UI

**Success Criteria:**
- [ ] Project compiles with new dependencies
- [ ] DeltaChat tab appears in TabBar
- [ ] Actor starts and receives commands (log verification)
- [ ] Unit tests pass

### Phase 2: Configuration (Email Setup)

**Goal:** Implement configuration dialog and connect/disconnect

**Tasks:**
1. Implement `Configure` command handler in actor
2. Implement `Connect`/`Disconnect` command handlers
3. Implement configuration dialog UI
4. Forward DeltaChat ConfigureProgress events to UI
5. Handle configuration errors with user-friendly messages
6. Persist configured state (use Dure's config system)

**Success Criteria:**
- [ ] Can open config dialog
- [ ] Can enter email/password
- [ ] Progress bar updates during configuration
- [ ] Success: dialog closes, shows "Connected" status
- [ ] Failure: error message shown in dialog
- [ ] Can disconnect and reconfigure

**Testing:**
- Manual test with real email account (Gmail, Yahoo, etc.)
- Test with invalid credentials → error handling works
- Test with network disconnected → timeout error shown

### Phase 3: Contacts (Contact Management)

**Goal:** Add, list, and view contacts

**Tasks:**
1. Implement `AddContact`, `ListContacts`, `GetContactInfo` commands
2. Create add contact dialog UI
3. Create contact list view UI
4. Handle duplicate contact errors

**Success Criteria:**
- [ ] Can add contact by email
- [ ] Contact appears in list after adding
- [ ] Contact list shows name, email
- [ ] Duplicate contact shows error

**Testing:**
- Add contact, verify appears in list
- Add same contact twice, verify error
- Restart app, verify contacts persist

### Phase 4: Chats (Chat Creation & Listing)

**Goal:** Create chats, list chats, select chat

**Tasks:**
1. Implement `CreateChat`, `ListChats`, `SelectChat` commands
2. Create chat list view UI
3. Create empty message view (selected chat, no messages yet)
4. Update chat list on new chat created

**Success Criteria:**
- [ ] Can create chat with contact
- [ ] Chat appears in chat list
- [ ] Can select chat (message view shows)
- [ ] Chat list persists across app restarts

**Testing:**
- Create chat with contact, verify appears
- Select chat, verify message view shows (empty)

### Phase 5: Messaging (Send & Receive)

**Goal:** Send and receive text messages

**Tasks:**
1. Implement `SendTextMessage`, `ListMessages`, `MarkMessagesSeen` commands
2. Create message list view UI (scroll view, message bubbles)
3. Create compose field + send button
4. Implement DeltaChat event listener for incoming messages
5. Implement auto-refresh timer (30s interval when tab active)
6. Handle message send errors

**Success Criteria:**
- [ ] Can send text message
- [ ] Sent message appears in chat
- [ ] Can receive message from another DeltaChat client
- [ ] Received message appears in chat
- [ ] Auto-refresh works (stops when tab inactive, resumes when active)
- [ ] Unread count updates

**Testing:**
- Send message to self (email loopback)
- Send message from another DeltaChat client (mobile app or repl)
- Verify received message appears
- Switch tabs, verify auto-refresh stops
- Switch back, verify auto-refresh resumes

### Phase 6: Polish & Bug Fixes

**Goal:** Improve UX, fix bugs, optimize performance

**Tasks:**
1. Add connection status indicator (dot icon: green/red/yellow)
2. Add loading spinners during operations
3. Add toast notifications for errors
4. Optimize message list rendering (virtual scrolling if needed)
5. Add keyboard shortcuts (Enter to send, Esc to close dialogs)
6. Write integration tests
7. Test on all platforms (Desktop, Android, WASM)

**Success Criteria:**
- [ ] UI feels responsive, no lag
- [ ] Errors are user-friendly
- [ ] Works on Desktop (Linux, macOS, Windows)
- [ ] Works on Android
- [ ] Works on WASM (with limitations noted)

---

## Platform-Specific Considerations

### Desktop (Linux, macOS, Windows)

**No special considerations:** Full support for all features.

**Database Path:**
- Linux: `~/.local/share/dure/deltachat-default.db`
- macOS: `~/Library/Application Support/dure/deltachat-default.db`
- Windows: `%APPDATA%\dure\deltachat-default.db`

### Android

**Database Path:** Use Android's app-specific storage via `android_activity` crate.

**Network Permissions:** Ensure INTERNET permission in AndroidManifest.xml (likely already present).

**Background Sync:** When tab is not active, sync stops. Future: consider Android WorkManager for background sync.

### WASM

**Database:** Use `sqlite-wasm` via IndexedDB (Dure already supports this).

**Network:** IMAP/SMTP over WebSockets or HTTPS proxies may be needed (DeltaChat supports this).

**Background Sync:** Browser tabs can't run background tasks when inactive. Auto-refresh works only when tab is visible.

**Limitations:**
- No persistent background sync (only when tab visible)
- Email server must support CORS or use proxy
- Some email providers may block browser-based clients

**Recommendation:** WASM support is experimental. Desktop and Android are primary targets for MVP.

---

## Future Enhancements (Post-MVP)

**Not in MVP scope, but designed to support:**

1. **Multi-Account Support**
   - UI: Account switcher dropdown
   - Actor: One actor per account, or single actor managing multiple contexts
   - Database: `deltachat-account1.db`, `deltachat-account2.db`

2. **Group Chats**
   - Commands: `CreateGroupChat`, `AddMember`, `RemoveMember`
   - UI: Group info view, member list

3. **File Attachments**
   - Commands: `SendFileMessage`, `DownloadAttachment`
   - UI: File picker, thumbnail preview, download button

4. **Message Reactions**
   - Commands: `ReactToMessage`
   - UI: Reaction picker, reaction count display

5. **Chat Features**
   - Archive/unarchive chats
   - Pin/unpin chats
   - Mute/unmute chats
   - Delete chats

6. **Message Features**
   - Forward messages
   - Delete messages
   - Quote/reply to messages
   - Search messages

7. **Notifications**
   - Desktop notifications for new messages
   - Unread badge on app icon
   - System tray integration

8. **Location Sharing**
   - Commands: `SendLocationMessage`, `EnableLocationStreaming`
   - UI: Map view, location picker

9. **OAuth2 Authentication**
   - Support Gmail, Outlook OAuth2 (no password needed)
   - Commands: `StartOAuth`, `CompleteOAuth`

10. **Contact Features**
    - Block/unblock contacts
    - Delete contacts
    - Contact verification (QR codes)
    - Contact search

---

## Security Considerations

### End-to-End Encryption

DeltaChat provides **Autocrypt**-based end-to-end encryption automatically:
- Public keys exchanged in email headers
- Messages encrypted when both parties have keys
- Verified chats (QR code verification) for enhanced security

**No additional work needed** - this is built into deltachat-core.

### Password Storage

**Current Design:** Password is passed to `context.set_config(Config::MailPw, password)` and stored in DeltaChat's database.

**DeltaChat Database Encryption:** DeltaChat database is **not encrypted by default**. Consider:
- Using Dure's existing `libsqlite3-hotbundle` encryption (if compatible)
- Or document that users should use full-disk encryption
- Future: add password protection to DeltaChat database

### Network Security

- IMAP/SMTP use TLS (enforced by DeltaChat)
- Certificate validation (enforced by DeltaChat)
- No plaintext passwords over network

### Token/Credential Handling

- Passwords never logged or displayed in UI after configuration
- Use `TextEdit::password(true)` for password field
- Clear password from memory after sending to actor (actor stores in DB)

---

## Open Questions & Risks

### Questions

1. **DeltaChat version:** Which version to target? (Recommendation: 1.146.x stable)
2. **Database encryption:** Should we encrypt the DeltaChat database? (Recommendation: defer to post-MVP, document reliance on full-disk encryption)
3. **WASM email provider compatibility:** Which email providers work in browsers? (Recommendation: test Gmail, Outlook; document limitations)

### Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Runtime bridge issues (smol↔tokio) | Medium | High | Test early, use `smol::unblock` pattern proven in other projects |
| DeltaChat library breaking changes | Low | Medium | Pin to stable version, monitor changelogs |
| Email provider blocking (WASM) | Medium | Medium | Document limitations, focus on Desktop/Android for MVP |
| Performance with large message history | Low | Medium | Implement virtual scrolling if needed, profile before optimizing |
| Database corruption | Low | High | Implement backup/export feature (post-MVP), document recovery steps |

---

## Success Metrics

**MVP is successful when:**

1. **Configuration:**
   - [ ] User can configure email account via dialog
   - [ ] Progress feedback during configuration
   - [ ] Error handling with clear messages

2. **Contacts:**
   - [ ] User can add contacts by email
   - [ ] Contact list displays correctly
   - [ ] Contacts persist across app restarts

3. **Messaging:**
   - [ ] User can create 1:1 chat with contact
   - [ ] User can send text messages
   - [ ] User can receive text messages from other DeltaChat clients
   - [ ] Messages display in correct order with timestamps
   - [ ] Auto-refresh works when tab is active

4. **Cross-Platform:**
   - [ ] Works on Desktop (Linux tested, macOS/Windows assumed working)
   - [ ] Works on Android
   - [ ] Works on WASM (with limitations documented)

5. **Quality:**
   - [ ] No crashes during normal operation
   - [ ] Unit tests pass
   - [ ] Integration tests pass
   - [ ] Code follows Dure's patterns and style

---

## Appendix A: DeltaChat API Reference

**Key Types:**

- `deltachat::Context` - Main context, owns database
- `deltachat::chat::Chat` - Represents a chat
- `deltachat::contact::Contact` - Represents a contact
- `deltachat::message::Message` - Represents a message
- `deltachat::EventType` - Events emitted by DeltaChat

**Key Functions:**

```rust
// Configuration
context.set_config(config::Config::Addr, email).await?;
context.set_config(config::Config::MailPw, password).await?;
context.configure().await?;

// Connection
context.start_io().await;
context.stop_io().await;

// Contacts
let contact_id = Contact::create(&context, name, email).await?;
let contact = Contact::get_by_id(&context, contact_id).await?;
let contacts = Contact::get_all(&context, flags, query).await?;

// Chats
let chat_id = create_by_contact_id(&context, contact_id).await?;
let chat = Chat::load_from_db(&context, chat_id).await?;
let chatlist = Chatlist::try_load(&context, flags, query, last_fetched_id).await?;

// Messages
let msg_id = send_text_msg(&context, chat_id, text).await?;
let msg = Message::load_from_db(&context, msg_id).await?;
let msgs = chat::get_chat_msgs(&context, chat_id).await?;

// Events
let events = context.get_event_emitter();
while let Some(event) = events.recv().await {
    match event.typ {
        EventType::IncomingMsg { chat_id, msg_id } => { ... }
        EventType::ConfigureProgress { progress, comment } => { ... }
        // ...
    }
}
```

---

## Appendix B: File Checklist

**Files to Create:**

- [ ] `mobile/src/viewmodel/deltachat/mod.rs`
- [ ] `mobile/src/viewmodel/deltachat/commands.rs`
- [ ] `mobile/src/viewmodel/deltachat/events.rs`
- [ ] `mobile/src/viewmodel/deltachat/actor.rs`
- [ ] `mobile/src/ui_tabs/deltachat.rs`
- [ ] `mobile/src/viewmodel/deltachat/tests.rs` (integration tests)

**Files to Modify:**

- [ ] `mobile/Cargo.toml`
- [ ] `mobile/src/viewmodel/mod.rs`
- [ ] `mobile/src/ui_tabs/mod.rs`
- [ ] `mobile/src/dure.rs`

**Total:** ~6 new files, ~4 modified files

---

## Appendix C: Estimated Effort

**By Phase:**

| Phase | Description | Estimated Effort |
|-------|-------------|-----------------|
| Phase 1 | Foundation | 2-3 hours |
| Phase 2 | Configuration | 3-4 hours |
| Phase 3 | Contacts | 2-3 hours |
| Phase 4 | Chats | 2-3 hours |
| Phase 5 | Messaging | 4-6 hours |
| Phase 6 | Polish | 3-4 hours |
| **Total** | **MVP Complete** | **16-23 hours** |

**Assumptions:**
- Developer familiar with Rust and egui
- No major blockers in deltachat library integration
- Desktop testing environment available
- Time includes testing and bug fixes

**Post-MVP features:** ~20-40 hours depending on scope.

---

## Conclusion

This design specifies a minimal but complete DeltaChat integration for Dure, following Dure's actor-based MVVM architecture and supporting all target platforms. The runtime bridge handles async runtime differences between DeltaChat (tokio) and Dure (smol), and the phased implementation plan ensures incremental progress with testable milestones.

**Next Steps:**
1. Review and approve this design document
2. Create implementation plan with detailed tasks
3. Begin Phase 1 (Foundation) implementation
4. Test continuously on target platforms

**Design Status:** ✅ Ready for Implementation
