use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand::rngs::OsRng;

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("failed to hash password: {e}"))?
        .to_string();

    Ok(hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> anyhow::Result<bool> {
    let parsed = PasswordHash::new(password_hash)
        .map_err(|e| anyhow::anyhow!("invalid password hash format: {e}"))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
