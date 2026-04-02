use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

/// create argon2 hash that is non-deterministic (uses a salt) => every generation results in a different value that can just be verified with another hash 
pub fn hash(value: &str) -> Result<String, argon2::password_hash::Error> {
    Ok(Argon2::default().hash_password(value.as_bytes())?.to_string())
}

/// verify an argon2 hash. value has to be the plain value, hash is the existing argon2 hash
pub fn verify_hash(value: &str, hash: &str) -> argon2::password_hash::Result<()> {
    Argon2::default().verify_password(value.as_bytes(), &PasswordHash::new(&hash)?)
}