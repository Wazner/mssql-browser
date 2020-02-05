# mssql-browser
Rust implementation of the [SQL Server Resolution Protocol](https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-wpo/c67adffd-2740-435d-bda7-dc66fb13f1b7).

> The SQL Server Resolution Protocol enables finding endpoint information of MSSQL servers running in the current network.
>
> The SQL Server Resolution Protocol (SSRP) [[MC-SQLR](https://docs.microsoft.com/en-us/openspecs/windows_protocols/mc-sqlr/1ea6e25f-bff9-4364-ba21-5dc449a601b7)] is a simple application-level protocol for the transfer of requests and responses between clients and database server discovery services. To determine the communication endpoint information of a particular database instance, the client sends a single request to a specific machine and waits for a single response. To enumerate database instances in the network and obtain the endpoint information of each instance, the client broadcasts or multicasts a request to the network and waits for responses from different discovery services on the network.
>
> The SQL Server Resolution Protocol is appropriate for retrieving database endpoint information or for database instance enumeration in scenarios where network or local connectivity is available.

## Usage
To use mssql-browser, first add this to your Cargo.toml:
```toml
[dependencies]
mssql-browser = "0.1"
```

Then you can make the different types and methods available in your module via an use statement:
```rust
use mssql_browser::{ 
  browse, browse_host, browse_instance, browse_instance_dac
};
```

## Examples
Below are a few different ways to get endpoint information of MSSQL server instances.
Check out the docs (TODO link) for a list of fields returned for each found instance.

### Discover endpoint information of instances within network
```rust
use mssql_browser::{ browse, BrowserError };

async fn run() -> Result<(), BrowserError> {
  let broadcast_addr = IpAddr::V4(Ipv4Addr::BROADCAST);
  let mut iterator = browse(broadcast_addr).await?;
  
  while let Some(instance) = iterator.next().await? {
    println!("Found instance {} on host {}.", instance.instance_name, instance.addr);
  }
  
  Ok(())
}
```

### Discover endpoint information of instances on host
```rust
use mssql_browser::{ browse_host, BrowserError };

async fn run() -> Result<(), BrowserError> {
  let host_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
  let mut iterator = browse_host(host_addr).await?;
  
  while let Some(instance) = iterator.next() {
    println!("Found instance {}", instance.instance_name);
  }
  
  Ok(())
}
```

### Discover endpoint information of specific instance
```rust
use mssql_browser::{ browse_instance, BrowserError };

async fn run() -> Result<(), BrowserError> {
  let host_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
  let instance = browse_instance(host_addr, "MSSQLSERVER").await?;
  
  if let Some(tcp) = instance.tcp_info {
    println!("Instance is available via TCP on port {}", np.port);
  }
 
  if let Some(np) = instance.np_info {
    println!("Instance is available via named pipe {}", np.name);
  }
 
  Ok(())
}
```

### Discover DAC endpoint information
```rust
use mssql_browser::{ browse_instance_dac, BrowserError };

async fn run() -> Result<(), BrowserError> {
  let host_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
  let dac_info = browse_instance_dac(host_addr, "MSSQLSERVER").await?;
  
  println!("DAC is exposed on port {}", dac_info.port);
 
  Ok(())
}
```
