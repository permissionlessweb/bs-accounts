// condensed from: https://github.com/MegaRockLabs/smart-account-auth
use bech32::{Bech32, Hrp};
use cosmwasm_std::{ensure, Binary, StdError};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

#[cosmwasm_schema::cw_serde]
pub struct CosmosArbitrary {
    pub pubkey: Binary,
    pub signature: Binary,
    pub message: Binary,
    pub hrp: Option<String>,
}

impl CosmosArbitrary {
    pub fn message_digest(&self) -> Result<Vec<u8>, StdError> {
        ensure!(
            self.hrp.is_some(),
            StdError::generic_err("Must provide prefix for the public key".to_string())
        );
        Ok(sha256(
            &preamble_msg_arb_036(
                pubkey_to_address(&self.pubkey, self.hrp.as_ref().unwrap())?.as_str(),
                &self.message.to_string(),
            )
            .as_bytes(),
        ))
    }
    pub fn hrp(&self) -> Option<String> {
        self.hrp.clone()
    }

    fn _validate(&self) -> Result<(), StdError> {
        if !(self.signature.len() > 0
            && self.message.to_string().len() > 0
            && self.pubkey.len() > 0)
        {
            return Err(StdError::generic_err("Empty credential data".to_string()));
        }
        Ok(())
    }

    pub fn verify(&self) -> Result<(), StdError> {
        let success = cosmwasm_crypto::secp256k1_verify(
            &self.message_digest()?,
            &self.signature,
            &self.pubkey,
        )
        .map_err(|e| StdError::generic_err(e.to_string()))?;
        ensure!(
            success,
            StdError::generic_err("Signature verification failed".to_string())
        );
        Ok(())
    }

    pub fn verify_return_readable(&self) -> Result<String, StdError> {
        self.verify()?;
        Ok(pubkey_to_address(
            &self.pubkey,
            &self.hrp().expect("must have prefix"),
        )?)
    }
}

pub fn preamble_msg_arb_036(signer: &str, data: &str) -> String {
    format!(
        "{{\"account_number\":\"0\",\"chain_id\":\"\",\"fee\":{{\"amount\":[],\"gas\":\"0\"}},\"memo\":\"\",\"msgs\":[{{\"type\":\"sign/MsgSignData\",\"value\":{{\"data\":\"{}\",\"signer\":\"{}\"}}}}],\"sequence\":\"0\"}}", 
        data, signer
    )
}

pub fn pubkey_to_address(pubkey: &[u8], hrp: &str) -> Result<String, StdError> {
    let base32_addr = ripemd160(&sha256(pubkey));
    let account: String = bech32::encode::<Bech32>(
        Hrp::parse(hrp).map_err(|e| StdError::generic_err(e.to_string()))?,
        &base32_addr,
    )
    .map_err(|e| StdError::generic_err(e.to_string()))?;
    Ok(account)
}

pub fn sha256(msg: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(msg);
    hasher.finalize().to_vec()
}

pub fn ripemd160(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}
