//! # mssql-browser
//!
//! mssql-browser is a Rust implementation of the SQL Server Resolution Protocol.
//!
//! >The SQL Server Resolution Protocol enables finding endpoint information of MSSQL servers running in the current network.
//! >
//! > The SQL Server Resolution Protocol (SSRP) [MC-SQLR] is a simple application-level protocol for the transfer of requests and responses between clients and database server discovery services. To determine the communication endpoint information of a particular database instance, the client sends a single request to a specific machine and waits for a single response. To enumerate database instances in the network and obtain the endpoint information of each instance, the client broadcasts or multicasts a request to the network and waits for responses from different discovery services on the network.
//! >
//! > The SQL Server Resolution Protocol is appropriate for retrieving database endpoint information or for database instance enumeration in scenarios where network or local connectivity is available.
//!
//! ## Examples
//! Below are a few different ways to get endpoint information of MSSQL server instances.
//!
//! ### Discover endpoint information of instances within network
//! ```rust
//! use std::net::{ IpAddr, Ipv4Addr };
//! use std::error::Error;
//! use mssql_browser::{ browse, BrowserError };
//!
//! async fn run() -> Result<(), Box<dyn Error>> {
//!   let broadcast_addr = IpAddr::V4(Ipv4Addr::BROADCAST);
//!   let mut iterator = browse(broadcast_addr).await?;
//!   
//!   while let instance = iterator.next().await? {
//!     println!("Found instance {} on host {}.", instance.instance_name, instance.addr);
//!   }
//!   
//!   Ok(())
//! }
//! ```
//!
//! ### Discover endpoint information of instances on host
//! ```rust
//! use std::net::{ IpAddr, Ipv4Addr };
//! use std::error::Error;
//! use mssql_browser::{ browse_host, BrowserError };
//! 
//! async fn run() -> Result<(), Box<dyn Error>> {
//!   let host_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
//!   let mut iterator = browse_host(host_addr).await?;
//!   
//!   while let Some(instance) = iterator.next()? {
//!     println!("Found instance {}", instance.instance_name);
//!   }
//!   
//!   Ok(())
//! }
//! ```
//!
//! ### Discover endpoint information of specific instance
//! ```rust
//! use std::net::{ IpAddr, Ipv4Addr };
//! use std::error::Error;
//! use mssql_browser::{ browse_instance, BrowserError };
//!
//! async fn run() -> Result<(), Box<dyn Error>> {
//!   let host_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
//!   let instance = browse_instance(host_addr, "MSSQLSERVER").await?;
//!   
//!   if let Some(tcp) = instance.tcp_info {
//!     println!("Instance is available via TCP on port {}", tcp.port);
//!   }
//!  
//!   if let Some(np) = instance.np_info {
//!     println!("Instance is available via named pipe {}", np.name);
//!   }
//!  
//!   Ok(())
//! }
//! ```
//!
//! ### Discover DAC endpoint information
//! ```rust
//! use std::net::{ IpAddr, Ipv4Addr };
//! use std::error::Error;
//! use mssql_browser::{ browse_instance_dac, BrowserError };
//!
//! async fn run() -> Result<(), Box<dyn Error>> {
//!   let host_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
//!   let dac_info = browse_instance_dac(host_addr, "MSSQLSERVER").await?;
//!   
//!   println!("DAC is exposed on port {}", dac_info.port);
//!  
//!   Ok(())
//! }
//! ```

mod error;
mod info;
mod socket;

mod browse;
mod browse_host;
mod browse_instance;
mod browse_instance_dac;

/// Maximum length of an instance name
pub const MAX_INSTANCE_NAME_LEN: usize = 32;

pub use error::*;
pub use info::*;

#[cfg(any(feature = "tokio", feature = "async-std"))]
pub use browse::browse;
pub use browse::AsyncInstanceIterator;
#[cfg(any(feature = "tokio", feature = "async-std"))]
pub use browse_host::browse_host;
pub use browse_host::InstanceIterator;
#[cfg(any(feature = "tokio", feature = "async-std"))]
pub use browse_instance::browse_instance;
#[cfg(any(feature = "tokio", feature = "async-std"))]
pub use browse_instance_dac::browse_instance_dac;

/// Types and functions related to using a custom socket implementation
pub mod custom_socket {
    pub use super::browse::browse_inner as browse;
    pub use super::browse_host::browse_host_inner as browse_host;
    pub use super::browse_instance::browse_instance_inner as browse_instance;
    pub use super::browse_instance_dac::browse_instance_dac_inner as browse_instance_dac;
    pub use super::socket::*;
}
