use super::error::*;
use super::info::*;
use super::socket::{UdpSocket, UdpSocketFactory};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// The CLNT_UCAST_INST packet is a request for information related to a specific instance.
const CLNT_UCAST_INST: u8 = 0x04;

/// The server responds to all client requests with an SVR_RESP.
const SVR_RESP: u8 = 0x05;

/// Gets information about the given instance.
///
/// # Arguments
/// * `remote_addr` - The address of the remote host on which the instance is running.
/// * `instance_name` - The name of the instance, must be less than `MAX_INSTANCE_NAME_LEN` characters.
#[cfg(any(feature = "tokio", feature = "async-std"))]
pub async fn browse_instance(
    remote_addr: IpAddr,
    instance_name: &str,
) -> Result<
    InstanceInfo,
    BrowserError<
        <super::socket::DefaultSocketFactory as UdpSocketFactory>::Error,
        <<super::socket::DefaultSocketFactory as UdpSocketFactory>::Socket as UdpSocket>::Error,
    >,
> {
    let mut factory = super::socket::DefaultSocketFactory::new();
    browse_instance_inner(remote_addr, instance_name, &mut factory).await
}

/// Gets information about the given instance.
///
/// # Arguments
/// * `remote_addr` - The address of the remote host on which the instance is running.
/// * `instance_name` - The name of the instance, must be less than `MAX_INSTANCE_NAME_LEN` characters.
pub async fn browse_instance_inner<SF: UdpSocketFactory>(
    remote_addr: IpAddr,
    instance_name: &str,
    socket_factory: &mut SF,
) -> Result<InstanceInfo, BrowserError<SF::Error, <SF::Socket as UdpSocket>::Error>> {
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

    let mut buffer = [0u8; 1 + super::MAX_INSTANCE_NAME_LEN + 1];
    buffer[0] = CLNT_UCAST_INST;
    buffer[1..(1 + instance_name.len())].copy_from_slice(instance_name.as_bytes()); // TODO: Encode as mbcs string
    let buffer_len = 2 + instance_name.len();
    socket
        .send_to(&buffer[0..buffer_len], &remote)
        .await
        .map_err(|e| BrowserError::SendFailed(remote, e))?;

    let mut buffer = [0u8; 3 + 1024];

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

    let resp_data_len = u16::from_le_bytes([buffer[1], buffer[2]]);
    if resp_data_len as usize != bytes_received - 3 {
        return Err(BrowserError::ProtocolError(
            BrowserProtocolError::LengthMismatch {
                datagram: bytes_received,
                header: (resp_data_len + 3) as usize,
            },
        ));
    }

    // TODO: Decode mbcs string
    let as_str = std::str::from_utf8(&buffer[3..bytes_received]).unwrap();
    let (instance, consumed) =
        parse_instance_info(remote_addr, &as_str).map_err(|e| BrowserError::ProtocolError(e))?;

    if consumed != as_str.len() {
        return Err(BrowserError::ProtocolError(
            BrowserProtocolError::ExtraneousData(Vec::from(&buffer[(3 + consumed)..])),
        ));
    }

    Ok(instance)
}
