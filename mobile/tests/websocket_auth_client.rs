//! WebSocket authentication client example
//!
//! This example demonstrates how to connect to the Dure WSS server
//! and authenticate using the auth.login message.
//!
//! Run the server first:
//! ```sh
//! cargo run --bin wss-server -- --domain localhost
//! ```
//!
//! Then run this example:
//! ```sh
//! cargo run --example websocket_auth_client
//! ```

use serde_json::json;
use async_tungstenite::tungstenite::{Message, connect};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("WebSocket Auth Client Example");
    println!("==============================\n");

    // Connect to the WebSocket server
    let url = "ws://localhost:8443/ws";
    println!("Connecting to {}...", url);

    let (mut socket, response) = connect(url)?;
    println!("Connected! Response status: {}", response.status());
    println!("Headers:");
    for (name, value) in response.headers() {
        println!("  {}: {:?}", name, value);
    }
    println!();

    // Create authentication login request
    let auth_request = json!({
        "type": "auth.login",
        "device_id": "example-device-12345",
        "public_key": "example-public-key-abc123",
        "session_id": null,
        "client_version": "dure-client-example/1.0.0"
    });

    println!("Sending auth.login request:");
    println!("{}\n", serde_json::to_string_pretty(&auth_request)?);

    // Send the authentication request
    socket.write_message(Message::Text(auth_request.to_string()))?;

    // Wait for response
    println!("Waiting for auth response...\n");
    let response = socket.read_message()?;

    match response {
        Message::Text(text) => {
            println!("Received auth response:");
            let json: serde_json::Value = serde_json::from_str(&text)?;
            println!("{}\n", serde_json::to_string_pretty(&json)?);

            // Check if authentication was successful
            if let Some(success) = json.get("success") {
                if success.as_bool() == Some(true) {
                    println!("✓ Authentication successful!");

                    if let Some(session_id) = json.get("session_id") {
                        println!("  Session ID: {}", session_id);
                    }
                    if let Some(expires_at) = json.get("expires_at") {
                        println!("  Expires at: {}", expires_at);
                    }

                    // Example: Send a logout request
                    println!("\nSending auth.logout request...");
                    let logout_request = json!({
                        "type": "auth.logout",
                        "session_id": json.get("session_id").unwrap_or(&json!(null))
                    });
                    socket.write_message(Message::Text(logout_request.to_string()))?;

                    // Wait for logout response
                    let logout_response = socket.read_message()?;
                    if let Message::Text(text) = logout_response {
                        println!("Logout response:");
                        let json: serde_json::Value = serde_json::from_str(&text)?;
                        println!("{}\n", serde_json::to_string_pretty(&json)?);
                    }
                } else {
                    println!("✗ Authentication failed!");
                    if let Some(error) = json.get("error") {
                        println!("  Error: {}", error);
                    }
                }
            }
        }
        _ => {
            println!("Received non-text message: {:?}", response);
        }
    }

    // Close the connection
    socket.close(None)?;
    println!("Connection closed.");

    Ok(())
}
