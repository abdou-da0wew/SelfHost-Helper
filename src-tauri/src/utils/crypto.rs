use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

const KDF_ITERATIONS: u32 = 600_000;
const KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;
const IV_LEN: usize = 12;
const CURRENT_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("passphrase is required")]
    MissingPassphrase,
    #[error("invalid backup envelope: {0}")]
    InvalidEnvelope(String),
    #[error("unsupported backup version: {0}")]
    UnsupportedVersion(u32),
    #[error("decryption failed -- check your passphrase")]
    DecryptionFailed,
    #[error("decrypted payload is invalid: {0}")]
    InvalidPayload(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("base64 error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("cipher error: {0}")]
    Cipher(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfInfo {
    pub algo: String,
    pub hash: String,
    pub iterations: u32,
    pub salt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipherInfo {
    pub algo: String,
    pub iv: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupAppMeta {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPayload {
    #[serde(default)]
    pub projects: Vec<serde_json::Value>,
    #[serde(default)]
    pub categories: Vec<serde_json::Value>,
    #[serde(default)]
    pub settings: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEnvelope {
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<BackupAppMeta>,
    pub payload: BackupPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBackup {
    pub version: u32,
    pub kdf: KdfInfo,
    pub cipher: CipherInfo,
    pub tag: String,
    pub data: String,
    pub meta: EncryptedBackupMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBackupMeta {
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<BackupAppMeta>,
}

fn derive_key(passphrase: &str, salt: &[u8], iterations: u32) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, iterations, &mut key);
    key
}

pub fn encrypt_envelope(
    envelope: &BackupEnvelope,
    passphrase: &str,
) -> Result<EncryptedBackup, CryptoError> {
    if passphrase.trim().is_empty() {
        return Err(CryptoError::MissingPassphrase);
    }
    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut iv);
    let key = derive_key(passphrase, &salt, KDF_ITERATIONS);
    let plaintext = serde_json::to_vec(envelope)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CryptoError::Cipher(e.to_string()))?;
    let nonce = Nonce::from_slice(&iv);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| CryptoError::Cipher(e.to_string()))?;
    let tag_len = 16;
    if ciphertext.len() < tag_len {
        return Err(CryptoError::Cipher("ciphertext shorter than tag".into()));
    }
    let (encrypted_data, auth_tag) = ciphertext.split_at(ciphertext.len() - tag_len);
    let created_at = envelope
        .created_at
        .clone()
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    Ok(EncryptedBackup {
        version: CURRENT_VERSION,
        kdf: KdfInfo {
            algo: "pbkdf2".into(),
            hash: "sha256".into(),
            iterations: KDF_ITERATIONS,
            salt: BASE64.encode(salt),
        },
        cipher: CipherInfo {
            algo: "aes-256-gcm".into(),
            iv: BASE64.encode(iv),
        },
        tag: BASE64.encode(auth_tag),
        data: BASE64.encode(encrypted_data),
        meta: EncryptedBackupMeta {
            created_at,
            app: envelope.app.clone(),
        },
    })
}

pub fn decrypt_backup(
    backup: &EncryptedBackup,
    passphrase: &str,
) -> Result<BackupEnvelope, CryptoError> {
    if passphrase.trim().is_empty() {
        return Err(CryptoError::MissingPassphrase);
    }
    if backup.version < 1 {
        return Err(CryptoError::UnsupportedVersion(backup.version));
    }
    let salt = BASE64.decode(&backup.kdf.salt)?;
    let iv = BASE64.decode(&backup.cipher.iv)?;
    let auth_tag = BASE64.decode(&backup.tag)?;
    let ciphertext = BASE64.decode(&backup.data)?;
    let iterations = if backup.kdf.iterations > 0 {
        backup.kdf.iterations
    } else {
        KDF_ITERATIONS
    };
    let key = derive_key(passphrase, &salt, iterations);
    let mut full_ciphertext = ciphertext;
    full_ciphertext.extend_from_slice(&auth_tag);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CryptoError::Cipher(e.to_string()))?;
    let nonce = Nonce::from_slice(&iv);
    let plaintext = cipher
        .decrypt(nonce, full_ciphertext.as_ref())
        .map_err(|_| CryptoError::DecryptionFailed)?;
    let envelope: BackupEnvelope =
        serde_json::from_slice(&plaintext).map_err(|e| CryptoError::InvalidPayload(e.to_string()))?;
    Ok(envelope)
}

pub fn encrypt_to_json(
    payload: BackupPayload,
    passphrase: &str,
    app: Option<BackupAppMeta>,
) -> Result<String, CryptoError> {
    let envelope = BackupEnvelope {
        version: CURRENT_VERSION,
        created_at: Some(Utc::now().to_rfc3339()),
        app,
        payload,
    };
    let encrypted = encrypt_envelope(&envelope, passphrase)?;
    Ok(serde_json::to_string_pretty(&encrypted)?)
}

pub fn decrypt_from_json(json: &str, passphrase: &str) -> Result<BackupEnvelope, CryptoError> {
    let backup: EncryptedBackup = serde_json::from_str(json)?;
    decrypt_backup(&backup, passphrase)
}
