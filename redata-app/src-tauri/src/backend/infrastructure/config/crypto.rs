// 加密工具模块 - 用于 API 密钥加密

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use std::env;

/// 加密错误
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed")]
    DecryptionFailed,

    #[error("Invalid key")]
    InvalidKey,

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

/// 获取加密密钥
/// 从环境变量 ENCRYPTION_KEY 读取，如果不存在则使用默认密钥（仅用于开发）
fn get_encryption_key() -> [u8; 32] {
    if let Ok(key_str) = env::var("ENCRYPTION_KEY") {
        let key_bytes = general_purpose::STANDARD
            .decode(key_str)
            .unwrap_or_else(|_| vec![0u8; 32]);

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes[..32.min(key_bytes.len())]);
        key
    } else {
        // 默认密钥（仅用于开发环境）
        // 生产环境必须设置 ENCRYPTION_KEY 环境变量
        tracing::warn!("Using default encryption key. Set ENCRYPTION_KEY environment variable for production.");
        [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
            0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
        ]
    }
}

/// 加密字符串
pub fn encrypt(plaintext: &str) -> Result<String, CryptoError> {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new(&key.into());

    // 生成随机 nonce
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 加密
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| CryptoError::EncryptionFailed)?;

    // 将 nonce 和 ciphertext 组合并编码为 base64
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(result))
}

/// 解密字符串
pub fn decrypt(encrypted: &str) -> Result<String, CryptoError> {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new(&key.into());

    // 解码 base64
    let data = general_purpose::STANDARD.decode(encrypted)?;

    if data.len() < 12 {
        return Err(CryptoError::DecryptionFailed);
    }

    // 分离 nonce 和 ciphertext
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // 解密
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    String::from_utf8(plaintext).map_err(|_| CryptoError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let original = "sk-1234567890abcdef";
        let encrypted = encrypt(original).unwrap();
        let decrypted = decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
        assert_ne!(original, encrypted);
    }

    #[test]
    fn test_decrypt_invalid() {
        let result = decrypt("invalid_base64!");
        assert!(result.is_err());
    }
}
