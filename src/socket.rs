use async_trait::async_trait;
use std::net::SocketAddr;

/// A trait used to create `UdpSocket` instances.
#[async_trait]
pub trait UdpSocketFactory: Sized {
    type Socket: UdpSocket;
    type Error: std::error::Error;

    /// Creates a UDP socket from the given address.
    async fn bind(&mut self, addr: &SocketAddr) -> Result<Self::Socket, Self::Error>;
}

/// A generic contract for an UDP socket. Used to be agnostic of the
/// underlying async framework used.
#[async_trait]
pub trait UdpSocket: Sized {
    type Error: std::error::Error;

    /// When enabled, this socket is allowed to send packets to a broadcast address.
    async fn enable_broadcast(&mut self) -> Result<(), Self::Error>;

    /// Connects the UDP socket setting to default destination for send() and limiting packets
    /// that are read via recv from the address specified in `addr`.
    async fn connect(&mut self, addr: &SocketAddr) -> Result<(), Self::Error>;

    /// Sends data on the socket to the remote address to which it is connected.
    /// On success, returns the number of bytes written.
    async fn send(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;

    /// Sends data on the socket to the given address.
    /// On success, returns the number of bytes written.
    async fn send_to(&mut self, buf: &[u8], addr: &SocketAddr) -> Result<usize, Self::Error>;

    /// Receives a single datagram on the socket from the remote address to which it is connected.
    /// On success, returns the number of bytes read.
    async fn recv(&mut self, but: &mut [u8]) -> Result<usize, Self::Error>;

    /// Receives a single datagram on the socket.
    /// On success, returns the number of bytes read and the origin.
    async fn recv_from(&mut self, buf: &mut [u8]) -> Result<(usize, SocketAddr), Self::Error>;
}

#[cfg(feature = "tokio")]
pub type DefaultSocketFactory = TokioSocketFactory;

#[cfg(all(feature = "async-std", not(feature = "tokio")))]
pub type DefaultSocketFactory = AsyncStdSocketFactory;

#[cfg(feature = "tokio")]
pub struct TokioSocketFactory;

#[cfg(feature = "tokio")]
impl TokioSocketFactory {
    pub fn new() -> TokioSocketFactory {
        TokioSocketFactory
    }
}

#[cfg(feature = "tokio")]
#[async_trait]
impl UdpSocketFactory for TokioSocketFactory {
    type Error = tokio::io::Error;
    type Socket = tokio::net::UdpSocket;

    async fn bind(&mut self, addr: &SocketAddr) -> Result<Self::Socket, Self::Error> {
        tokio::net::UdpSocket::bind(addr).await
    }
}

#[cfg(feature = "tokio")]
#[async_trait]
impl UdpSocket for tokio::net::UdpSocket {
    type Error = tokio::io::Error;

    async fn enable_broadcast(&mut self) -> Result<(), Self::Error> {
        self.set_broadcast(true)
    }

    async fn connect(&mut self, addr: &SocketAddr) -> Result<(), Self::Error> {
        self.connect(addr).await
    }

    async fn send(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.send(buf).await
    }

    async fn send_to(&mut self, buf: &[u8], addr: &SocketAddr) -> Result<usize, Self::Error> {
        self.send_to(buf, addr).await
    }

    async fn recv(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.recv(buf).await
    }

    async fn recv_from(&mut self, buf: &mut [u8]) -> Result<(usize, SocketAddr), Self::Error> {
        self.recv_from(buf).await
    }
}

#[cfg(feature = "async-std")]
pub struct AsyncStdSocketFactory;

#[cfg(feature = "async-std")]
impl AsyncStdSocketFactory {
    pub fn new() -> AsyncStdSocketFactory {
        AsyncStdSocketFactory
    }
}

#[cfg(feature = "async-std")]
#[async_trait]
impl UdpSocketFactory for AsyncStdSocketFactory {
    type Error = async_std::io::Error;
    type Socket = async_std::net::UdpSocket;

    async fn bind(&mut self, addr: &SocketAddr) -> Result<Self::Socket, Self::Error> {
        async_std::net::UdpSocket::bind(addr).await
    }
}

#[cfg(feature = "async-std")]
#[async_trait]
impl UdpSocket for async_std::net::UdpSocket {
    type Error = async_std::io::Error;

    async fn enable_broadcast(&mut self) -> Result<(), Self::Error> {
        self.set_broadcast(true)
    }

    async fn connect(&mut self, addr: &SocketAddr) -> Result<(), Self::Error> {
        self.connect(addr).await
    }

    async fn send(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.send(buf).await
    }

    async fn send_to(&mut self, buf: &[u8], addr: &SocketAddr) -> Result<usize, Self::Error> {
        self.send_to(buf, addr).await
    }

    async fn recv(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.recv(buf).await
    }

    async fn recv_from(&mut self, buf: &mut [u8]) -> Result<(usize, SocketAddr), Self::Error> {
        use std::net::ToSocketAddrs;

        match self.recv_from(buf).await {
            Ok((recv_bytes, addr)) => {
                Ok((recv_bytes, addr.to_socket_addrs().unwrap().next().unwrap()))
            }
            Err(x) => Err(x),
        }
    }
}
