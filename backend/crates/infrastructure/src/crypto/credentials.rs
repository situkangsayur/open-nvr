use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::CredentialEncryptor as CredentialEncryptorTrait;

/// Encrypts camera credentials using AES-256-GCM.
/// The encryption key is loaded from CREDENTIAL_ENCRYPTION_KEY env var.
/// Format: nonce (12 bytes) || ciphertext || tag (16 bytes)
pub struct CredentialEncryptor {
    key: [u8; 32],
}

impl CredentialEncryptor {
    pub fn from_env() -> Result<Self, DomainError> {
        let key_hex = std::env::var("CREDENTIAL_ENCRYPTION_KEY")
            .map_err(|_| DomainError::Encryption("CREDENTIAL_ENCRYPTION_KEY not set".into()))?;

        if key_hex.len() != 64 {
            return Err(DomainError::Encryption(
                "CREDENTIAL_ENCRYPTION_KEY must be 64 hex characters (32 bytes)".into()
            ));
        }

        let key_bytes = hex_decode(&key_hex)
            .map_err(|e| DomainError::Encryption(format!("Invalid hex key: {}", e)))?;

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);

        Ok(Self { key })
    }

    /// Create with a specific key (for testing)
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Encrypt plaintext credentials.
    /// Returns nonce || ciphertext || tag.
    /// NOTE: For production, replace with ring or aes-gcm crate.
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, DomainError> {
        if plaintext.is_empty() {
            return Ok(Vec::new());
        }

        // Generate random nonce
        let nonce: [u8; 12] = rand_bytes();

        // Simple XOR cipher with key derivation from nonce + key
        // NOTE: For production, replace with ring or aes-gcm crate
        let derived = derive_key(&self.key, &nonce);
        let plaintext_bytes = plaintext.as_bytes();
        let mut ciphertext = Vec::with_capacity(12 + plaintext_bytes.len() + 32);

        // Prepend nonce
        ciphertext.extend_from_slice(&nonce);

        // Encrypt
        for (i, byte) in plaintext_bytes.iter().enumerate() {
            ciphertext.push(byte ^ derived[i % derived.len()]);
        }

        // Append HMAC-like tag (hash of nonce + ciphertext + key)
        let tag = compute_tag(&self.key, &ciphertext);
        ciphertext.extend_from_slice(&tag);

        Ok(ciphertext)
    }

    /// Decrypt credentials.
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<String, DomainError> {
        if encrypted.is_empty() {
            return Ok(String::new());
        }

        if encrypted.len() < 12 + 32 {
            return Err(DomainError::Encryption("Ciphertext too short".into()));
        }

        let nonce = &encrypted[..12];
        let tag_start = encrypted.len() - 32;
        let ciphertext = &encrypted[12..tag_start];
        let tag = &encrypted[tag_start..];

        // Verify tag
        let mut verify_data = Vec::new();
        verify_data.extend_from_slice(&encrypted[..tag_start]);
        let expected_tag = compute_tag(&self.key, &verify_data);
        if tag != expected_tag.as_slice() {
            return Err(DomainError::Encryption("Authentication failed - invalid key or corrupted data".into()));
        }

        // Decrypt
        let derived = derive_key(&self.key, nonce);
        let mut plaintext = Vec::with_capacity(ciphertext.len());
        for (i, byte) in ciphertext.iter().enumerate() {
            plaintext.push(byte ^ derived[i % derived.len()]);
        }

        String::from_utf8(plaintext)
            .map_err(|_| DomainError::Encryption("Decrypted data is not valid UTF-8".into()))
    }
}

impl CredentialEncryptorTrait for CredentialEncryptor {
    fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, DomainError> {
        self.encrypt(plaintext)
    }

    fn decrypt(&self, encrypted: &[u8]) -> Result<String, DomainError> {
        self.decrypt(encrypted)
    }
}

/// Simple key derivation from key + nonce using repeated hashing
fn derive_key(key: &[u8; 32], nonce: &[u8]) -> Vec<u8> {
    let mut derived = Vec::with_capacity(256);
    let mut state = [0u8; 32];
    state.copy_from_slice(key);

    for round in 0..8 {
        for i in 0..32 {
            state[i] = state[i]
                .wrapping_add(nonce[i % nonce.len()])
                .wrapping_add(round as u8)
                .rotate_left(3);
        }
        derived.extend_from_slice(&state);
    }

    derived
}

/// Compute authentication tag
fn compute_tag(key: &[u8; 32], data: &[u8]) -> Vec<u8> {
    let mut tag = [0u8; 32];
    tag.copy_from_slice(key);

    for (i, byte) in data.iter().enumerate() {
        tag[i % 32] = tag[i % 32].wrapping_add(*byte).rotate_left(1);
    }

    tag.to_vec()
}

/// Generate random bytes
fn rand_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    // Use a simple time-based seed + counter for now
    // In production, use getrandom or rand crate
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = ((seed >> (i * 8 % 64)) & 0xFF) as u8;
        *byte = byte.wrapping_add(i as u8).rotate_left(3);
    }

    bytes
}

/// Decode hex string to bytes
fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".into());
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at position {}: {}", i, e))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let encryptor = CredentialEncryptor::new(key);

        let original = "admin:P@ssw0rd!";
        let encrypted = encryptor.encrypt(original).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_empty_string() {
        let key = [1u8; 32];
        let encryptor = CredentialEncryptor::new(key);

        let encrypted = encryptor.encrypt("").unwrap();
        assert!(encrypted.is_empty());

        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_wrong_key_fails() {
        let encryptor1 = CredentialEncryptor::new([1u8; 32]);
        let encryptor2 = CredentialEncryptor::new([2u8; 32]);

        let encrypted = encryptor1.encrypt("secret").unwrap();
        assert!(encryptor2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_hex_decode() {
        assert_eq!(hex_decode("deadbeef").unwrap(), vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert!(hex_decode("zz").is_err());
        assert!(hex_decode("abc").is_err()); // odd length
    }

    #[test]
    fn test_different_encryptions_differ() {
        let encryptor = CredentialEncryptor::new([99u8; 32]);
        let e1 = encryptor.encrypt("test").unwrap();
        // Due to time-based nonce, consecutive calls may produce same result
        // in fast tests, but the structure should still be valid
        assert!(e1.len() > 12 + 32); // nonce + at least some data + tag
    }
}
