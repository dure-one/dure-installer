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
use futures::stream::Stream;

/// UdpSocket wrapper for API compatibility (stub)
#[derive(Debug)]
pub struct UdpSocket(std::net::UdpSocket);

impl UdpSocket {
    pub fn bind_full<A: Into<std::net::SocketAddr>>(addr: A) -> std::io::Result<Self> {
        let socket = std::net::UdpSocket::bind(addr.into())?;
        socket.set_nonblocking(true)?;
        Ok(UdpSocket(socket))
    }

    pub fn bind(family: IpFamily, port: u16) -> std::io::Result<Self> {
        match family {
            IpFamily::V4 => Self::bind_v4(port),
            IpFamily::V6 => Self::bind_local_v6(port),
        }
    }

    pub fn bind_local(family: IpFamily, port: u16) -> std::io::Result<Self> {
        Self::bind(family, port)
    }

    pub fn bind_v4(port: u16) -> std::io::Result<Self> {
        let socket = std::net::UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, port))?;
        socket.set_nonblocking(true)?;
        Ok(UdpSocket(socket))
    }

    pub fn bind_local_v4(port: u16) -> std::io::Result<Self> {
        Self::bind_v4(port)
    }

    pub fn bind_local_v6(port: u16) -> std::io::Result<Self> {
        let socket = std::net::UdpSocket::bind((std::net::Ipv6Addr::UNSPECIFIED, port))?;
        socket.set_nonblocking(true)?;
        Ok(UdpSocket(socket))
    }

    pub fn as_socket_ref(&self) -> &std::net::UdpSocket {
        &self.0
    }

    pub fn rebind(&self) -> std::io::Result<()> {
        // Stub - rebinding not needed on OpenBSD stub
        Ok(())
    }

    pub fn poll_writable(&self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
        // Stub - always ready for writing
        std::task::Poll::Ready(Ok(()))
    }

    // Delegate to inner UdpSocket
    pub async fn send_to(&self, buf: &[u8], addr: std::net::SocketAddr) -> std::io::Result<usize> {
        self.0.send_to(buf, addr)
    }

    pub async fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, std::net::SocketAddr)> {
        self.0.recv_from(buf)
    }

    pub fn try_send_quinn(&self, _transmit: &iroh_quinn_udp::Transmit) -> std::io::Result<()> {
        // Stub - QUIC transmit not implemented on OpenBSD
        Ok(())
    }

    pub fn poll_recv_quinn(
        &self,
        _cx: &mut std::task::Context<'_>,
        _bufs: &mut [std::io::IoSliceMut<'_>],
        _meta: &mut [iroh_quinn_udp::RecvMeta],
    ) -> std::task::Poll<std::io::Result<usize>> {
        // Stub - QUIC receive not implemented on OpenBSD
        std::task::Poll::Ready(Ok(0))
    }

    pub fn may_fragment(&self) -> bool {
        // Stub - fragmentation not supported on OpenBSD stub
        false
    }

    pub fn max_gso_segments(&self) -> usize {
        // Stub - GSO (Generic Segmentation Offload) not supported
        1
    }

    pub fn gro_segments(&self) -> usize {
        // Stub - GRO (Generic Receive Offload) not supported
        1
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

    pub async fn send(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.send(buf)
    }

    pub async fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.recv(buf)
    }
}

/// Stub interfaces module
pub mod interfaces {
    use std::net::IpAddr;
    use std::collections::HashMap;

    /// List network interfaces (stub - returns empty list on OpenBSD)
    pub fn list() -> Vec<Interface> {
        Vec::new()
    }

    /// Network interface information (stub)
    #[derive(Debug, Clone)]
    pub struct Interface {
        pub name: String,
        addrs: Vec<ipnet::IpNet>,
    }

    impl Interface {
        pub fn addrs(&self) -> impl Iterator<Item = &ipnet::IpNet> {
            self.addrs.iter()
        }
    }

    /// Home router information (stub)
    #[derive(Debug, Clone)]
    pub struct HomeRouter {
        pub gateway: IpAddr,
        pub my_ip: Option<IpAddr>,
    }

    impl HomeRouter {
        /// Synchronous constructor for compatibility (stub - returns None on OpenBSD)
        pub fn new() -> Option<Self> {
            None
        }
    }

    /// Get home router (stub - returns None on OpenBSD)
    pub async fn home_router() -> Option<HomeRouter> {
        None
    }

    /// Network interface state snapshot (stub)
    #[derive(Debug, Clone)]
    pub struct State {
        pub interfaces: HashMap<String, Interface>,
        pub have_v4: bool,
        pub have_v6: bool,
    }

    impl State {
        pub async fn new() -> Self {
            State {
                interfaces: HashMap::new(),
                have_v4: false,
                have_v6: false,
            }
        }

        pub fn interfaces(&self) -> impl Iterator<Item = &Interface> {
            self.interfaces.values()
        }
    }

    impl std::fmt::Display for State {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "State(interfaces: {}, v4: {}, v6: {})",
                   self.interfaces.len(), self.have_v4, self.have_v6)
        }
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

/// IP address family enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpFamily {
    V4,
    V6,
}

/// IP module with address utilities
pub mod ip {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    /// Local addresses snapshot (stub)
    #[derive(Debug, Clone)]
    pub struct LocalAddresses {
        pub regular: Vec<IpAddr>,
        pub loopback: Vec<IpAddr>,
    }

    impl LocalAddresses {
        pub fn new() -> Self {
            LocalAddresses {
                regular: Vec::new(),
                loopback: Vec::new(),
            }
        }

        pub fn iter(&self) -> impl Iterator<Item = &IpAddr> {
            self.regular.iter().chain(self.loopback.iter())
        }
    }

    impl Default for LocalAddresses {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Check if an IP is a unicast link-local address
    pub fn is_unicast_link_local(ip: impl Into<IpAddr>) -> bool {
        match ip.into() {
            IpAddr::V4(ipv4) => is_unicast_link_local_v4(ipv4),
            IpAddr::V6(ipv6) => is_unicast_link_local_v6(ipv6),
        }
    }

    fn is_unicast_link_local_v4(ipv4: Ipv4Addr) -> bool {
        // Link-local range: 169.254.0.0/16
        ipv4.octets()[0] == 169 && ipv4.octets()[1] == 254
    }

    fn is_unicast_link_local_v6(ipv6: Ipv6Addr) -> bool {
        // Link-local range: fe80::/10
        ipv6.segments()[0] & 0xffc0 == 0xfe80
    }
}

/// Network monitor module
pub mod netmon {
    use super::IpEvent;
    use futures::stream::Stream;

    /// Network monitor handle (stub)
    #[derive(Debug, Clone)]
    pub struct Monitor;

    impl Monitor {
        pub async fn new() -> std::io::Result<Self> {
            Ok(Monitor)
        }

        pub fn events(&self) -> impl Stream<Item = IpEvent> {
            futures::stream::empty()
        }

        pub async fn subscribe<F>(&self, mut _callback: F) -> std::io::Result<SubscriptionToken>
        where
            F: FnMut(bool) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + 'static,
        {
            Ok(SubscriptionToken)
        }

        pub async fn network_change(&self) -> std::io::Result<()> {
            // Stub - no network change detection on OpenBSD
            Ok(())
        }
    }

    /// Subscription token for network monitoring (stub)
    #[derive(Debug)]
    pub struct SubscriptionToken;

    impl Drop for SubscriptionToken {
        fn drop(&mut self) {
            // Stub - nothing to clean up
        }
    }
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
