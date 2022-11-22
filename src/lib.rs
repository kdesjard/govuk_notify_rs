use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

static BASE_URL: &str = "https://api.notifications.service.gov.uk";

pub struct NotifyClient {
    api_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    iss: String,
    iat: usize,
}

impl NotifyClient {
    pub fn new(api_key: String) -> Self {
        NotifyClient { api_key }
    }

    pub async fn send_email(
        &self,
        email_address: String,
        template_id: String,
        personalisation: Option<Map<String, Value>>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();

        let token = self.create_jwt().unwrap();
        let auth_header: &str = &["Bearer ", token.as_str()].concat();

        let mut body = Map::new();
        body.insert("email_address".to_string(), Value::String(email_address));
        body.insert("template_id".to_string(), Value::String(template_id));
        match personalisation {
            Some(p) => {
                body.insert("personalisation".to_string(), Value::Object(p));
            }
            _ => {}
        }

        client
            .post(BASE_URL.to_owned() + "/v2/notifications/email")
            .header(USER_AGENT, "rust-client-pre-alpha")
            .header(AUTHORIZATION, auth_header)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
    }

    fn create_jwt(&self) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = Claims {
            iss: String::from(self.service_id()),
            iat: Utc::now().timestamp() as usize,
        };
        let header = Header::new(Algorithm::HS256);
        encode(
            &header,
            &claims,
            &EncodingKey::from_secret(self.secret_key()),
        )
    }

    fn service_id(&self) -> &str {
        &self.api_key[(self.api_key.len() - 73)..=(self.api_key.len() - 38)]
    }

    fn secret_key(&self) -> &[u8] {
        let key = &self.api_key[(self.api_key.len() - 36)..self.api_key.len()];
        key.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use std::env;
    use tokio;

    #[tokio::test]
    async fn send_email_with_personalisation() {
        let email_address = String::from("john.doe@example.com");
        let template_id = String::from("217a419e-6a7d-482a-9596-718b889dffce");
        let mut personalisation = Map::new();
        let mut personalisation_values = Map::new();
        personalisation_values.insert(
            "variables".to_string(),
            Value::String("some value".to_string()),
        );
        personalisation.insert(
            "personalisation".to_string(),
            Value::Object(personalisation_values),
        );
        let response = client()
            .send_email(email_address, template_id, Some(personalisation))
            .await
            .unwrap();
        assert_eq!(response.status(), 201)
    }

    #[tokio::test]
    async fn send_email_without_personalisation() {
        let email_address = String::from("john.doe@example.com");
        let template_id = String::from("217a419e-6a7d-482a-9596-718b889dffce");
        let response = client()
            .send_email(email_address, template_id, None)
            .await
            .unwrap();
        assert_eq!(response.status(), 201)
    }

    #[test]
    fn service_id() {
        let client = test_client();
        let service_id = client.service_id();
        assert_eq!(service_id, "26785a09-ab16-4eb0-8407-a37497a57506");
    }

    #[test]
    fn secret_key() {
        let client = test_client();
        let secret_key = client.secret_key();
        assert_eq!(secret_key, b"3d844edf-8d35-48ac-975b-e847b4f122b0");
    }

    #[cfg(test)]
    fn test_client() -> NotifyClient {
        let example_api_key = String::from(
            "my_test_key-26785a09-ab16-4eb0-8407-a37497a57506-3d844edf-8d35-48ac-975b-e847b4f122b0",
        );

        NotifyClient {
            api_key: example_api_key,
        }
    }

    #[cfg(test)]
    fn client() -> NotifyClient {
        dotenv().ok();
        let api_key = env::var("GOVUK_NOTIFY_API_KEY")
            .expect("No GOVUK_NOTIFY_API_KEY environment variable found");

        NotifyClient { api_key }
    }
}
