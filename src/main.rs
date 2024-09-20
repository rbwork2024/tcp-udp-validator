use std::io::Write;

use anyhow::anyhow;
use clap::{Parser, Subcommand, ValueEnum};
use tokio::net::UdpSocket;
use tokio::net::{TcpListener, TcpStream};

mod tcp;
mod udp;

const REFRESH_INTERVAL: u64 = 10000;

async fn run_udp_server(
    bind_addr: &str,
    send_addr: &str,
    abort_on_fail: bool,
) -> anyhow::Result<()> {
    let mut socket = UdpSocket::bind(bind_addr).await?;
    let mut prev_success = false;
    let mut success_counter: u64 = 0;
    let mut failure_counter: u64 = 0;
    print!("# successful packets: 0 :: # unsuccessful packets: 0");
    loop {
        let mut update = false;
        if udp::sender_logic(&mut socket, send_addr, abort_on_fail, prev_success).await? {
            if !prev_success {
                prev_success = true;
            }
            success_counter += 1;
            if success_counter % REFRESH_INTERVAL == 0 {
                update = true;
            }
        } else {
            failure_counter += 1;
            update = true;
        }
        if update {
            print!(
                "\rSuccessful: {} | Unsuccessful: {}",
                success_counter, failure_counter
            );
            std::io::stdout().flush().unwrap();
        }
    }
}

async fn run_udp_client(bind_addr: &str, abort_on_fail: bool) -> anyhow::Result<()> {
    let mut socket = UdpSocket::bind(bind_addr).await?;
    let mut prev_success = false;
    let mut success_counter: u64 = 0;
    let mut failure_counter: u64 = 0;
    print!("# successful packets: 0 :: # unsuccessful packets: 0");
    loop {
        let mut update = false;
        if udp::recipient_logic(&mut socket, abort_on_fail, prev_success).await? {
            if !prev_success {
                prev_success = true;
            }
            success_counter += 1;
            if success_counter % REFRESH_INTERVAL == 0 {
                update = true;
            }
        } else {
            failure_counter += 1;
            update = true;
        }
        if update {
            print!(
                "\rSuccessful: {} | Unsuccessful: {}",
                success_counter, failure_counter
            );
            std::io::stdout().flush().unwrap();
        }
    }
}

fn print_and_log(stuff: &str, print: bool) {
    log::info!(
        "[{}] {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        stuff
    );
    if print {
        println!(
            "[{}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            stuff
        );
    }
}

async fn run_server(addr: &str, abort_on_fail: bool, print: bool) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    print_and_log("Waiting for connection to client!", print);
    let (mut socket, _) = listener.accept().await?;
    print_and_log("Connected!", print);
    let mut success_counter: u64 = 0;
    let mut failure_counter: u64 = 0;
    loop {
        let mut update = false;
        match tcp::sender_logic(&mut socket, abort_on_fail).await {
            Ok(result) => {
                if result {
                    success_counter += 1;
                    if success_counter % REFRESH_INTERVAL == 0 {
                        update = true;
                    }
                } else {
                    failure_counter += 1;
                    update = true;
                }
                if update {
                    print_and_log(
                        &format!(
                            "Successful: {} | Unsuccessful: {}",
                            success_counter, failure_counter
                        ),
                        print,
                    );
                }
            }
            Err(e) => {
                print_and_log(
                    &format!("There was a problem with the connection: {}", e),
                    print,
                );
                let mut connected = false;
                while !connected {
                    print_and_log("Attempting to reconnect...", print);
                    if let Ok((s, _)) = listener.accept().await {
                        socket = s;
                        connected = true;
                    }
                }
            }
        }
    }
}

async fn run_client(addr: &str, abort_on_fail: bool, print: bool) -> anyhow::Result<()> {
    print_and_log("Waiting for connection to server!", print);
    let mut socket = TcpStream::connect(addr).await?;
    print_and_log("Connected!", print);
    let mut success_counter: u64 = 0;
    let mut failure_counter: u64 = 0;
    loop {
        let mut update = false;
        match tcp::recipient_logic(&mut socket, abort_on_fail).await {
            Ok(result) => {
                if result {
                    success_counter += 1;
                    if success_counter % REFRESH_INTERVAL == 0 {
                        update = true;
                    }
                } else {
                    failure_counter += 1;
                    update = true;
                }
                if update {
                    print_and_log(
                        &format!(
                            "Successful: {} | Unsuccessful: {}",
                            success_counter, failure_counter
                        ),
                        print,
                    );
                }
            }
            Err(e) => {
                print_and_log(
                    &format!("There was a problem with the connection: {}", e),
                    print,
                );
                let mut connected = false;
                while !connected {
                    print_and_log("Attempting to reconnect...", print);
                    if let Ok(s) = TcpStream::connect(addr).await {
                        socket = s;
                        connected = true;
                    }
                }
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
/// Simple program to validate data sent through TCP or UDP.
///
/// View the source here: https://github.com/rbwork2024/tcp-udp-validator
struct Cli {
    /// Connection type. Use either tcp or udp
    #[command(subcommand)]
    connection_type: ConnectionType,
    /// Abort on failure
    #[arg(long)]
    abort_on_fail: bool,
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
        } => run_server(&address, args.abort_on_fail, args.print).await?,
        ConnectionType::Tcp {
            unit: Unit::Client,
            address,
        } => run_client(&address, args.abort_on_fail, args.print).await?,
        ConnectionType::Udp {
            unit: Unit::Server,
            bind_address,
            send_address,
        } => {
            run_udp_server(
                &bind_address,
                if send_address.is_some() {
                    send_address.as_deref().unwrap()
                } else {
                    return Err(anyhow!(
                        "The UDP server MUST specify a send address to send data to."
                    ));
                },
                args.abort_on_fail,
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
            run_udp_client(&bind_address, args.abort_on_fail).await?
        }
    }
    Ok(())
}
