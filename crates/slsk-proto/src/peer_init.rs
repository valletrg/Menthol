// Peer init message structs
pub mod pierce_firewall;
pub mod peer_init;

use bytes::{Buf, Bytes};
use crate::codec::SlskRead;
use crate::error::ProtoError;

/// Top-level peer init message dispatcher
#[derive(Debug)]
pub enum PeerInitMessage {
    PierceFirewall(pierce_firewall::PierceFirewallRequest),
    PeerInit(peer_init::PeerInitRequest),
    Unknown(u8, Bytes),
}

impl PeerInitMessage {
    pub fn decode(code: u8, payload: &mut impl Buf) -> Result<Self, ProtoError> {
        match code {
            pierce_firewall::CODE => Ok(Self::PierceFirewall(pierce_firewall::PierceFirewallRequest::read(payload)?)),
            peer_init::CODE => Ok(Self::PeerInit(peer_init::PeerInitRequest::read(payload)?)),
            other => Ok(Self::Unknown(other, payload.copy_to_bytes(payload.remaining()))),
        }
    }
}
