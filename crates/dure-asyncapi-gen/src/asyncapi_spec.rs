//! AsyncAPI 3.0 specification for Dure WebSocket protocol
//!
//! This module defines the complete AsyncAPI specification for the Dure
//! distributed e-commerce platform's WebSocket communication protocol.

use asyncapi_rust::AsyncApi;

use dure_messages::{ClientMessage, ServerMessage};

/// Dure WebSocket API Specification
///
/// This specification documents all WebSocket messages and operations
/// used in the Dure distributed e-commerce platform.
///
/// ## Architecture
///
/// - **Client → Server**: Commands and requests
/// - **Server → Client**: Responses and notifications
/// - **Protocol**: WebSocket Secure (WSS)
/// - **Message Format**: JSON with type discriminator
///
/// ## Authentication Flow
///
/// 1. Client connects to WSS endpoint
/// 2. Client sends `auth.login` with device credentials
/// 3. Server responds with `auth.response` containing session ID
/// 4. All subsequent messages include session context
///
/// ## Message Types
///
/// ### Authentication
/// - Login/logout operations
/// - Device identity management
///
/// ### Hosting Operations
/// - DNS, SSL, and server configuration
/// - Multi-provider support (Firebase, Supabase, GCE, etc.)
///
/// ### Member Management
/// - Owner devices and store members
/// - Role-based access control
///
/// ### Channel Operations
/// - Text, voice, and forum channels
/// - Auto-created order channels
///
/// ### Messaging
/// - Real-time chat
/// - File attachments
/// - Reactions and threads
///
/// ### E-commerce
/// - Product catalog management
/// - Order processing
/// - Payment integration (Portone, KakaoPay)
/// - Customer reviews
///
/// ## Usage
///
/// Generate the AsyncAPI specification:
///
/// ```sh
/// cargo run --bin generate-asyncapi
/// ```
///
/// This will create `docs/asyncapi.json` and `docs/asyncapi.yaml`.
#[derive(AsyncApi)]
#[asyncapi(
    title = "Dure WebSocket API",
    version = "1.0.0",
    description = "Distributed e-commerce WebSocket protocol for the Dure platform. \
                   Supports real-time communication between clients, stores, and hosting servers."
)]
#[asyncapi_server(
    name = "production",
    host = "{domain}",
    protocol = "wss",
    pathname = "/api/ws",
    description = "Production WebSocket server for Dure stores",
    variable(
        name = "domain",
        description = "Store domain name (e.g., www.example.com, shop.mystore.com)",
        examples = ["www.dure.com", "shop.example.com", "store.mydomain.com"]
    )
)]
#[asyncapi_server(
    name = "development",
    host = "localhost:8443",
    protocol = "wss",
    pathname = "/api/ws",
    description = "Development WebSocket server for local testing"
)]
#[asyncapi_channel(
    name = "client_commands",
    address = "/api/ws",
    description = "Client sends commands and requests to the server. \
                   Includes authentication, hosting management, member operations, \
                   channel management, messaging, and e-commerce transactions."
)]
#[asyncapi_channel(
    name = "server_responses",
    address = "/api/ws",
    description = "Server sends responses and real-time notifications to clients. \
                   Includes operation results, status updates, and broadcast messages."
)]
#[asyncapi_messages(ClientMessage, ServerMessage)]
pub struct DureApi;

#[cfg(test)]
mod tests {
    use super::*;
    use asyncapi_rust::AsyncApi;

    #[test]
    fn test_generate_spec() {
        let spec = DureApi::asyncapi_spec();

        // Basic validation
        assert_eq!(spec.asyncapi, "3.0.0");
        assert_eq!(spec.info.title, "Dure WebSocket API");
        assert_eq!(spec.info.version, "1.0.0");

        // Check servers
        assert!(spec.servers.is_some());
        let servers = spec.servers.as_ref().unwrap();
        assert!(servers.contains_key("production"));
        assert!(servers.contains_key("development"));

        // Check channels
        assert!(spec.channels.is_some());
        let channels = spec.channels.as_ref().unwrap();
        assert!(channels.contains_key("client_commands"));
        assert!(channels.contains_key("server_responses"));

        // Check components
        assert!(spec.components.is_some());
        let components = spec.components.as_ref().unwrap();
        assert!(components.messages.is_some());

        // Can be serialized
        let json = serde_json::to_string_pretty(&spec).unwrap();
        assert!(!json.is_empty());

        println!("Generated spec: {} bytes", json.len());
    }
}
