//! OpenBSD stub implementation of netwatch
//!
//! This is a compatibility shim that provides the netwatch API but with no
//! actual network monitoring functionality on OpenBSD. It allows DeltaChat
//! and iroh to compile and run on OpenBSD, but without automatic network
//! change detection.
//!
//! Users will need to manually reconnect if their network changes (e.g.,
//! switching from WiFi to mobile, VPN connect/disconnect).

use std::net::IpAddr;
use futures::stream::{Stream, StreamExt};

/// UdpSocket wrapper for API compatibility (stub)
#[derive(Debug)]
pub struct UdpSocket(std::net::UdpSocket);

impl UdpSocket {
    pub fn bind_full(addr: std::net::SocketAddr) -> std::io::Result<Self> {
        std::net::UdpSocket::bind(addr).map(UdpSocket)
    }

    pub fn bind(addr: std::net::SocketAddr) -> std::io::Result<Self> {
        std::net::UdpSocket::bind(addr).map(UdpSocket)
    }

    // Delegate to inner UdpSocket
    pub fn send_to(&self, buf: &[u8], addr: std::net::SocketAddr) -> std::io::Result<usize> {
        self.0.send_to(buf, addr)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, std::net::SocketAddr)> {
        self.0.recv_from(buf)
    }

    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.0.local_addr()
    }

    pub fn set_read_timeout(&self, dur: Option<std::time::Duration>) -> std::io::Result<()> {
        self.0.set_read_timeout(dur)
    }

    pub fn set_write_timeout(&self, dur: Option<std::time::Duration>) -> std::io::Result<()> {
        self.0.set_write_timeout(dur)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> std::io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }

    pub fn connect(&self, addr: std::net::SocketAddr) -> std::io::Result<()> {
        self.0.connect(addr)
    }

    pub fn send(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.send(buf)
    }

    pub fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.recv(buf)
    }
}

/// Stub interfaces module
pub mod interfaces {
    use std::net::IpAddr;

    /// List network interfaces (stub - returns empty list on OpenBSD)
    pub fn list() -> Vec<Interface> {
        Vec::new()
    }

    /// Network interface information (stub)
    #[derive(Debug, Clone)]
    pub struct Interface {
        pub name: String,
        pub addrs: Vec<IpAddr>,
    }

    /// Home router information (stub)
    #[derive(Debug, Clone)]
    pub struct HomeRouter {
        pub gateway: IpAddr,
        pub my_ip: IpAddr,
    }

    impl HomeRouter {
        pub fn new(gateway: IpAddr, my_ip: IpAddr) -> Self {
            HomeRouter { gateway, my_ip }
        }
    }

    /// Get home router (stub - returns None on OpenBSD)
    pub async fn home_router() -> Option<HomeRouter> {
        None
    }
}

/// Event types for IP/network changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpEvent {
    /// A new IP address was added
    NewAddr(IpAddr),
    /// An IP address was removed
    DelAddr(IpAddr),
    /// Network interface went up
    InterfaceUp(String),
    /// Network interface went down
    InterfaceDown(String),
}

/// Watch for IP address changes (stub - never yields events on OpenBSD)
pub async fn watch() -> impl Stream<Item = IpEvent> {
    log::warn!("netwatch: Using OpenBSD stub - network change detection disabled");
    log::info!("netwatch: Manual reconnection required after network changes");

    // Return an empty stream that never yields any events
    // This allows the code to compile and run, but won't detect network changes
    futures::stream::empty()
}

/// List current IP addresses
pub async fn list() -> Vec<IpAddr> {
    // Return empty list - we don't monitor interfaces on OpenBSD
    // The application will use other methods to determine connectivity
    Vec::new()
}

/// Check if we're currently online (stub - always returns true)
pub async fn is_online() -> bool {
    // Optimistically assume we're online
    // Applications should implement their own connectivity checks
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_watch_returns_empty_stream() {
        let mut stream = watch().await;

        // Stream should be empty
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_list_returns_empty() {
        let addrs = list().await;
        assert!(addrs.is_empty());
    }

    #[tokio::test]
    async fn test_is_online() {
        assert!(is_online().await);
    }
}
