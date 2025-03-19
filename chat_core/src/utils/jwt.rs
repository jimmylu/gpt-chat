use jwt_simple::{Error, prelude::*};

use crate::User;

const JWT_DURATION: u64 = 60 * 60 * 24 * 7; // 7 days
const JWT_ISSUER: &str = "chat_server";
const JWT_AUDIENCE: &str = "chat_web";

pub struct EncodingKey(Ed25519KeyPair);
pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    // 从 PEM 格式的字符串加载密钥
    pub fn load(pem: &str) -> Result<Self, Error> {
        let key = Ed25519KeyPair::from_pem(pem)?; // pem 是 PEM 格式的私钥字符串
        Ok(Self(key))
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, Error> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from(JWT_DURATION))
            .with_issuer(JWT_ISSUER)
            .with_audience(JWT_AUDIENCE);
        let token = self.0.sign(claims)?;
        Ok(token)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, Error> {
        let key = Ed25519PublicKey::from_pem(pem)?;
        Ok(Self(key))
    }

    #[allow(unused)]
    pub fn verify(&self, token: &str) -> Result<User, Error> {
        let mut opts = VerificationOptions {
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUDIENCE])),
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISSUER])),
            ..Default::default()
        };
        let claims = self.0.verify_token::<User>(token, Some(opts))?;
        Ok(claims.custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn jwt_sign_verify_should_work() -> Result<()> {
        let encoding_pem = include_str!("../../fixtures/encoding.pem");
        let decoding_pem = include_str!("../../fixtures/decoding.pem");

        let encoding_key = EncodingKey::load(encoding_pem)?;
        let decoding_key = DecodingKey::load(decoding_pem)?;

        let user = User::new(1, 1, "test".to_string(), "test@test.com".to_string());
        let token = encoding_key.sign(user.clone())?;

        let user1 = decoding_key.verify(&token)?;
        assert_eq!(user, user1);
        Ok(())
    }
}
