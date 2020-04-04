use tokio;
use std::error::Error;
use std::net::{ IpAddr, Ipv4Addr };
use mssql_browser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    args.next().unwrap();
    let mode = args.next().unwrap_or(String::from("broadcast"));

    match &mode[..] {
        "broadcast" => {
            let multicast = args.next().map(|ip| ip.parse().unwrap()).unwrap_or(IpAddr::V4(Ipv4Addr::BROADCAST));
            println!("Listening...");

            let mut iterator = mssql_browser::browse(multicast)
                .await?;
            
            while let Some(x) = iterator.next().await? {
                println!("Got response: {:#?}", x);
            }
            println!("Finished");
        },
        "host" => {
            let remote_addr = args.next().map(|ip| ip.parse().unwrap()).unwrap_or(IpAddr::V4(Ipv4Addr::BROADCAST));
            println!("Listening...");

            let mut iterator = mssql_browser::browse_host(remote_addr).await?;
            while let Some(x) = iterator.next()? {
                println!("Got response: {:#?}", x);
            }
            println!("Finished");
        },
        "instance" => {
            let remote_addr = args.next().map(|ip| ip.parse().unwrap()).unwrap_or(IpAddr::V4(Ipv4Addr::BROADCAST));
            let instance_name = args.next().unwrap_or(String::from("MSSQLSERVER"));

            println!("Listening...");

            let x = mssql_browser::browse_instance(remote_addr, &instance_name).await?;
            println!("Got response: {:#?}", x);
            
            println!("Finished");
        },
        "instance_dac" => {
            let remote_addr = args.next().map(|ip| ip.parse().unwrap()).unwrap_or(IpAddr::V4(Ipv4Addr::BROADCAST));
            let instance_name = args.next().unwrap_or(String::from("MSSQLSERVER"));

            println!("Listening...");

            let x = mssql_browser::browse_instance_dac(remote_addr, &instance_name).await?;
            println!("Got response: {:#?}", x);
            
            println!("Finished");
        },
        m => println!("Invalid mode {:?}", m)
    }

    Ok(())
}