use super::error::*;
use super::info::*;
use super::socket::{UdpSocket, UdpSocketFactory};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// The CLNT_UCAST_DAC packet request is used to determine the TCP [RFC793] port on which the
/// Microsoft SQL Server dedicated administrator connection (DAC) endpoint is listening.
const CLNT_UCAST_DAC: u8 = 0x0F;

/// The server responds to all client requests with an SVR_RESP.
const SVR_RESP: u8 = 0x05;

/// Gets DAC information about the given instance
///
/// # Arguments
/// * `remote_addr` - The address of the remote host on which the instance is running.
/// * `instance_name` - The name of the instance, must be less than `MAX_INSTANCE_NAME_LEN` characters.
#[cfg(any(feature = "tokio", feature = "async-std"))]
pub async fn browse_instance_dac(
    remote_addr: IpAddr,
    instance_name: &str,
) -> Result<
    DacInfo,
    BrowserError<
        <super::socket::DefaultSocketFactory as UdpSocketFactory>::Error,
        <<super::socket::DefaultSocketFactory as UdpSocketFactory>::Socket as UdpSocket>::Error,
    >,
> {
    let mut factory = super::socket::DefaultSocketFactory::new();
    browse_instance_dac_inner(remote_addr, instance_name, &mut factory).await
}

/// Gets DAC information about the given instance
///
/// # Arguments
/// * `remote_addr` - The address of the remote host on which the instance is running.
/// * `instance_name` - The name of the instance, must be less than `MAX_INSTANCE_NAME_LEN` characters.
pub async fn browse_instance_dac_inner<SF: UdpSocketFactory>(
    remote_addr: IpAddr,
    instance_name: &str,
    socket_factory: &mut SF,
) -> Result<DacInfo, BrowserError<SF::Error, <SF::Socket as UdpSocket>::Error>> {
    const VERSION: u8 = 0x01;

    if instance_name.len() > super::MAX_INSTANCE_NAME_LEN {
        return Err(BrowserError::InstanceNameTooLong);
    }

    let local_addr = if remote_addr.is_ipv4() {
        IpAddr::V4(Ipv4Addr::UNSPECIFIED)
    } else {
        IpAddr::V6(Ipv6Addr::UNSPECIFIED)
    };

    let bind_to = SocketAddr::new(local_addr, 0);
    let mut socket = socket_factory
        .bind(&bind_to)
        .await
        .map_err(BrowserError::BindFailed)?;

    let remote = SocketAddr::new(remote_addr, 1434);
    socket
        .connect(&remote)
        .await
        .map_err(|e| BrowserError::ConnectFailed(remote, e))?;

    let mut buffer = [0u8; 2 + super::MAX_INSTANCE_NAME_LEN + 1];
    buffer[0] = CLNT_UCAST_DAC;
    buffer[1] = VERSION;
    buffer[2..(2 + instance_name.len())].copy_from_slice(instance_name.as_bytes()); // TODO: Encode as mbcs string
    let buffer_len = 3 + instance_name.len();
    socket
        .send(&buffer[0..buffer_len])
        .await
        .map_err(|e| BrowserError::SendFailed(remote, e))?;

    let mut buffer = [0u8; 6];

    let bytes_received = socket
        .recv(&mut buffer)
        .await
        .map_err(BrowserError::ReceiveFailed)?;

    if bytes_received < 1 {
        return Err(BrowserError::ProtocolError(
            BrowserProtocolError::UnexpectedToken {
                expected: BrowserProtocolToken::MessageIdentifier(SVR_RESP),
                found: BrowserProtocolToken::EndOfMessage,
            },
        ));
    }

    if buffer[0] != SVR_RESP {
        return Err(BrowserError::ProtocolError(
            BrowserProtocolError::UnexpectedToken {
                expected: BrowserProtocolToken::MessageIdentifier(SVR_RESP),
                found: BrowserProtocolToken::MessageIdentifier(buffer[0]),
            },
        ));
    }

    if bytes_received < 3 {
        return Err(BrowserError::ProtocolError(
            BrowserProtocolError::UnexpectedToken {
                expected: BrowserProtocolToken::MessageLength,
                found: BrowserProtocolToken::EndOfMessage,
            },
        ));
    }

    let packet_size = u16::from_le_bytes([buffer[1], buffer[2]]) as usize;
    if packet_size != buffer.len() {
        return Err(BrowserError::ProtocolError(
            BrowserProtocolError::LengthMismatch {
                datagram: bytes_received,
                header: packet_size,
            },
        ));
    }

    if bytes_received < 4 {
        return Err(BrowserError::ProtocolError(
            BrowserProtocolError::UnexpectedToken {
                expected: BrowserProtocolToken::DacVersion(VERSION),
                found: BrowserProtocolToken::EndOfMessage,
            },
        ));
    }

    if buffer[3] != VERSION {
        return Err(BrowserError::ProtocolError(
            BrowserProtocolError::UnexpectedToken {
                expected: BrowserProtocolToken::DacVersion(VERSION),
                found: BrowserProtocolToken::DacVersion(buffer[3]),
            },
        ));
    }

    if bytes_received < 6 {
        return Err(BrowserError::ProtocolError(
            BrowserProtocolError::UnexpectedToken {
                expected: BrowserProtocolToken::DacPort,
                found: BrowserProtocolToken::EndOfMessage,
            },
        ));
    }

    let port = u16::from_le_bytes([buffer[4], buffer[5]]);
    return Ok(DacInfo { port });
}
