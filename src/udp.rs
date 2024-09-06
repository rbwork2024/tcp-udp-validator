use rand::Rng;
use sha2::{Digest, Sha256};
use tokio::net::UdpSocket;

pub async fn sender_logic(
    socket: &mut UdpSocket,
    send_addr: &str,
    abort_on_fail: bool,
) -> anyhow::Result<()> {
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
    if let Err(e) = socket.recv_from(&mut ack).await {
        log::error!("There was a problem receiving acknowledgement from the recipient. \
        If this error occurred early on, make sure you're running the client first, *then* the server. inner: '{}'", e);
        return Err(e.into());
    }
    if &ack == b"ACK\0" {
        log::info!("Client acknowledged data receipt.");
    } else {
        log::warn!("Client failed to acknowledge.");
        if abort_on_fail {
            return Err(anyhow::anyhow!("Data corruption detected"));
        }
    }
    Ok(())
}

pub async fn recipient_logic(
    socket: &mut UdpSocket,
    send_addr: &str,
    abort_on_fail: bool,
) -> anyhow::Result<()> {
    // Receive data
    let mut buffer = [0; 2048];
    let (n, _) = socket.recv_from(&mut buffer).await?;
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
        socket.send_to(b"ACK\0", send_addr).await?;
    } else {
        log::warn!("Data corruption detected!");
        socket.send_to(b"NACK\0", send_addr).await?;
        if abort_on_fail {
            return Err(anyhow::anyhow!("Data corruption detected"));
        }
    }
    Ok(())
}
