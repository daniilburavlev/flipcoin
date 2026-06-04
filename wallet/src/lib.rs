use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use common::{AppResult, error::AppError};
use libp2p::identity::secp256k1::{Keypair, PublicKey, SecretKey};

#[derive(Clone)]
pub struct Wallet {
    keypair: Keypair,
}

impl Wallet {
    pub fn from_secret(secret: [u8; 32]) -> AppResult<Self> {
        let secret = SecretKey::try_from_bytes(secret).map_err(|_| AppError::InvalidSecretKey)?;
        Ok(Self {
            keypair: Keypair::from(secret),
        })
    }

    pub fn from_secret_str(secret: String) -> AppResult<Self> {
        let secret = bs58::decode(secret)
            .into_vec()
            .map_err(|_| AppError::InvalidSecretKey)?;
        let secret: [u8; 32] = secret.try_into().map_err(|_| AppError::InvalidSecretKey)?;
        Self::from_secret(secret)
    }

    pub fn sign(&self, data: &[u8; 32]) -> AppResult<String> {
        Ok(bs58::encode(self.keypair.secret().sign(data)).into_string())
    }

    pub fn address(&self) -> [u8; 33] {
        self.keypair.public().to_bytes()
    }

    pub fn address_str(&self) -> String {
        bs58::encode(self.address()).into_string()
    }

    pub fn keypair(&self) -> Keypair {
        self.keypair.clone()
    }

    pub fn secret(&self) -> [u8; 32] {
        self.keypair.secret().to_bytes()
    }

    pub fn read(path: &Path) -> AppResult<Self> {
        let secret = fs::read_to_string(path)?;
        let secret: [u8; 32] = serde_json::from_str(&secret).map_err(|_| AppError::Decode)?;
        Self::from_secret(secret)
    }

    pub fn write(&self, path: &Path) -> AppResult<()> {
        let secret = self.secret();
        let secret = serde_json::to_string(&secret).map_err(|_| AppError::Decode)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        write!(file, "{}", secret)?;
        Ok(())
    }
}

impl Default for Wallet {
    fn default() -> Self {
        let secret = SecretKey::generate();
        let keypair = Keypair::from(secret);
        Self { keypair }
    }
}

pub fn verify_signature(
    public_key: &String,
    signature: &String,
    data: &[u8; 32],
) -> AppResult<bool> {
    let public_key = bs58::decode(public_key)
        .into_vec()
        .map_err(|_| AppError::InvalidPublicKey)?;
    let public_key: [u8; 33] = public_key
        .try_into()
        .map_err(|_| AppError::InvalidPublicKey)?;
    match PublicKey::try_from_bytes(&public_key) {
        Ok(public_key) => match bs58::decode(signature).into_vec() {
            Ok(signature) => Ok(public_key.verify(data, &signature)),
            Err(_) => Ok(false),
        },
        Err(_) => Ok(false),
    }
}
