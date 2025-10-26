use anyhow::{Context, Error, Result, anyhow, bail};
use core::mem::{size_of, size_of_val};
use p256::{
    NistP256,
    ecdsa::{
        Signature, SigningKey, VerifyingKey,
        signature::{Signer, Verifier},
    },
    elliptic_curve::Curve,
};
use rand::{TryRngCore, rngs::OsRng};
use std::net::SocketAddr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpSocket, TcpStream},
    select,
    sync::mpsc,
    time::{Duration, sleep},
};

use crate::{
    DeviceId,
    protocol::simple::{DeviceBoundSimpleMessage, SIGNATURE_LEN, ServerBoundSimpleMessage},
};

const _: () = assert!(
    SIGNATURE_LEN == NistP256::ORDER.bits() / 8 * 2, // 8 bits per byte / 2 field elements
    "Length of two Nist p256 field elements differs from simple protocol's SIGNATURE_LEN"
);

#[derive(Debug, Clone)]
pub struct CryptoContext {
    pub server_public_key: VerifyingKey,
    pub private_key: SigningKey,
}

#[derive(Debug)]
pub enum TransportEvent {
    Connected,
    Disconnected,
    Message(DeviceBoundSimpleMessage),
    Error(Error),
}

// Client-facing ends: send outgoing app messages, receive incoming transport events.
pub struct TransportClient {
    pub outgoing: mpsc::Sender<ServerBoundSimpleMessage>,
    pub incoming: mpsc::Receiver<TransportEvent>,
}

// Worker-facing ends: receive outgoing app messages, send incoming transport events.
pub struct TransportWorker {
    pub outgoing: mpsc::Receiver<ServerBoundSimpleMessage>,
    pub incoming: mpsc::Sender<TransportEvent>,
}

pub fn make_transport_channels(capacity: usize) -> (TransportClient, TransportWorker) {
    let (outgoing_tx, outgoing_rx) = mpsc::channel::<ServerBoundSimpleMessage>(capacity);
    let (incoming_tx, incoming_rx) = mpsc::channel::<TransportEvent>(capacity);

    let client = TransportClient {
        outgoing: outgoing_tx,
        incoming: incoming_rx,
    };

    let worker = TransportWorker {
        outgoing: outgoing_rx,
        incoming: incoming_tx,
    };

    (client, worker)
}

pub async fn transport_task(
    server_addr: SocketAddr,
    mut worker: TransportWorker,
    device_id: DeviceId,
    crypto: CryptoContext,
) {
    loop {
        sleep(Duration::from_secs(5)).await;

        match run_connection(server_addr, &mut worker, &device_id, &crypto)
            .await
            .context("failed to run connection loop")
        {
            Ok(()) => {
                // Clean disconnect (should never happen)
                let _ = worker.incoming.send(TransportEvent::Disconnected).await;
            }
            Err(e) => {
                let _ = worker.incoming.send(TransportEvent::Error(e)).await;
                let _ = worker.incoming.send(TransportEvent::Disconnected).await;
            }
        }
    }
}

async fn run_connection(
    server_addr: SocketAddr,
    worker: &mut TransportWorker,
    device_id: &DeviceId,
    crypto: &CryptoContext,
) -> Result<()> {
    // Build socket and set keepalive before connecting
    let socket = match server_addr {
        SocketAddr::V4(_) => TcpSocket::new_v4(),
        SocketAddr::V6(_) => TcpSocket::new_v6(),
    }
    .context("failed to create IP socket")?;
    socket
        .set_keepalive(true)
        .context("failed to set keepalive")?;
    let mut stream = socket
        .connect(server_addr)
        .await
        .context("failed to connect")?;

    // Nonce handshake
    let mut expected_recv_nonce = OsRng.try_next_u32().context("failed to generate nonce")?;
    stream
        .write_all(&expected_recv_nonce.to_be_bytes())
        .await
        .context("failed to send client nonce")?;

    let mut send_nonce_buf = [0u8; size_of::<u32>()];
    stream
        .read_exact(&mut send_nonce_buf)
        .await
        .context("failed to read server nonce")?;
    let mut send_nonce = u32::from_be_bytes(send_nonce_buf);

    send_identify_message(&mut stream, *device_id).await?;

    worker.incoming.send(TransportEvent::Connected).await?;

    loop {
        // Read the next incoming message header OR send an outgoing message, whichever is ready first.
        // We use read_exact for the nonce to avoid the partial-read FIXME in the original code.
        let mut nonce_buf = [0u8; size_of::<u32>()];

        select! {
            read_res = stream.read_exact(&mut nonce_buf) => {
                read_res.context("failed to read nonce")?;
                let recv_nonce = u32::from_be_bytes(nonce_buf);

                expected_recv_nonce = expected_recv_nonce.wrapping_add(1);
                if recv_nonce != expected_recv_nonce {
                    bail!(
                        "Server nonce did not match expected value! expected={}, got={}",
                        expected_recv_nonce,
                        recv_nonce
                    );
                }

                let mut len_buf = [0u8; size_of::<u32>()];
                stream
                    .read_exact(&mut len_buf)
                    .await
                    .context("failed to read payload length")?;
                let payload_len = u32::from_be_bytes(len_buf) as usize;

                let mut payload = vec![0u8; payload_len];
                stream
                    .read_exact(&mut payload)
                    .await
                    .context("failed to read payload")?;

                let mut sig_buf = [0u8; SIGNATURE_LEN];
                stream
                    .read_exact(&mut sig_buf)
                    .await
                    .context("failed to read signature")?;

                // Verify signature over [nonce | len | payload]
                let mut to_verify = Vec::with_capacity(size_of::<u32>() + size_of::<u32>() + payload.len());
                to_verify.extend_from_slice(&recv_nonce.to_be_bytes());
                to_verify.extend_from_slice(&(payload_len as u32).to_be_bytes());
                to_verify.extend_from_slice(&payload);

                crypto
                    .server_public_key
                    .verify(&to_verify, &Signature::from_slice(&sig_buf)?)
                    .context("ecdsa verification failed")?;

                match serde_json::from_slice::<DeviceBoundSimpleMessage>(&payload)
                    .context("failed to parse message")
                {
                    Ok(msg) => {
                         worker.incoming.send(TransportEvent::Message(msg)).await?;
                    }
                    Err(err) => {
                         worker.incoming.send(TransportEvent::Error(err)).await?;
                    }
                }
            }

            maybe_message = worker.outgoing.recv() => {
                match maybe_message {
                    Some(message) => {
                        send_message(&mut stream, &mut send_nonce, crypto, &message).await?;
                    }
                    None => {
                        bail!("outgoing channel closed");
                    }
                }
            }
        }
    }
}

async fn send_identify_message(stream: &mut TcpStream, device_id: DeviceId) -> Result<()> {
    let data = serde_json::to_vec(&ServerBoundSimpleMessage::Identify(device_id))?;
    let len_be = (data.len() as u32).to_be_bytes();

    let mut payload = Vec::with_capacity(size_of_val(&len_be) + data.len());
    payload.extend(&len_be);
    payload.extend(&data);

    stream
        .write_all(&payload)
        .await
        .map_err(|err| anyhow!("{:?}", err))?;

    Ok(())
}

async fn send_message(
    stream: &mut TcpStream,
    send_nonce: &mut u32,
    crypto: &CryptoContext,
    message: &ServerBoundSimpleMessage,
) -> Result<()> {
    let payload = serde_json::to_vec(message)?;
    let payload_len_be = (payload.len() as u32).to_be_bytes();

    *send_nonce = send_nonce.wrapping_add(1);
    let nonce_be = send_nonce.to_be_bytes();

    let unsigned_len = size_of::<u32>() + size_of_val(&payload_len_be) + payload.len();

    let mut data = Vec::with_capacity(unsigned_len + SIGNATURE_LEN);
    data.extend(&nonce_be);
    data.extend(&payload_len_be);
    data.extend(&payload);

    let sig: Signature = crypto
        .private_key
        .clone()
        .try_sign(&data[..unsigned_len])
        .context("failed to sign outbound message")?;
    data.extend(&sig.to_bytes());

    stream
        .write_all(&data)
        .await
        .map_err(|err| anyhow!("{:?}", err))?;

    Ok(())
}
