use rand::Rng;
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::util::print_and_log;

const REFRESH_INTERVAL: u64 = 10000;

pub async fn run_server(addr: &str, print: bool) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    print_and_log("Waiting for connection to client!", print, log::Level::Info);
    let (mut socket, _) = listener.accept().await?;
    print_and_log("Connected!", print, log::Level::Info);
    let mut success_counter: u64 = 0;
    let mut failure_counter: u64 = 0;
    loop {
        let mut update = false;
        match sender_logic(&mut socket).await {
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
                        log::Level::Info,
                    );
                }
            }
            Err(e) => {
                print_and_log(
                    &format!("There was a problem with the connection: {}", e),
                    print,
                    log::Level::Info,
                );
                let mut connected = false;
                while !connected {
                    print_and_log("Attempting to reconnect...", print, log::Level::Info);
                    if let Ok((s, _)) = listener.accept().await {
                        socket = s;
                        connected = true;
                    }
                }
            }
        }
    }
}

pub async fn run_client(addr: &str, print: bool) -> anyhow::Result<()> {
    print_and_log("Waiting for connection to server!", print, log::Level::Info);
    let mut socket = TcpStream::connect(addr).await?;
    print_and_log("Connected!", print, log::Level::Info);
    let mut success_counter: u64 = 0;
    let mut failure_counter: u64 = 0;
    loop {
        let mut update = false;
        match recipient_logic(&mut socket).await {
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
                        log::Level::Info,
                    );
                }
            }
            Err(e) => {
                print_and_log(
                    &format!("There was a problem with the connection: {}", e),
                    print,
                    log::Level::Info,
                );
                let mut connected = false;
                while !connected {
                    print_and_log("Attempting to reconnect...", print, log::Level::Info);
                    if let Ok(s) = TcpStream::connect(addr).await {
                        socket = s;
                        connected = true;
                    }
                }
            }
        }
    }
}

async fn sender_logic(socket: &mut TcpStream) -> anyhow::Result<bool> {
    let mut data = [0u8; 1024];
    rand::thread_rng().fill(&mut data);
    // Calculate the checksum using SHA256
    let mut hasher = Sha256::new();
    hasher.update(data);
    let checksum: &[u8] = &hasher.finalize();
    let mut combined: Vec<u8> = Vec::with_capacity(data.len() + checksum.len());
    combined.extend_from_slice(&data);
    combined.extend_from_slice(checksum);
    // Send data
    socket.write_all(&combined).await?;
    let mut ack = [0; 4];
    socket.read_exact(&mut ack).await?;
    if &ack == b"ACK\0" {
        Ok(true)
    } else {
        log::error!("Data corruption detected");
        Ok(false)
    }
}

async fn recipient_logic(socket: &mut TcpStream) -> anyhow::Result<bool> {
    // Receive data
    let mut buffer = [0; 2048];
    let n = socket.read(&mut buffer).await?;
    let received_data = &buffer[..n - 32]; // message, accounting for 32 byte checksum
                                           // Receive checksum
    let received_checksum = &buffer[n - 32..n]; // 32 byte checksum
                                                // Calculate checksum on the recipient side
    let mut hasher = Sha256::new();
    hasher.update(received_data);
    let calculated_checksum: &[u8] = &hasher.finalize();
    // Validate checksum
    if &calculated_checksum[..] == received_checksum {
        socket.write_all(b"ACK\0").await?;
        Ok(true)
    } else {
        socket.write_all(b"NACK\0").await?;
        log::error!("Data corruption detected");
        Ok(false)
    }
}
