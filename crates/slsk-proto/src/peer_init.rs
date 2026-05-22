// Peer init message structs
pub mod pierce_firewall;
pub mod req;

use crate::codec::SlskRead;
use crate::error::ProtoError;
use bytes::{Buf, Bytes};

/// Top-level peer init message dispatcher
#[derive(Debug)]
pub enum PeerInitMessage {
    PierceFirewall(pierce_firewall::PierceFirewallRequest),
    PeerInit(req::PeerInitRequest),
    Unknown(u8, Bytes),
}

impl PeerInitMessage {
    pub fn decode(code: u8, payload: &mut impl Buf) -> Result<Self, ProtoError> {
        match code {
            pierce_firewall::CODE => Ok(Self::PierceFirewall(
                pierce_firewall::PierceFirewallRequest::read(payload)?,
            )),
            req::CODE => Ok(Self::PeerInit(req::PeerInitRequest::read(payload)?)),
            other => Ok(Self::Unknown(
                other,
                payload.copy_to_bytes(payload.remaining()),
            )),
        }
    }
}
