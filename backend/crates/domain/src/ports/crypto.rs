use crate::errors::DomainError;

pub trait CredentialEncryptor: Send + Sync {
    fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, DomainError>;
    fn decrypt(&self, encrypted: &[u8]) -> Result<String, DomainError>;
}
