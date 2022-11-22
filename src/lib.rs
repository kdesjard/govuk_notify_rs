//! # GOV.UK Notify Rust client
//!
//! Use this (unofficial) client to send emails using the
//! [GOV.UK Notify](https://www.notifications.service.gov.uk) API. See the linked documentation to
//! obtain an API key.
//
//! ```rust
//! use govuk_notify::NotifyClient;
//! use serde_json::{Map, Value};
//!
//! async fn send_notification() {
//!     let api_key = String::from("my_test_key-26785a09-ab16-4eb0-8407-a37497a57506-3d844edf-8d35-48ac-975b-e847b4f122b0");
//!     let notify_client = NotifyClient::new(api_key);
//!     let mut personalisation = Map::new();
//!     let mut personalisation_values = Map::new();
//!     personalisation_values.insert("my_var".to_string(), Value::String("my value".to_string()));
//!     personalisation.insert("personalisation".to_string(), Value::Object(personalisation_values));
//!     let email_address = String::from("john.doe@example.com");
//!     let template_id = String::from("217a419e-6a7d-482a-9596-718b889dffce");
//!
//!     notify_client.send_email(email_address, template_id, Some(personalisation)).await;
//! }
//! ```

mod auth;

use reqwest;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde_json::{Map, Value};

static BASE_URL: &str = "https://api.notifications.service.gov.uk";

pub struct NotifyClient {
    api_key: String,
    client: reqwest::Client,
}

enum NotificationType {
    EMAIL,
    SMS,
}

impl NotifyClient {
    pub fn new(api_key: String) -> Self {
        NotifyClient {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_email(
        &self,
        email_address: String,
        template_id: String,
        personalisation: Option<Map<String, Value>>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut body = Map::new();
        body.insert("email_address".to_string(), Value::String(email_address));
        body.insert("template_id".to_string(), Value::String(template_id));

        self.send_notification(NotificationType::EMAIL, body, personalisation)
            .await
    }

    pub async fn send_sms(
        &self,
        phone_number: String,
        template_id: String,
        personalisation: Option<Map<String, Value>>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut body = Map::new();
        body.insert("phone_number".to_string(), Value::String(phone_number));
        body.insert("template_id".to_string(), Value::String(template_id));

        self.send_notification(NotificationType::SMS, body, personalisation)
            .await
    }

    async fn send_notification(
        &self,
        notification_type: NotificationType,
        mut body: Map<String, Value>,
        personalisation: Option<Map<String, Value>>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = match notification_type {
            NotificationType::EMAIL => "/v2/notifications/email",
            NotificationType::SMS => "/v2/notifications/sms",
        };
        let token = auth::create_jwt(&self.api_key).unwrap();
        let auth_header: &str = &["Bearer ", token.as_str()].concat();

        match personalisation {
            Some(p) => {
                body.insert("personalisation".to_string(), Value::Object(p));
            }
            _ => {}
        }

        self.client
            .post(BASE_URL.to_owned() + url)
            .header(USER_AGENT, "rust-client-pre-alpha")
            .header(AUTHORIZATION, auth_header)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
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
    
    #[tokio::test]
    async fn send_sms_with_personalisation() {
        let phone_number = String::from("+447900900123");
        let template_id = String::from("a4dcf0f1-2eb4-44e7-a8a1-145801e47afe");
        let mut personalisation = Map::new();
        let mut personalisation_values = Map::new();
        personalisation_values.insert(
            "reference".to_string(),
            Value::String("some value".to_string()),
        );
        personalisation.insert(
            "personalisation".to_string(),
            Value::Object(personalisation_values),
        );
        let response = client()
            .send_sms(phone_number, template_id, Some(personalisation))
            .await
            .unwrap();
        assert_eq!(response.status(), 201)
    }

    #[cfg(test)]
    fn client() -> NotifyClient {
        dotenv().ok();
        let api_key = env::var("GOVUK_NOTIFY_API_KEY")
            .expect("No GOVUK_NOTIFY_API_KEY environment variable found");

        NotifyClient::new(api_key)
    }
}
