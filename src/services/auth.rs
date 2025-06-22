use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString},
    Argon2, PasswordVerifier,
};

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    PasswordHash::new(hash)
        .map(|parsed_hash| Argon2::default().verify_password(password.as_bytes(), &parsed_hash))
        .is_ok_and(|res| res.is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_password_correct() {
        let password = "test_password";
        let hash = hash_password(password);

        assert!(verify_password(&hash, password));
    }

    #[test]
    fn verify_password_failed() {
        let password = "test_password";
        let hash = hash_password(password);

        assert!(!verify_password(&hash, "wrong_password"));
    }
}
