use super::super::{
    serializable_to_json, try_response_from_json, NashProtocol, ResponseOrError, State,
};
use super::response;
use crate::errors::{ProtocolError, Result};
use crate::graphql::dh_fill_pool;
use crate::types::Blockchain;
use nash_mpc::curves::secp256_k1::{Secp256k1Point, Secp256k1Scalar};
use nash_mpc::curves::secp256_r1::{Secp256r1Point, Secp256r1Scalar};

use async_trait::async_trait;
use futures::lock::Mutex;
use std::sync::Arc;

/// DhFillPool requests coordinate between the user's client and the Nash server to
/// gather a set of shared secret R values. The user sends a list of public ECDSA
/// Points to the Nash server. The server sends back its own list of public Points.
/// Both parties then multply the public point by the secret value to construct the
/// same shared secret value (diffie-hellman). Bitcoin and Ethereum both use the
/// Secp256k1 curve, while NEO users the Secp256r1 curve. While this request type
/// holds both the secret and the public values, only the public values are used in
/// creating the GraphQL request. The secrets are used to process a response.
#[derive(Clone, Debug)]
pub enum DhFillPoolRequest {
    Bitcoin(K1FillPool),
    Ethereum(K1FillPool),
    NEO(R1FillPool),
}

impl DhFillPoolRequest {
    /// Create a new DhFillPool request for a given blockchain
    pub fn new(chain: Blockchain) -> Result<Self> {
        match chain {
            Blockchain::Ethereum => Ok(Self::Ethereum(K1FillPool::new()?)),
            Blockchain::Bitcoin => Ok(Self::Bitcoin(K1FillPool::new()?)),
            Blockchain::NEO => Ok(Self::NEO(R1FillPool::new()?)),
        }
    }
    /// Get blockchain assocaited with DH request
    pub fn blockchain(&self) -> Blockchain {
        match self {
            Self::Bitcoin(_) => Blockchain::Bitcoin,
            Self::Ethereum(_) => Blockchain::Ethereum,
            Self::NEO(_) => Blockchain::NEO,
        }
    }
}

/// Values for k1 curve (Bitcoin and Ethereum)
#[derive(Clone, Debug)]
pub struct K1FillPool {
    pub publics: Vec<Secp256k1Point>,
    pub secrets: Vec<Secp256k1Scalar>,
}

impl K1FillPool {
    pub fn new() -> Result<Self> {
        let (secrets, publics) = nash_mpc::common::dh_init_secp256k1(100)
            .map_err(|_| ProtocolError("Could not initialize k1 values"))?;
        Ok(Self { publics, secrets })
    }
}

/// Values for r1 curve (NEO)
#[derive(Clone, Debug)]
pub struct R1FillPool {
    pub publics: Vec<Secp256r1Point>,
    pub secrets: Vec<Secp256r1Scalar>,
}

impl R1FillPool {
    pub fn new() -> Result<Self> {
        let (secrets, publics) = nash_mpc::common::dh_init_secp256r1(100)
            .map_err(|_| ProtocolError("Could not initialize r1 values"))?;
        Ok(Self { publics, secrets })
    }
}

/// Nash server returns a list of public values that we can use to
/// compute a DH shared secret
#[derive(Clone, Debug)]
pub struct DhFillPoolResponse {
    pub server_publics: Vec<String>,
}

/// Representation of server public keys that can be generated from a
/// response using the appropriate curve
pub enum ServerPublics {
    Bitcoin(Vec<Secp256k1Point>),
    Ethereum(Vec<Secp256k1Point>),
    NEO(Vec<Secp256r1Point>),
}

#[async_trait]
impl NashProtocol for DhFillPoolRequest {
    type Response = DhFillPoolResponse;
    /// Serialize a SignStates protocol request to a GraphQL string
    async fn graphql(&self, _state: Arc<Mutex<State>>) -> Result<serde_json::Value> {
        let query = self.make_query();
        serializable_to_json(&query)
    }
    /// Deserialize response to DhFillPool protocol response
    fn response_from_json(
        &self,
        response: serde_json::Value,
    ) -> Result<ResponseOrError<Self::Response>> {
        try_response_from_json::<DhFillPoolResponse, dh_fill_pool::ResponseData>(response)
    }
    /// Update pool with response from server
    async fn process_response(
        &self,
        response: &Self::Response,
        state: Arc<Mutex<State>>,
    ) -> Result<()> {
        let server_publics = ServerPublics::from_hexstrings(self.blockchain(), response)?;
        response::fill_pool(self, server_publics, state.clone()).await?;
        let mut state = state.lock().await;
        // Update state to indicate we now have 100 new r values
        state.signer()?.fill_r_vals(self.blockchain(), 100);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Blockchain, DhFillPoolRequest, NashProtocol};
    use crate::protocol::State;
    use futures::executor;
    use futures::lock::Mutex;
    use std::sync::Arc;

    #[test]
    fn serialize_dh_fill_pool() {
        let state = Arc::new(Mutex::new(
            State::new(Some("../nash-native-client/test_data/keyfile.json")).unwrap(),
        ));
        let async_block = async {
            println!(
                "{:?}",
                DhFillPoolRequest::new(Blockchain::Ethereum)
                    .unwrap()
                    .graphql(state)
                    .await
                    .unwrap()
            );
        };
        executor::block_on(async_block);
    }
}