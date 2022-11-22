use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    iss: String,
    iat: usize,
}

pub fn create_jwt(api_key: &String) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        iss: String::from(service_id(&api_key)),
        iat: Utc::now().timestamp() as usize,
    };
    let header = Header::new(Algorithm::HS256);
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret_key(&api_key)),
    )
}

fn service_id(api_key: &str) -> String {
    api_key[(api_key.len() - 73)..=(api_key.len() - 38)].to_string()
}

fn secret_key(api_key: &str) -> &[u8] {
    api_key[(api_key.len() - 36)..api_key.len()].as_bytes()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_service_id() {
        let api_key = String::from(
            "my_test_key-26785a09-ab16-4eb0-8407-a37497a57506-3d844edf-8d35-48ac-975b-e847b4f122b0",
        );
        assert_eq!(service_id(&api_key), "26785a09-ab16-4eb0-8407-a37497a57506");
    }

    #[test]
    fn test_secret_key() {
        let api_key = String::from(
            "my_test_key-26785a09-ab16-4eb0-8407-a37497a57506-3d844edf-8d35-48ac-975b-e847b4f122b0",
        );
        assert_eq!(
            secret_key(&api_key),
            b"3d844edf-8d35-48ac-975b-e847b4f122b0"
        );
    }
}
