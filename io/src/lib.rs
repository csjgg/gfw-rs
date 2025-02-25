//! This crate provides functionality for handling network packets, including
//! registering callbacks for packet processing, setting verdicts for packets,
//! and managing packet I/O operations.

pub mod nfqueue;
pub mod pcap;

use std::error::Error;
use std::{any::Any, time::SystemTime};

use downcast_rs::{impl_downcast, DowncastSync};
use tokio::net::TcpStream;

/// Verdict represents the possible actions that can be taken on a packet.
#[derive(Debug)]
pub enum Verdict {
    /// Accept accepts the packet, but continues to process the stream.
    Accept,
    /// AcceptModify is like Accept, but replaces the packet with a new one.
    AcceptModify,
    /// AcceptStream accepts the packet and stops processing the stream.
    AcceptStream,
    /// Drop drops the packet, but does not block the stream.
    Drop,
    /// DropStream drops the packet and blocks the stream.
    DropStream,
}

/// Packet represents an IP packet.
pub trait Packet: DowncastSync + Send + Sync {
    /// The ID of the stream the packet belongs to.
    fn stream_id(&self) -> u32;

    /// The time the packet was received.
    fn timestamp(&self) -> SystemTime;

    /// The raw packet data, starting with the IP header.
    fn data(&self) -> &[u8];

    fn as_any(&self) -> &dyn Any;
}

impl_downcast!(sync Packet);

impl std::fmt::Debug for dyn Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Packet: stream_id = {:?}, timestamp = {:?}, data = {:?},",
            self.stream_id(),
            self.timestamp(),
            self.data()
        )
    }
}

/// The function to be called for each received packet.
/// Return false to "unregister" and stop receiving packets.
// pub type PacketCallback =
//     Box<dyn Fn(Box<dyn Packet>, Option<Box<dyn Error + Send + Sync>>) -> bool + Send + Sync>;
use std::future::Future;
use std::pin::Pin;

pub type PacketCallback = Box<
    dyn Fn(
            Box<dyn Packet>,
            Option<Box<dyn Error + Send + Sync>>,
        ) -> Pin<Box<dyn Future<Output = bool> + Send>>
        + Send
        + Sync,
>;

/// Manage the packet io.
#[async_trait::async_trait]
pub trait PacketIO: DowncastSync + Send + Sync {
    /// Registers a callback function to be called for each received packet.
    ///
    /// # Arguments
    ///
    /// * `callback` - A `PacketCallback` function to be called for each packet.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - A result indicating success or failure.
    async fn register(
        &self,
        callback: PacketCallback,
        service_rx: tokio::sync::watch::Receiver<bool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Set the verdict for a packet. (Used in iptables/nftables)
    ///
    /// # Arguments
    ///
    /// * `packet` - A boxed `Packet` instance.
    /// * `verdict` - A `Verdict` indicating the verdict for the packet.
    /// * `data` - A vector of bytes representing additional data.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - A result indicating success or failure.
    async fn set_verdict(
        &self,
        packet: Box<dyn Packet>,
        verdict: Verdict,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Establishes a protected TCP connection to the given address.
    ///
    /// The packets sent/received through the connection must bypass
    /// the packet IO and not be processed by the callback.
    ///
    /// # Arguments
    ///
    /// * `addr` - A string slice representing the address to connect to.
    ///
    /// # Returns
    ///
    /// * `Result<TcpStream, Box<dyn Error>>` - A result containing the TCP stream or an error.
    async fn protected_conn(
        &self,
        address: &str,
    ) -> Result<TcpStream, Box<dyn Error + Send + Sync>>;

    ///// Close the packet io.
    //async fn close(&self) -> Result<(), Whatever>;

    /// Sets a cancellation function to be called when the packet processing is cancelled.
    ///
    /// # Arguments
    ///
    /// * `cancel_func` - A boxed function to be called on cancellation.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - A result indicating success or failure.
    async fn set_cancel_func(
        &self,
        cancel_func: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    async fn close(&self);
}

impl_downcast!(sync PacketIO);
