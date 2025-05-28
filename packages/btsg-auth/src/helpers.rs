use cosmwasm_std::{to_json_binary, StdResult};
use sha2::{Digest, Sha256};

use crate::{Any, MsgAddAuthenticator};

pub fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Register a given seckp256k1 key with a specific authenticator
async fn setup_bitsong_smart_account(authenticator: MsgAddAuthenticator) -> StdResult<Any> {
    // register custom authenticator to account
    Ok(Any {
        type_url: "/bitsong.smartaccount.v1beta1.MsgAddAuthenticator".into(),
        value: to_json_binary(&authenticator)?,
    })
}
