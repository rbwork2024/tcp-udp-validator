use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use env_logger::Env;
use rand::Rng;
use sha2::{Digest, Sha256}; // For data integrity check
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn sender_logic(socket: &mut TcpStream, abort_on_fail: bool) -> anyhow::Result<()> {
    let mut data = [0u8; 1024];
    rand::thread_rng().fill(&mut data);
    // Calculate the checksum using SHA256
    let mut hasher = Sha256::new();
    hasher.update(data);
    let checksum: &[u8] = &hasher.finalize();

    log::trace!("Checksum: {:?}", hex::encode(checksum));

    log::debug!(
        "Sending {} bytes of data not including checksum.",
        data.len()
    );
    let mut combined: Vec<u8> = Vec::with_capacity(data.len() + checksum.len());
    combined.extend_from_slice(&data);
    combined.extend_from_slice(checksum);
    // Send data
    socket.write_all(&combined).await?;
    log::trace!("Data sent with checksum!");

    let mut ack = [0; 4];
    socket.read_exact(&mut ack).await?;
    if &ack == b"ACK\0" {
        log::info!("Client acknowledged data receipt.");
    } else {
        log::warn!("Client failed to acknowledge.");
        if abort_on_fail {
            return Err(anyhow!("Data corruption detected"));
        }
    }
    Ok(())
}

async fn recipient_logic(socket: &mut TcpStream, abort_on_fail: bool) -> anyhow::Result<()> {
    // Receive data
    let mut buffer = [0; 2048];
    let n = socket.read(&mut buffer).await?;
    log::debug!("Received {} bytes", n);
    let received_data = &buffer[..n - 32]; // message, accounting for 32 byte checksum

    // Receive checksum
    let received_checksum = &buffer[n - 32..n]; // 32 byte checksum

    log::trace!("Received Checksum: {:?}", hex::encode(received_checksum));

    // Calculate checksum on the recipient side
    let mut hasher = Sha256::new();
    hasher.update(received_data);
    let calculated_checksum: &[u8] = &hasher.finalize();

    log::trace!(
        "Calculated Checksum: {:?}",
        hex::encode(calculated_checksum)
    );

    // Validate checksum
    if &calculated_checksum[..] == received_checksum {
        log::info!("Data integrity verified!");
        socket.write_all(b"ACK\0").await?;
    } else {
        log::warn!("Data corruption detected!");
        socket.write_all(b"NACK\0").await?;
        if abort_on_fail {
            return Err(anyhow!("Data corruption detected"));
        }
    }
    Ok(())
}

async fn run_server(addr: &str, abort_on_fail: bool) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let (mut socket, _) = listener.accept().await?;
    log::info!("Client connected!");
    loop {
        sender_logic(&mut socket, abort_on_fail).await?;
        recipient_logic(&mut socket, abort_on_fail).await?;
    }
}

async fn run_client(addr: &str, abort_on_fail: bool) -> anyhow::Result<()> {
    let mut socket = TcpStream::connect(addr).await?;
    log::info!("Connected to server!");
    loop {
        recipient_logic(&mut socket, abort_on_fail).await?;
        sender_logic(&mut socket, abort_on_fail).await?;
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Simple program to validate data sent through TCP
struct Cli {
    /// Whether to run as server or client
    unit: Unit,
    /// Bind address for the server, and connection address for the client
    /// Example(server): 0.0.0.0:8080, Example(client): 127.0.0.1:8080
    address: String,
    /// Define a log level (default=Warn)
    #[arg(long)]
    log_level: Option<LogLevel>,
    /// Abort on failure
    #[arg(long)]
    abort_on_fail: bool,
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
    match args.unit {
        Unit::Server => run_server(&args.address, args.abort_on_fail).await?,
        Unit::Client => run_client(&args.address, args.abort_on_fail).await?,
    }
    Ok(())
}
