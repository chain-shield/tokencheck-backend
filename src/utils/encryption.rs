use aes_gcm::{
    aead::{consts::U32, generic_array::GenericArray, Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as Base64, Engine};
use rand::RngCore;

use crate::env_config::Config;

pub struct EncryptionService {
    cipher: Aes256Gcm,
}
impl EncryptionService {
    pub fn new() -> Result<Self> {
        let config = Config::from_env();

        // Decode the hexadecimal string into a 32-byte array
        let key_bytes = hex::decode(&config.encryption_key).context("Failed to decode hex key")?;

        // Ensure the key is exactly 32 bytes (256 bits)
        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!(
                "Encryption key must be 32 bytes (64 hex characters), got {} bytes",
                key_bytes.len()
            ));
        }

        // Convert the Vec<u8> into a GenericArray<u8, U32>
        let key_array: [u8; 32] = key_bytes.try_into().map_err(|e: Vec<u8>| {
            anyhow::anyhow!(
                "Failed to convert key to 32-byte array: expected 32 bytes, got {} bytes",
                e.len()
            )
        })?;
        let key = GenericArray::<u8, U32>::from(key_array);

        // Initialize the cipher
        let cipher = Aes256Gcm::new(&key);
        Ok(Self { cipher })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        // Generate a random 12-byte IV (nonce)
        let mut iv = [0u8; 12];
        OsRng.fill_bytes(&mut iv);
        let nonce = Nonce::from_slice(&iv);

        // Encrypt the plaintext
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to encrypt plaintext: {}", e))?;

        // Combine IV, ciphertext, and auth tag (last 16 bytes of ciphertext)
        let mut combined = Vec::new();
        combined.extend_from_slice(&iv); // 12 bytes
        combined.extend_from_slice(&ciphertext); // ciphertext + 16-byte auth tag

        // Encode as base64
        Ok(Base64.encode(&combined))
    }

    pub fn decrypt(&self, encrypted: &str) -> Result<String> {
        // Decode the base64 string
        let combined = Base64
            .decode(encrypted)
            .context("Failed to decode base64 string")?;

        if combined.len() < 28 {
            // 12 bytes IV + at least 1 byte ciphertext + 16 bytes auth tag
            return Err(anyhow::anyhow!("Invalid encrypted data length"));
        }

        // Split into IV and ciphertext (including auth tag)
        let iv = &combined[0..12];
        let ciphertext = &combined[12..];

        let nonce = Nonce::from_slice(iv);

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Failed to decrypt ciphertext: {}", e))?;

        String::from_utf8(plaintext).context("Failed to convert decrypted data to UTF-8")
    }
}
