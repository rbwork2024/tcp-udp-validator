use rand::Rng;
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn sender_logic(socket: &mut TcpStream, abort_on_fail: bool) -> anyhow::Result<()> {
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
        log::debug!("Client acknowledged data receipt.");
    } else {
        log::warn!("Client failed to acknowledge.");
        if abort_on_fail {
            return Err(anyhow::anyhow!("Data corruption detected"));
        }
    }
    Ok(())
}

pub async fn recipient_logic(socket: &mut TcpStream, abort_on_fail: bool) -> anyhow::Result<()> {
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
        log::debug!("Data integrity verified!");
        socket.write_all(b"ACK\0").await?;
    } else {
        log::warn!("Data corruption detected!");
        socket.write_all(b"NACK\0").await?;
        if abort_on_fail {
            return Err(anyhow::anyhow!("Data corruption detected"));
        }
    }
    Ok(())
}
