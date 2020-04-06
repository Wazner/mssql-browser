use super::error::{BrowserProtocolError, BrowserProtocolField, BrowserProtocolToken};
use std::net::IpAddr;

/// Information send in a browser protocol response
/// See [SVR_RESP](https://docs.microsoft.com/en-us/openspecs/windows_protocols/mc-sqlr/2e1560c9-5097-4023-9f5e-72b9ff1ec3b1)
#[derive(Debug)]
pub struct InstanceInfo {
    /// The address of the instance
    pub addr: IpAddr,

    /// The name of the server. The SERVERNAME MUST be no greater than 255 bytes.
    pub server_name: String,

    /// A text string that represents the name of the server instance being described.
    /// The INSTANCENAME MUST be no greater than 255 bytes but SHOULD be no greater than 16 MBCS characters.
    pub instance_name: String,

    pub is_clustered: bool,

    /// A text string that conveys the version of the server instance. The VERSION_STRING MUST be no greater than 16 bytes.
    /// VERSION_STRING MUST NOT be empty and MUST appear as follows: VERSION_STRING=1*[0-9"."]
    pub version: String,

    pub np_info: Option<NamedPipeInfo>,
    pub tcp_info: Option<TcpInfo>,
    pub via_info: Option<ViaInfo>,
    pub rpc_info: Option<RpcInfo>,
    pub spx_info: Option<SpxInfo>,
    pub adsp_info: Option<AdspInfo>,
    pub bv_info: Option<BvInfo>,
}

/// Information about the named pipe endpoint
#[derive(Debug)]
pub struct NamedPipeInfo {
    /// A text string that represents the pipe name.
    pub name: String,
}

/// Information about the Tcp endpoint
#[derive(Debug)]
pub struct TcpInfo {
    /// A text string that represents the decimal value of the TCP port that is used to connect to the requested server instance.
    /// TCP_PORT SHOULD be a valid TCP port as specified in \[RFC793\]
    pub port: u16,
}

/// Information about the Virtual Interface Architecture endpoint
#[derive(Debug)]
pub struct ViaInfo {
    /// A text string that MUST be no greater than 15 bytes and that represents the NetBIOS name of a machine where the server resides.
    pub machine_name: String,

    /// The VIA addresses specified
    pub addresses: Vec<ViaAddress>,
}

/// A combination of NIC name and port.
#[derive(Debug)]
pub struct ViaAddress {
    /// A text string that represents the VIA network interface card (NIC) identifier.
    /// VIANIC SHOULD be a valid VIA Adapter NIC number \[VIA2002\].
    pub nic: String,

    /// A text string that represents the decimal value of the VIA NIC's port.
    /// VIAPORT SHOULD be a valid VIA Adapter port number \[VIA2002\].
    pub port: String,
}

/// Contains information about an RPC endpoint
#[derive(Debug)]
pub struct RpcInfo {
    /// The name of the computer to connect to. SHOULD be no more than 127 MBCS characters.
    pub computer_name: String,
}

/// Contains information about an SPX service endpoint
#[derive(Debug)]
pub struct SpxInfo {
    /// The SPX service name of the server.
    /// MUST NOT be greater than 1,024 bytes and SHOULD be no more than 127 MBCS characters.
    pub service_name: String,
}

/// Contains information about an AppleTalk endpoint
#[derive(Debug)]
pub struct AdspInfo {
    /// The AppleTalk service object name. SHOULD be no more than 127 MBCS characters.
    pub object_name: String,
}

/// Contains information about an Banyan VINES endpoint
#[derive(Debug)]
pub struct BvInfo {
    /// The Banyan VINES item name. SHOULD be no more than 127 MBCS characters.
    pub item_name: String,

    /// The Banyan VINES group name. SHOULD be no more than 127 MBCS characters.
    pub group_name: String,

    /// The Banyan VINES organization name. SHOULD be no more than 127 MBCS characters.
    pub org_name: String,
}

/// Contains information about the DAC endpoint of an instance
#[derive(Debug)]
pub struct DacInfo {
    pub port: u16,
}

struct SplitIteratorWithPosition<'a> {
    inner: std::str::Split<'a, char>,
    position: usize,
}

impl<'a> SplitIteratorWithPosition<'a> {
    fn new(inner: std::str::Split<'a, char>) -> SplitIteratorWithPosition<'a> {
        SplitIteratorWithPosition {
            inner: inner,
            position: 0,
        }
    }

    fn string_position(&self) -> usize {
        self.position
    }
}

impl<'a> Iterator for SplitIteratorWithPosition<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        match self.inner.next() {
            Some(x) => {
                self.position += x.len() + 1;
                Some(x)
            }
            None => None,
        }
    }
}

pub(crate) fn parse_instance_info(
    addr: IpAddr,
    string: &str,
) -> Result<(InstanceInfo, usize), BrowserProtocolError> {
    #[inline]
    fn expect_next<'a, T: Iterator<Item = &'a str>>(
        iterator: &mut T,
        identifier: &str,
        field: BrowserProtocolField,
    ) -> Result<(), BrowserProtocolError> {
        iterator
            .next()
            .ok_or_else(|| BrowserProtocolError::UnexpectedToken {
                expected: BrowserProtocolToken::Identifier(field),
                found: BrowserProtocolToken::EndOfMessage,
            })
            .and_then(|x| {
                if x == identifier {
                    Ok(())
                } else {
                    Err(BrowserProtocolError::UnexpectedToken {
                        expected: BrowserProtocolToken::Identifier(field),
                        found: BrowserProtocolToken::Literal(x.to_string()),
                    })
                }
            })
    }

    fn consume_next<'a, T: Iterator<Item = &'a str>>(
        iterator: &mut T,
        value_name: BrowserProtocolField,
    ) -> Result<&'a str, BrowserProtocolError> {
        iterator
            .next()
            .ok_or_else(|| BrowserProtocolError::UnexpectedToken {
                expected: BrowserProtocolToken::ValueOf(value_name),
                found: BrowserProtocolToken::EndOfMessage,
            })
    }

    let mut iterator = SplitIteratorWithPosition::new(string.split(';'));

    // Instance information
    expect_next(
        &mut iterator,
        "ServerName",
        BrowserProtocolField::ServerName,
    )?;
    let server_name = consume_next(&mut iterator, BrowserProtocolField::ServerName)?;
    expect_next(
        &mut iterator,
        "InstanceName",
        BrowserProtocolField::InstanceName,
    )?;
    let instance_name = consume_next(&mut iterator, BrowserProtocolField::InstanceName)?;
    expect_next(
        &mut iterator,
        "IsClustered",
        BrowserProtocolField::IsClustered,
    )?;
    let is_clustered_str = consume_next(&mut iterator, BrowserProtocolField::IsClustered)?;
    let is_clustered = match is_clustered_str {
        "Yes" => true,
        "No" => false,
        v => {
            return Err(BrowserProtocolError::UnexpectedToken {
                expected: BrowserProtocolToken::ValueOf(BrowserProtocolField::IsClustered),
                found: BrowserProtocolToken::Literal(v.to_string()),
            })
        }
    };
    expect_next(&mut iterator, "Version", BrowserProtocolField::Version)?;
    let version = consume_next(&mut iterator, BrowserProtocolField::Version)?;

    // Supported protocols
    let mut np_info: Option<NamedPipeInfo> = None;
    let mut tcp_info: Option<TcpInfo> = None;
    let mut via_info: Option<ViaInfo> = None;
    let mut rpc_info: Option<RpcInfo> = None;
    let mut spx_info: Option<SpxInfo> = None;
    let mut adsp_info: Option<AdspInfo> = None;
    let mut bv_info: Option<BvInfo> = None;

    loop {
        match iterator.next() {
            Some("np") => {
                let pipe_name = consume_next(&mut iterator, BrowserProtocolField::NamedPipeName)?;
                np_info = Some(NamedPipeInfo {
                    name: pipe_name.to_owned(),
                });
            }
            Some("tcp") => {
                let port_str = consume_next(&mut iterator, BrowserProtocolField::TcpPort)?;
                let port: u16 =
                    port_str
                        .parse()
                        .map_err(|_| BrowserProtocolError::UnexpectedToken {
                            expected: BrowserProtocolToken::TcpPort,
                            found: BrowserProtocolToken::Literal(port_str.to_string()),
                        })?;
                tcp_info = Some(TcpInfo { port });
            }
            Some("via") => {
                let parameters = consume_next(&mut iterator, BrowserProtocolField::ViaMachineName)?;
                let comma_idx =
                    parameters
                        .find(',')
                        .ok_or_else(|| BrowserProtocolError::UnexpectedToken {
                            expected: BrowserProtocolToken::ViaParameters,
                            found: BrowserProtocolToken::Literal(parameters.to_string()),
                        })?;
                let machine_name = &parameters[0..comma_idx];
                let mut nic_port_parts = (&parameters[(comma_idx + 1)..]).split(&[',', ':'][..]);
                let mut addresses = Vec::new();
                while let Some(nic) = nic_port_parts.next() {
                    let port = nic_port_parts.next().ok_or_else(|| {
                        BrowserProtocolError::UnexpectedToken {
                            expected: BrowserProtocolToken::ViaParameters,
                            found: BrowserProtocolToken::Literal(parameters.to_string()),
                        }
                    })?;
                    addresses.push(ViaAddress {
                        nic: nic.to_owned(),
                        port: port.to_owned(),
                    });
                }
                via_info = Some(ViaInfo {
                    machine_name: machine_name.to_owned(),
                    addresses,
                });
            }
            Some("rpc") => {
                let computer_name =
                    consume_next(&mut iterator, BrowserProtocolField::RpcComputerName)?;
                rpc_info = Some(RpcInfo {
                    computer_name: computer_name.to_owned(),
                });
            }
            Some("spx") => {
                let service_name =
                    consume_next(&mut iterator, BrowserProtocolField::SpxServiceName)?;
                spx_info = Some(SpxInfo {
                    service_name: service_name.to_owned(),
                });
            }
            Some("adsp") => {
                let object_name =
                    consume_next(&mut iterator, BrowserProtocolField::AppleTalkObjectName)?;
                adsp_info = Some(AdspInfo {
                    object_name: object_name.to_owned(),
                });
            }
            Some("bv") => {
                let item_name = consume_next(&mut iterator, BrowserProtocolField::BvItemName)?;
                let group_name = consume_next(&mut iterator, BrowserProtocolField::BvGroupName)?;
                let org_name = consume_next(&mut iterator, BrowserProtocolField::BvOrgName)?;
                bv_info = Some(BvInfo {
                    item_name: item_name.to_owned(),
                    group_name: group_name.to_owned(),
                    org_name: org_name.to_owned(),
                });
            }
            Some("") => break,
            Some(x) => {
                return Err(BrowserProtocolError::UnexpectedToken {
                    expected: BrowserProtocolToken::EndpointIdentifierOrSemicolon,
                    found: BrowserProtocolToken::Literal(x.to_string()),
                })
            }
            None => {
                return Err(BrowserProtocolError::UnexpectedToken {
                    expected: BrowserProtocolToken::EndpointIdentifierOrSemicolon,
                    found: BrowserProtocolToken::EndOfMessage,
                })
            }
        };
    }

    let consumed = iterator.string_position();

    Ok((
        InstanceInfo {
            addr,
            server_name: server_name.to_owned(),
            instance_name: instance_name.to_owned(),
            is_clustered,
            version: version.to_owned(),
            np_info,
            tcp_info,
            via_info,
            rpc_info,
            spx_info,
            adsp_info,
            bv_info,
        },
        consumed,
    ))
}
