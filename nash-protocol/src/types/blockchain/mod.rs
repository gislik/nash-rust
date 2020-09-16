//! Types used across protocol requests for constructing and validating
//! blockchain payload data. Any types specific to an individual protocol
//! request will live in the respective module.

pub mod btc;
pub mod eth;
pub mod neo;

use crate::errors::{ProtocolError, Result};
use crate::types::{Asset, Blockchain};
use std::convert::TryFrom;

/// The type prefix indicates what operation this data represents. This is
/// encoded by 1 byte in the protocol. For example, a payload representing
/// the fill order operation has `0x01` at the start of the data. These
/// prefixes are the same for both NEO and Ethereum payloads.

#[derive(Clone, Debug, PartialEq)]
pub enum Prefix {
    SyncState,
    FillOrder,
    Deposit,
    Withdrawal,
}

impl Prefix {
    pub fn to_bytes(&self) -> [u8; 1] {
        match self {
            Self::SyncState => [0x00],
            Self::FillOrder => [0x01],
            Self::Deposit => [0x02],
            Self::Withdrawal => [0x03],
        }
    }
    pub fn from_bytes(bytes: [u8; 1]) -> Result<Self> {
        match bytes {
            [0x00] => Ok(Self::SyncState),
            [0x01] => Ok(Self::FillOrder),
            [0x02] => Ok(Self::Deposit),
            [0x03] => Ok(Self::Withdrawal),
            _ => Err(ProtocolError("Invalid prefix byte")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Address {
    Ethereum(eth::Address),
    Bitcoin(btc::Address),
    NEO(neo::Address),
}

impl Address {
    pub fn new(chain: Blockchain, hex_str: &str) -> Result<Self> {
        match chain {
            Blockchain::Bitcoin => Ok(Self::Bitcoin(btc::Address::new(hex_str)?)),
            Blockchain::Ethereum => Ok(Self::Ethereum(eth::Address::new(hex_str)?)),
            Blockchain::NEO => Ok(Self::NEO(neo::Address::new(hex_str)?)),
        }
    }
}

impl TryFrom<Address> for eth::Address {
    type Error = ProtocolError;

    fn try_from(address: Address) -> Result<Self> {
        match address {
            Address::Ethereum(address) => Ok(address),
            _ => Err(ProtocolError(
                "Tried to convert from something that is not an ETH address",
            )),
        }
    }
}

impl TryFrom<Address> for neo::Address {
    type Error = ProtocolError;

    fn try_from(address: Address) -> Result<Self> {
        match address {
            Address::NEO(address) => Ok(address),
            _ => Err(ProtocolError(
                "Tried to convert from something that is not an NEO address",
            )),
        }
    }
}

impl TryFrom<Address> for btc::Address {
    type Error = ProtocolError;

    fn try_from(address: Address) -> Result<Self> {
        match address {
            Address::Bitcoin(address) => Ok(address),
            _ => Err(ProtocolError(
                "Tried to convert from something that is not an ETH address",
            )),
        }
    }
}

/// An asset in the Ethereum and NEO smart contract protocols is represented
/// either by an asset id or an indicator that the trade is occurring across
/// blockchains.
#[derive(Clone, Debug, PartialEq)]
pub enum AssetOrCrosschain {
    Asset(Asset),
    Crosschain,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PublicKey {
    Bitcoin(btc::PublicKey),
    Ethereum(eth::PublicKey),
    NEO(neo::PublicKey),
}

impl PublicKey {
    pub fn new(chain: Blockchain, hex_str: &str) -> Result<Self> {
        Ok(match chain {
            Blockchain::Bitcoin => Self::Bitcoin(btc::PublicKey::new(hex_str)?),
            Blockchain::Ethereum => Self::Ethereum(eth::PublicKey::new(hex_str)?),
            Blockchain::NEO => Self::NEO(neo::PublicKey::new(hex_str)?),
        })
    }

    pub fn to_hex_str(&self) -> String {
        match self {
            Self::Bitcoin(key) => key.to_hex(),
            Self::Ethereum(key) => key.to_hex(),
            Self::NEO(key) => key.to_hex(),
        }
    }

    pub fn to_address(&self) -> Result<Address> {
        Ok(match self {
            Self::Bitcoin(key) => Address::Bitcoin(key.to_address()?),
            Self::Ethereum(key) => Address::Ethereum(key.to_address()),
            Self::NEO(key) => Address::NEO(key.to_address()),
        })
    }
}

impl TryFrom<PublicKey> for eth::PublicKey {
    type Error = ProtocolError;

    fn try_from(address: PublicKey) -> Result<Self> {
        match address {
            PublicKey::Ethereum(pub_key) => Ok(pub_key),
            _ => Err(ProtocolError(
                "Tried to convert from something that is not an ETH public key",
            )),
        }
    }
}

impl TryFrom<PublicKey> for neo::PublicKey {
    type Error = ProtocolError;

    fn try_from(address: PublicKey) -> Result<Self> {
        match address {
            PublicKey::NEO(pub_key) => Ok(pub_key),
            _ => Err(ProtocolError(
                "Tried to convert from something that is not an NEO public key",
            )),
        }
    }
}

impl TryFrom<PublicKey> for btc::PublicKey {
    type Error = ProtocolError;

    fn try_from(address: PublicKey) -> Result<Self> {
        match address {
            PublicKey::Bitcoin(pub_key) => Ok(pub_key),
            _ => Err(ProtocolError(
                "Tried to convert from something that is not an BTC public key",
            )),
        }
    }
}

/// Wrapper type for the two kinds of movments: deposit and withdrawal
#[derive(PartialEq)]
pub enum MovementType {
    Deposit,
    Withdrawal,
}

impl MovementType {
    /// Return appropriate prefix for the movement on ETH
    pub fn to_prefix(&self) -> Prefix {
        match self {
            Self::Deposit => Prefix::Deposit,
            Self::Withdrawal => Prefix::Withdrawal,
        }
    }
}
