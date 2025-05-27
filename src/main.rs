use anyhow::anyhow;
use clap::{Parser, Subcommand, ValueEnum};

mod tcp;
mod udp;
pub(crate) mod util;

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
/// Simple program to validate data sent through TCP or UDP.
///
/// View the source here: https://github.com/rbwork2024/tcp-udp-validator
struct Cli {
    /// Connection type. Use either tcp or udp
    #[command(subcommand)]
    connection_type: ConnectionType,
    /// In addition to logging, print
    #[arg(short)]
    print: bool,
}

#[derive(Clone, Debug, Subcommand)]
enum ConnectionType {
    Tcp {
        /// Whether to run as server or client (TCP)
        unit: Unit,
        /// Bind address for the server, and connection address for the client
        /// Example(server): 0.0.0.0:8080, Example(client): 127.0.0.1:8080
        address: String,
    },
    Udp {
        /// Whether to run as server or client (UDP)
        unit: Unit,
        /// Bind address for the server/client
        /// Example(server): 0.0.0.0:8080, Example(client): 0.0.0.0:8081
        bind_address: String,
        /// Send address for the server. Will be unused for client
        /// Example(server): 127.0.0.1:8081
        send_address: Option<String>,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum Unit {
    Server,
    Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let args = Cli::parse();
    match args.connection_type {
        ConnectionType::Tcp {
            unit: Unit::Server,
            address,
        } => tcp::run_server(&address, args.print).await?,
        ConnectionType::Tcp {
            unit: Unit::Client,
            address,
        } => tcp::run_client(&address, args.print).await?,
        ConnectionType::Udp {
            unit: Unit::Server,
            bind_address,
            send_address,
        } => {
            udp::run_udp_server(
                &bind_address,
                if send_address.is_some() {
                    send_address.as_deref().unwrap()
                } else {
                    return Err(anyhow!(
                        "The UDP server MUST specify a send address to send data to."
                    ));
                },
                args.print,
            )
            .await?
        }
        ConnectionType::Udp {
            unit: Unit::Client,
            bind_address,
            send_address,
        } => {
            if send_address.is_some() {
                println!(
                    "[{}] As a UDP client, send_address will be ignored!",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                );
            }
            udp::run_udp_client(&bind_address, args.print).await?
        }
    }
    Ok(())
}
