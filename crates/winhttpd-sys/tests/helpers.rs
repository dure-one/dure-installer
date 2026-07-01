use std::net::TcpListener;

/// Find a free port on localhost
pub fn find_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to find free port")
        .local_addr()
        .expect("Failed to get local addr")
        .port()
}
