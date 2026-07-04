//! WSS actor commands

#[derive(Debug, Clone)]
pub enum WssCommand {
    // Stub commands
    ConnectClient { url: String, auth_token: Option<String> },
    DisconnectClient { connection_id: String },
    SendMessage { connection_id: String, message: Vec<u8> },
}
