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

    log::trace!("Checksum: {:?}", hex::encode(checksum));

    log::debug!(
        "Sending {} bytes of data not including checksum.",
        data.len()
    );
    let mut combined: Vec<u8> = Vec::with_capacity(data.len() + checksum.len());
    combined.extend_from_slice(&data);
    combined.extend_from_slice(checksum);
    // Send data
    socket.send_to(&combined, send_addr).await?;
    log::trace!("Data sent with checksum!");

    let mut ack = [0; 4];
    let timeout = tokio::time::timeout(Duration::from_secs(1), socket.recv_from(&mut ack)).await;
    match timeout {
        Ok(Err(e)) => {
            log::error!("There was a problem receiving acknowledgement from the recipient. \
        If this error occurred early on, make sure you're running the client first, *then* the server. inner: '{}'", e);
            return Err(e.into());
        }
        Ok(Ok((n, addr))) => {
            log::debug!("Received {} bytes from receiver addr {}", n, addr);
        }
        Err(e) => {
            log::warn!("Couldn't read ACK from socket: {}", e);
            return Ok(false);
        }
    };

    if &ack == b"ACK\0" {
        log::debug!("Client acknowledged data receipt.");
        if !prev_success {
            log::info!("Connected and sent data between client and server!");
        }
        Ok(true)
    } else {
        log::warn!("Client failed to acknowledge.");
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
            log::error!(
                "There was a problem receiving data from the sender. \
        inner: '{}'",
                e
            );
            return Err(e.into());
        }
        Ok(Ok((n, sender))) => {
            log::debug!("Received {} bytes from sender addr {}", n, sender);
            (n, sender)
        }
        Err(e) => {
            log::warn!("Couldn't read sender data from socket: {}", e);
            return Ok(false);
        }
    };
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
        log::debug!("Data integrity verified!");
        socket.send_to(b"ACK\0", sender).await?;
        if !prev_success {
            log::info!("Connected and sent data between client and server!");
        }
        Ok(true)
    } else {
        log::warn!("Data corruption detected!");
        socket.send_to(b"NACK\0", sender).await?;
        if abort_on_fail {
            return Err(anyhow::anyhow!("Data corruption detected"));
        }
        Ok(false)
    }
}
