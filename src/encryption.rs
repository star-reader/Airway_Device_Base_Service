use crate::error::{AeroBaseError, Result};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use zeroize::Zeroize;


/// - 密钥派生：使用PBKDF2从密码派生加密密钥
const AES_KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const RSA_KEY_BITS: usize = 2048;
const PBKDF2_ITERATIONS: u32 = 100_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: String,
    pub nonce: String,
    pub algorithm: String,
}

pub struct RsaKeyPair {
    pub public_key: RsaPublicKey,
    private_key: RsaPrivateKey,
}

impl RsaKeyPair {
    pub fn generate() -> Result<Self> {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, RSA_KEY_BITS)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("RSA密钥生成失败: {}", e)))?;
        let public_key = RsaPublicKey::from(&private_key);

        Ok(Self {
            public_key,
            private_key,
        })
    }

    pub fn from_pem(pem: &str) -> Result<Self> {
        use rsa::pkcs8::DecodePrivateKey;
        let private_key = RsaPrivateKey::from_pkcs8_pem(pem)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("PEM导入失败: {}", e)))?;
        let public_key = RsaPublicKey::from(&private_key);

        Ok(Self {
            public_key,
            private_key,
        })
    }

    pub fn to_pem(&self) -> Result<String> {
        use rsa::pkcs8::EncodePrivateKey;
        self.private_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .map(|s| s.to_string())
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("PEM导出失败: {}", e)))
    }

    pub fn public_key_to_pem(&self) -> Result<String> {
        use rsa::pkcs8::EncodePublicKey;
        self.public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("公钥PEM导出失败: {}", e)))
    }
}

impl Drop for RsaKeyPair {
    fn drop(&mut self) {
        // 注意：RsaPrivateKey内部已经实现了Zeroize
    }
}

impl fmt::Debug for RsaKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RsaKeyPair")
            .field("public_key", &"<present>")
            .field("private_key", &"<redacted>")
            .finish()
    }
}

pub struct AesEncryptor {
    key: Vec<u8>,
}

impl AesEncryptor {
    pub fn new() -> Result<Self> {
        let mut key = vec![0u8; AES_KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        Ok(Self { key })
    }

    pub fn from_key(key: Vec<u8>) -> Result<Self> {
        if key.len() != AES_KEY_SIZE {
            return Err(AeroBaseError::DeviceFingerprint(format!(
                "无效的AES密钥长度: 期望{}, 实际{}",
                AES_KEY_SIZE,
                key.len()
            )));
        }
        Ok(Self { key })
    }

    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self> {
        use sha2::Sha256;
        
        let mut key = vec![0u8; AES_KEY_SIZE];
        pbkdf2::pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            salt,
            PBKDF2_ITERATIONS,
            &mut key,
        );
        
        Ok(Self { key })
    }

    pub fn get_key(&self) -> Vec<u8> {
        self.key.clone()
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("AES加密失败: {}", e)))?;

        Ok(EncryptedData {
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            nonce: general_purpose::STANDARD.encode(&nonce_bytes),
            algorithm: "AES-256-GCM".to_string(),
        })
    }

    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        if encrypted.algorithm != "AES-256-GCM" {
            return Err(AeroBaseError::DeviceFingerprint(format!(
                "不支持的加密算法: {}",
                encrypted.algorithm
            )));
        }

        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted.nonce)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("Nonce解码失败: {}", e)))?;
        let ciphertext = general_purpose::STANDARD
            .decode(&encrypted.ciphertext)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("密文解码失败: {}", e)))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("AES解密失败: {}", e)))?;

        Ok(plaintext)
    }

    pub fn encrypt_string(&self, plaintext: &str) -> Result<EncryptedData> {
        self.encrypt(plaintext.as_bytes())
    }

    pub fn decrypt_string(&self, encrypted: &EncryptedData) -> Result<String> {
        let plaintext = self.decrypt(encrypted)?;
        String::from_utf8(plaintext)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("UTF-8解码失败: {}", e)))
    }
}

impl Drop for AesEncryptor {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

impl Default for AesEncryptor {
    fn default() -> Self {
        Self::new().expect("Failed to create default AesEncryptor")
    }
}

pub struct RsaEncryptor {
    key_pair: RsaKeyPair,
}

impl RsaEncryptor {
    pub fn new() -> Result<Self> {
        let key_pair = RsaKeyPair::generate()?;
        Ok(Self { key_pair })
    }

    pub fn from_keypair(key_pair: RsaKeyPair) -> Self {
        Self { key_pair }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut rng = rand::thread_rng();
        self.key_pair
            .public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, plaintext)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("RSA加密失败: {}", e)))
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        self.key_pair
            .private_key
            .decrypt(Pkcs1v15Encrypt, ciphertext)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("RSA解密失败: {}", e)))
    }

    pub fn encrypt_string(&self, plaintext: &str) -> Result<String> {
        let ciphertext = self.encrypt(plaintext.as_bytes())?;
        Ok(general_purpose::STANDARD.encode(&ciphertext))
    }

    pub fn decrypt_string(&self, ciphertext_b64: &str) -> Result<String> {
        let ciphertext = general_purpose::STANDARD
            .decode(ciphertext_b64)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("Base64解码失败: {}", e)))?;
        let plaintext = self.decrypt(&ciphertext)?;
        String::from_utf8(plaintext)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("UTF-8解码失败: {}", e)))
    }

    pub fn public_key_pem(&self) -> Result<String> {
        self.key_pair.public_key_to_pem()
    }

    pub fn private_key_pem(&self) -> Result<String> {
        self.key_pair.to_pem()
    }

    pub fn encrypt_aes_key(&self, aes_key: &[u8]) -> Result<String> {
        let encrypted = self.encrypt(aes_key)?;
        Ok(general_purpose::STANDARD.encode(&encrypted))
    }

    pub fn decrypt_aes_key(&self, encrypted_key_b64: &str) -> Result<Vec<u8>> {
        let encrypted_key = general_purpose::STANDARD
            .decode(encrypted_key_b64)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("Base64解码失败: {}", e)))?;
        self.decrypt(&encrypted_key)
    }
}

impl Default for RsaEncryptor {
    fn default() -> Self {
        Self::new().expect("Failed to create default RsaEncryptor")
    }
}

/// AES + RSA
pub struct HybridEncryptor {
    aes: AesEncryptor,
    rsa: RsaEncryptor,
}

impl HybridEncryptor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            aes: AesEncryptor::new()?,
            rsa: RsaEncryptor::new()?,
        })
    }

    pub fn with_rsa_keypair(key_pair: RsaKeyPair) -> Result<Self> {
        Ok(Self {
            aes: AesEncryptor::new()?,
            rsa: RsaEncryptor::from_keypair(key_pair),
        })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(EncryptedData, String)> {
        let encrypted_data = self.aes.encrypt(plaintext)?;
        let encrypted_key = self.rsa.encrypt_aes_key(&self.aes.get_key())?;
        Ok((encrypted_data, encrypted_key))
    }

    pub fn decrypt(&self, encrypted_data: &EncryptedData, encrypted_key: &str) -> Result<Vec<u8>> {
        let aes_key = self.rsa.decrypt_aes_key(encrypted_key)?;
        let temp_aes = AesEncryptor::from_key(aes_key)?;
        temp_aes.decrypt(encrypted_data)
    }

    pub fn encrypt_string(&self, plaintext: &str) -> Result<(EncryptedData, String)> {
        self.encrypt(plaintext.as_bytes())
    }

    pub fn decrypt_string(&self, encrypted_data: &EncryptedData, encrypted_key: &str) -> Result<String> {
        let plaintext = self.decrypt(encrypted_data, encrypted_key)?;
        String::from_utf8(plaintext)
            .map_err(|e| AeroBaseError::DeviceFingerprint(format!("UTF-8解码失败: {}", e)))
    }

    pub fn public_key_pem(&self) -> Result<String> {
        self.rsa.public_key_pem()
    }

    pub fn private_key_pem(&self) -> Result<String> {
        self.rsa.private_key_pem()
    }
}

impl Default for HybridEncryptor {
    fn default() -> Self {
        Self::new().expect("Failed to create default HybridEncryptor")
    }
}

pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// SHA-256
pub fn hash_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption() {
        let encryptor = AesEncryptor::new().unwrap();
        let plaintext = "Hello, World!";

        let encrypted = encryptor.encrypt_string(plaintext).unwrap();
        let decrypted = encryptor.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_rsa_encryption() {
        let encryptor = RsaEncryptor::new().unwrap();
        let plaintext = "Secret message";

        let encrypted = encryptor.encrypt_string(plaintext).unwrap();
        let decrypted = encryptor.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_hybrid_encryption() {
        let encryptor = HybridEncryptor::new().unwrap();
        let plaintext = "This is a long message that needs hybrid encryption!";

        let (encrypted_data, encrypted_key) = encryptor.encrypt_string(plaintext).unwrap();
        let decrypted = encryptor.decrypt_string(&encrypted_data, &encrypted_key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_password_based_encryption() {
        let password = "my_secure_password";
        let salt = generate_salt();

        let encryptor = AesEncryptor::from_password(password, &salt).unwrap();
        let plaintext = "Sensitive data";

        let encrypted = encryptor.encrypt_string(plaintext).unwrap();
        
        // 使用相同密码和盐值创建新的加密器
        let decryptor = AesEncryptor::from_password(password, &salt).unwrap();
        let decrypted = decryptor.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_key_pem_export_import() {
        let original = RsaEncryptor::new().unwrap();
        let pem = original.private_key_pem().unwrap();

        let key_pair = RsaKeyPair::from_pem(&pem).unwrap();
        let imported = RsaEncryptor::from_keypair(key_pair);

        let plaintext = "Test message";
        let encrypted = original.encrypt_string(plaintext).unwrap();
        let decrypted = imported.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_hash_sha256() {
        let data = b"test data";
        let hash = hash_sha256(data);
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
    }
}
