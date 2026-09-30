extern crate alloc;

use alloc::{vec, vec::Vec};
use anyhow::{Context, Error, Result, anyhow, bail};
use core::{
    mem::{size_of, size_of_val},
    net::SocketAddrV4,
};
use embassy_futures::select::{Either, select};
use embassy_net::{
    Stack,
    tcp::{TcpReader, TcpSocket, TcpWriter},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use embedded_io_async::{Read, Write};
use esp32_ecdsa::{CryptoContext, ecdsa_sign, ecdsa_verify};
use rand_core::RngCore;

use crate::{
    DeviceId,
    protocol::simple::{DeviceBoundSimpleMessage, SIGNATURE_LEN, ServerBoundSimpleMessage},
};

const _: () = assert!(
    SIGNATURE_LEN == esp32_ecdsa::SIGNATURE_LEN,
    "esp32_ecdsa's SIGNATURE_LEN is differs from simple protocol's SIGNATURE_LEN"
);

const MAX_SIMPLE_PAYLOAD_LEN: usize = 4096;

#[derive(Debug)]
pub enum TransportEvent {
    Connected,
    Disconnected,
    Message(DeviceBoundSimpleMessage),
    Error(Error),
}

pub struct TransportChannels {
    pub outgoing: Channel<CriticalSectionRawMutex, ServerBoundSimpleMessage, 32>,
    pub incoming: Channel<CriticalSectionRawMutex, TransportEvent, 32>,
}

impl TransportChannels {
    pub const fn new() -> Self {
        Self {
            outgoing: Channel::new(),
            incoming: Channel::new(),
        }
    }
}

#[embassy_executor::task]
pub async fn transport_task(
    stack: &'static Stack<'static>,
    server_addr: SocketAddrV4,
    channels: &'static TransportChannels,
    device_id: DeviceId,
    mut crypto: CryptoContext<'static>,
) {
    loop {
        Timer::after(Duration::from_secs(5)).await;

        match run_connection(stack, server_addr, channels, &device_id, &mut crypto)
            .await
            .context("failed to run connection loop")
        {
            Ok(()) => {
                // Clean disconnect (should never happen)
                channels.incoming.send(TransportEvent::Disconnected).await;
            }
            Err(e) => {
                channels.incoming.send(TransportEvent::Error(e)).await;
                channels.incoming.send(TransportEvent::Disconnected).await;
            }
        }
    }
}

async fn run_connection(
    stack: &'static Stack<'static>,
    server_addr: SocketAddrV4,
    channels: &TransportChannels,
    device_id: &DeviceId,
    crypto: &mut CryptoContext<'_>,
) -> Result<()> {
    let mut rx_buffer = [0u8; 4096];
    let mut tx_buffer = [0u8; 4096];

    let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_keep_alive(Some(Duration::from_secs(60)));

    socket
        .connect(server_addr)
        .await
        .map_err(|e| anyhow!("failed to connect: {:?}", e))?;

    // Nonce handshake
    let mut expected_recv_nonce = crypto.trng.next_u32();
    socket
        .write(&expected_recv_nonce.to_be_bytes())
        .await
        .map_err(|err| anyhow!("failed to send client nonce: {:?}", err))?;

    let mut send_nonce_buf = [0u8; size_of::<u32>()];
    socket
        .read_exact(&mut send_nonce_buf)
        .await
        .map_err(|err| anyhow!("failed to read server nonce: {:?}", err))?;
    let mut send_nonce = u32::from_be_bytes(send_nonce_buf);

    send_identify_message(&mut socket, *device_id).await?;

    channels.incoming.send(TransportEvent::Connected).await;

    let (mut reader, mut writer) = socket.split();

    loop {
        match select(reader.wait_read_ready(), channels.outgoing.receive()).await {
            Either::First(()) => {
                let message = read_message(&mut reader, &mut expected_recv_nonce, crypto).await?;
                channels
                    .incoming
                    .send(TransportEvent::Message(message))
                    .await;
            }

            Either::Second(message) => {
                send_message(&mut writer, &mut send_nonce, crypto, &message).await?;
            }
        }
    }
}

async fn read_message(
    reader: &mut TcpReader<'_>,
    expected_recv_nonce: &mut u32,
    crypto: &mut CryptoContext<'_>,
) -> Result<DeviceBoundSimpleMessage> {
    let mut nonce_buf = [0u8; size_of::<u32>()];
    reader
        .read_exact(&mut nonce_buf)
        .await
        .map_err(|err| anyhow!("failed to read nonce: {:?}", err))?;
    let recv_nonce = u32::from_be_bytes(nonce_buf);

    *expected_recv_nonce = expected_recv_nonce.wrapping_add(1);
    if recv_nonce != *expected_recv_nonce {
        bail!(
            "Server nonce did not match expected value! expected={}, got={}",
            *expected_recv_nonce,
            recv_nonce
        );
    }

    let mut len_buf = [0u8; size_of::<u32>()];
    reader
        .read_exact(&mut len_buf)
        .await
        .map_err(|err| anyhow!("failed to read length: {:?}", err))?;
    let payload_len = u32::from_be_bytes(len_buf) as usize;

    if payload_len > MAX_SIMPLE_PAYLOAD_LEN {
        bail!("payload length exceeds maximum simple payload size");
    }

    let mut payload = vec![0u8; payload_len];
    reader
        .read_exact(&mut payload)
        .await
        .map_err(|err| anyhow!("data recv: {:?}", err))?;

    let mut sig_buf = [0u8; SIGNATURE_LEN];
    reader
        .read_exact(&mut sig_buf)
        .await
        .map_err(|err| anyhow!("sig recv: {:?}", err))?;

    let mut to_verify = Vec::with_capacity(size_of::<u32>() + size_of::<u32>() + payload.len());
    to_verify.extend_from_slice(&recv_nonce.to_be_bytes());
    to_verify.extend_from_slice(&(payload_len as u32).to_be_bytes());
    to_verify.extend_from_slice(&payload);

    if !ecdsa_verify(crypto, &to_verify, &sig_buf).context("ecdsa verification failed")? {
        bail!("signature does not match!");
    }

    serde_json::from_slice::<DeviceBoundSimpleMessage>(&payload).context("failed to parse message")
}

async fn send_identify_message(socket: &mut TcpSocket<'_>, device_id: DeviceId) -> Result<()> {
    let data = serde_json::to_vec(&ServerBoundSimpleMessage::Identify(device_id))?;

    let len_be = (data.len() as u32).to_be_bytes();

    let mut payload = Vec::with_capacity(size_of_val(&len_be) + data.len());
    payload.extend(&len_be);
    payload.extend(&data);

    socket
        .write_all(&payload)
        .await
        .map_err(|err| anyhow!("{:?}", err))?;

    Ok(())
}

async fn send_message(
    socket: &mut TcpWriter<'_>,
    send_nonce: &mut u32,
    crypto: &mut CryptoContext<'_>,
    message: &ServerBoundSimpleMessage,
) -> Result<()> {
    let payload = serde_json::to_vec(message)?;
    let payload_len_be = (payload.len() as u32).to_be_bytes();

    *send_nonce = send_nonce.wrapping_add(1);
    let nonce_be = send_nonce.to_be_bytes();

    let unsigned_len = size_of_val(send_nonce) + size_of_val(&payload_len_be) + payload.len();

    let mut data = Vec::with_capacity(unsigned_len + SIGNATURE_LEN);
    data.extend(&nonce_be);
    data.extend(&payload_len_be);
    data.extend(&payload);

    let sig = ecdsa_sign(crypto, &data[..unsigned_len]).context("ecdsa signing failed")?;

    data.extend(&sig);

    socket
        .write_all(&data)
        .await
        .map_err(|err| anyhow!("{:?}", err))?;

    Ok(())
}
