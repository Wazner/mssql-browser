use mssql_browser;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::time::timeout;

const TIMEOUT: Duration = Duration::from_secs(1);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    args.next().unwrap();
    let mode = args.next().unwrap_or(String::from("broadcast"));

    match &mode[..] {
        "broadcast" => {
            let multicast = args
                .next()
                .map(|ip| ip.parse().unwrap())
                .unwrap_or(IpAddr::V4(Ipv4Addr::BROADCAST));
            println!("Listening...");

            let mut iterator = mssql_browser::browse(multicast).await?;

            loop {
                match timeout(TIMEOUT, iterator.next()).await {
                    // Found an instance
                    Ok(res) => println!("Got response: {:#?}", res?),

                    // Timeout expired
                    Err(_) => {
                        println!("Timeout expired");
                        break;
                    }
                }
            }
        }
        "host" => {
            let remote_addr = args
                .next()
                .map(|ip| ip.parse().unwrap())
                .unwrap_or(IpAddr::V4(Ipv4Addr::BROADCAST));
            println!("Listening...");

            match timeout(TIMEOUT, mssql_browser::browse_host(remote_addr)).await {
                // Found one or more instances
                Ok(iterator) => {
                    let mut iterator = iterator?;

                    while let Some(res) = iterator.next()? {
                        println!("Got response: {:#?}", res);
                    }
                }

                // Timeout expired
                Err(_) => println!("Timeout expired"),
            }
        }
        "instance" => {
            let remote_addr = args
                .next()
                .map(|ip| ip.parse().unwrap())
                .unwrap_or(IpAddr::V4(Ipv4Addr::BROADCAST));
            let instance_name = args.next().unwrap_or(String::from("MSSQLSERVER"));

            println!("Listening...");

            match timeout(
                TIMEOUT,
                mssql_browser::browse_instance(remote_addr, &instance_name),
            )
            .await
            {
                // Found an instance
                Ok(res) => println!("Got response: {:#?}", res?),

                // Timeout expired
                Err(_) => println!("Timeout expired"),
            }
        }
        "instance_dac" => {
            let remote_addr = args
                .next()
                .map(|ip| ip.parse().unwrap())
                .unwrap_or(IpAddr::V4(Ipv4Addr::BROADCAST));
            let instance_name = args.next().unwrap_or(String::from("MSSQLSERVER"));

            println!("Listening...");

            match timeout(
                TIMEOUT,
                mssql_browser::browse_instance_dac(remote_addr, &instance_name),
            )
            .await
            {
                // Found an instance
                Ok(res) => println!("Got response: {:#?}", res?),

                // Timeout expired
                Err(_) => println!("Timeout expired"),
            }
        }
        m => println!("Invalid mode {:?}", m),
    }

    Ok(())
}
