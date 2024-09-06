use clap::{Parser, Subcommand, ValueEnum};
use env_logger::Env;
use tokio::net::UdpSocket;
use tokio::net::{TcpListener, TcpStream};

mod tcp;
mod udp;

async fn run_udp_server(
    bind_addr: &str,
    send_addr: &str,
    abort_on_fail: bool,
) -> anyhow::Result<()> {
    let mut socket = UdpSocket::bind(bind_addr).await?;
    socket.connect(send_addr).await?;
    log::info!("UDP connected!");
    loop {
        udp::sender_logic(&mut socket, send_addr, abort_on_fail).await?;
        udp::recipient_logic(&mut socket, send_addr, abort_on_fail).await?;
    }
}

async fn run_udp_client(
    bind_addr: &str,
    send_addr: &str,
    abort_on_fail: bool,
) -> anyhow::Result<()> {
    let mut socket = UdpSocket::bind(bind_addr).await?;
    socket.connect(send_addr).await?;
    log::info!("UDP connected!");
    loop {
        udp::recipient_logic(&mut socket, send_addr, abort_on_fail).await?;
        udp::sender_logic(&mut socket, send_addr, abort_on_fail).await?;
    }
}

async fn run_server(addr: &str, abort_on_fail: bool) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let (mut socket, _) = listener.accept().await?;
    log::info!("TCP: Client connected!");
    loop {
        tcp::sender_logic(&mut socket, abort_on_fail).await?;
        tcp::recipient_logic(&mut socket, abort_on_fail).await?;
    }
}

async fn run_client(addr: &str, abort_on_fail: bool) -> anyhow::Result<()> {
    let mut socket = TcpStream::connect(addr).await?;
    log::info!("TCP: Connected to server!");
    loop {
        tcp::recipient_logic(&mut socket, abort_on_fail).await?;
        tcp::sender_logic(&mut socket, abort_on_fail).await?;
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Simple program to validate data sent through TCP or UDP
struct Cli {
    /// Connection type. Use either tcp or udp
    #[command(subcommand)]
    connection_type: ConnectionType,
    /// Define a log level (default=Warn)
    #[arg(long)]
    log_level: Option<LogLevel>,
    /// Abort on failure
    #[arg(long)]
    abort_on_fail: bool,
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
        /// Send address for the server/client
        /// Example(server): 127.0.0.1:8081, Example(client): 127.0.0.1:8080
        connection_address: String,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum LogLevel {
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn to_str<'a>(&self) -> &'a str {
        match self {
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
enum Unit {
    Server,
    Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    env_logger::Builder::from_env(
        Env::default().default_filter_or(args.log_level.map_or("warn", |level| level.to_str())),
    )
    .init();
    log::warn!("Selected a log level where you can see warnings!");
    log::info!("Selected a log level where you can see info!");
    log::debug!("Selected a log level where you can see debug!");
    log::trace!("Selected a log level where you can see trace!");
    log::info!("Running program!");
    match args.connection_type {
        ConnectionType::Tcp {
            unit: Unit::Server,
            address,
        } => run_server(&address, args.abort_on_fail).await?,
        ConnectionType::Tcp {
            unit: Unit::Client,
            address,
        } => run_client(&address, args.abort_on_fail).await?,
        ConnectionType::Udp {
            unit: Unit::Server,
            bind_address,
            connection_address,
        } => run_udp_server(&bind_address, &connection_address, args.abort_on_fail).await?,
        ConnectionType::Udp {
            unit: Unit::Client,
            bind_address,
            connection_address,
        } => run_udp_client(&bind_address, &connection_address, args.abort_on_fail).await?,
    }
    Ok(())
}
