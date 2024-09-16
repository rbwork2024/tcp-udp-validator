use std::time::Duration;

use rand::Rng;
use sha2::{Digest, Sha256};
use tokio::net::UdpSocket;

pub async fn sender_logic(
    socket: &mut UdpSocket,
    send_addr: &str,
    abort_on_fail: bool,
    prev_success: bool,
) -> anyhow::Result<bool> {
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
    socket.send_to(&combined, send_addr).await?;
    let mut ack = [0; 4];
    let timeout = tokio::time::timeout(Duration::from_secs(1), socket.recv_from(&mut ack)).await;
    match timeout {
        Ok(Err(e)) => {
            return Err(e.into());
        }
        Ok(Ok((_, _))) => {}
        Err(_) => {
            return Ok(false);
        }
    };

    if &ack == b"ACK\0" {
        if !prev_success {
            println!(
                "[{}] Connected and sent data between client and server!",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            );
        }
        Ok(true)
    } else {
        if abort_on_fail {
            return Err(anyhow::anyhow!("Data corruption detected"));
        }
        Ok(false)
    }
}

pub async fn recipient_logic(
    socket: &mut UdpSocket,
    abort_on_fail: bool,
    prev_success: bool,
) -> anyhow::Result<bool> {
    // Receive data
    let mut buffer = [0; 2048];
    let timeout = tokio::time::timeout(Duration::from_secs(1), socket.recv_from(&mut buffer)).await;
    let (n, sender) = match timeout {
        Ok(Err(e)) => {
            return Err(e.into());
        }
        Ok(Ok((n, sender))) => (n, sender),
        Err(_) => {
            return Ok(false);
        }
    };
    let received_data = &buffer[..n - 32]; // message, accounting for 32 byte checksum
                                           // Receive checksum
    let received_checksum = &buffer[n - 32..n]; // 32 byte checksum
                                                // Calculate checksum on the recipient side
    let mut hasher = Sha256::new();
    hasher.update(received_data);
    let calculated_checksum: &[u8] = &hasher.finalize();
    // Validate checksum
    if &calculated_checksum[..] == received_checksum {
        socket.send_to(b"ACK\0", sender).await?;
        if !prev_success {
            println!(
                "[{}] Connected and sent data between client and server!",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            );
        }
        Ok(true)
    } else {
        socket.send_to(b"NACK\0", sender).await?;
        if abort_on_fail {
            return Err(anyhow::anyhow!("Data corruption detected"));
        }
        Ok(false)
    }
}
