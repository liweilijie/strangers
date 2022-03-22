use crate::error::AppError;
use crate::Result;
pub fn hash(pwd: &str) -> Result<String> {
    bcrypt::hash(pwd, bcrypt::DEFAULT_COST).map_err(AppError::from)
}

pub fn verify(pwd: &str, hashed_pwd: &str) -> Result<bool> {
    bcrypt::verify(pwd, hashed_pwd).map_err(AppError::from)
}

mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        let pwd = "wgr695";
        let hashed_pwd = hash(pwd).unwrap();
        println!("hashed_pwd: {}", hashed_pwd);
        // $2b$12$QW8Lmf0gvsb1xtRJLxJxzea2M2p5Pxx1LrmPuVzria5obcY8u890C
    }

    #[test]
    fn test_verify_hashed_pwd() {
        let pwd = "wgr695";
        let hashed_pwd = "$2b$12$QW8Lmf0gvsb1xtRJLxJxzea2M2p5Pxx1LrmPuVzria5obcY8u890C";
        let is_match = verify(pwd, hashed_pwd).unwrap();
        assert!(is_match);
    }
}
