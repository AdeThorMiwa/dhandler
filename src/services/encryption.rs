use std::sync::Arc;

use aes_gcm::{
    aead::{AeadMut, Nonce, OsRng},
    AeadCore, Aes256Gcm, Key, KeyInit,
};
use base64::{engine::general_purpose, Engine};
use di::injectable;
use loco_rs::prelude::*;
use tracing::instrument;

use crate::utils::settings::Settings;

#[injectable]
pub struct EncryptionService {
    settings: Arc<Settings>,
}

impl EncryptionService {
    /// Encrypts a token using AES-GCM with a key read from the encryption key path.
    ///
    /// # Errors
    ///
    /// Returns an error if the encryption key path cannot be read or if the token cannot be encrypted.
    #[instrument(skip(self, token))]
    pub async fn encrypt(&self, token: &str) -> Result<String> {
        let key_bytes = tokio::fs::read(&self.settings.encryption.key_path).await?;
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let mut cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher.encrypt(&nonce, token.as_ref()).map_err(|e| {
            tracing::error!("Error encrypting token: {}", e);
            Error::InternalServerError
        })?;

        let mut merged = nonce.to_vec();
        merged.extend(ciphertext);

        Ok(general_purpose::STANDARD.encode(merged))
    }

    /// Decrypts a token using AES-GCM with a key read from the encryption key path.
    ///
    /// # Errors
    ///
    /// Returns an error if the encryption key path cannot be read or if the token cannot be decrypted.
    #[instrument(skip(self))]
    pub async fn decrypt(&self, token: &str) -> Result<String> {
        let decoded = general_purpose::STANDARD.decode(token).map_err(|e| {
            tracing::error!("Error decoding token: {}", e);
            Error::InternalServerError
        })?;
        let key_bytes = tokio::fs::read(&self.settings.encryption.key_path).await?;
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let mut cipher = Aes256Gcm::new(key);
        let (nonce, ciphertext) = decoded.split_at(12);
        let nonce = Nonce::<Aes256Gcm>::from_slice(nonce);

        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|e| {
            tracing::error!("Error decrypting token: {}", e);
            Error::InternalServerError
        })?;

        String::from_utf8(plaintext).map_err(|e| {
            tracing::error!("Error converting plaintext to string: {}", e);
            Error::InternalServerError
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{services::encryption::EncryptionService, utils::testing};

    #[tokio::test]
    async fn test_encrypt() {
        let provider = testing::setup().await.expect("failed to setup provider");
        let service = provider.get::<EncryptionService>().unwrap();
        let result = service.encrypt("secret").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_decrypt() {
        let provider = testing::setup().await.expect("failed to setup provider");
        let service = provider.get::<EncryptionService>().unwrap();
        let encrypted = service.encrypt("secret").await.unwrap();
        let result = service.decrypt(&encrypted).await;
        println!("result: {:?}", result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "secret");
    }
}
