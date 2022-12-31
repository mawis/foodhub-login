use chrono::{Duration, Utc};
use hmac::{Hmac, NewMac};
use jwt::{AlgorithmType, SignWithKey, Token};
use rocket::yansi::Color::Default;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtRaw {
    pub sub: u32,
    pub exp: i64,
}

pub fn generate_jwt(secret: &str) -> String {
    let hex_key = hex::decode(secret).expect("Decoding key from HEX failed");
    let key: Hmac<Sha256> = Hmac::new_from_slice(&hex_key).unwrap();
    let header = jwt::Header {
        algorithm: AlgorithmType::Hs256,
        key_id: None,
        type_: None,
        content_type: None,
    };
    let entity = JwtRaw {
        sub: 0,
        exp: (Utc::now() + Duration::hours(24)).timestamp(),
    };
    let token = Token::new(header, entity)
        .sign_with_key(&key)
        .expect("generated JWT");
    token.as_str().to_string()
}
