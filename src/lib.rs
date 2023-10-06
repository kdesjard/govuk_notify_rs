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
//! async fn mailer() {
//!     let api_key = String::from("my_test_key-26785a09-ab16-4eb0-8407-a37497a57506-3d844edf-8d35-48ac-975b-e847b4f122b0");
//!     let notify_client = NotifyClient::new(api_key);
//!     let mut personalisation = Map::new();
//!     let mut personalisation_values = Map::new();
//!     personalisation_values.insert("my_var".to_string(), Value::String("my value".to_string()));
//!     personalisation.insert("personalisation".to_string(), Value::Object(personalisation_values));
//!     let email_address = String::from("john.doe@example.com");
//!     let template_id = String::from("217a419e-6a7d-482a-9596-718b889dffce");
//!
//!     notify_client.send_email(email_address, template_id, Some(personalisation), None).await;
//! }
//!
//! async fn texter() {
//!     let api_key = String::from("my_test_key-26785a09-ab16-4eb0-8407-a37497a57506-3d844edf-8d35-48ac-975b-e847b4f122b0");
//!     let notify_client = NotifyClient::new(api_key);
//!     let phone_number = String::from("+447900900123");
//!     let template_id = String::from("217a419e-6a7d-482a-9596-718b889dffce");
//!
//!     notify_client.send_sms(phone_number, template_id, None, None, None).await;
//! }
//! ```

mod auth;

use reqwest;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde_json::{Map, Value};

static DEFAULT_BASE_URL: &str = "https://api.notifications.service.gov.uk";

pub struct NotifyClient {
    notify_server: String,
    api_key: String,
    client: reqwest::Client,
}

enum NotificationType {
    EMAIL,
    SMS,
}

impl NotifyClient {
    pub fn new(api_key: String, notify_server: Option<String>) -> Self {
        let notify_server = match notify_server {
            Some(s) => s,
            None    => DEFAULT_BASE_URL.to_string(),
        };
        NotifyClient {
            notify_server,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_email(
        &self,
        email_address: String,
        template_id: String,
        personalisation: Option<Map<String, Value>>,
        reference: Option<String>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut body = Map::new();
        body.insert("email_address".to_string(), Value::String(email_address));
        body.insert("template_id".to_string(), Value::String(template_id));

        self.send_notification(
            NotificationType::EMAIL,
            body,
            personalisation,
            reference,
            None,
        )
        .await
    }

    pub async fn send_sms(
        &self,
        phone_number: String,
        template_id: String,
        personalisation: Option<Map<String, Value>>,
        reference: Option<String>,
        sms_sender_id: Option<String>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut body = Map::new();
        body.insert("phone_number".to_string(), Value::String(phone_number));
        body.insert("template_id".to_string(), Value::String(template_id));

        self.send_notification(
            NotificationType::SMS,
            body,
            personalisation,
            reference,
            sms_sender_id,
        )
        .await
    }

    async fn send_notification(
        &self,
        notification_type: NotificationType,
        mut body: Map<String, Value>,
        personalisation: Option<Map<String, Value>>,
        reference: Option<String>,
        sms_sender_id: Option<String>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = match notification_type {
            NotificationType::EMAIL => "/v2/notifications/email",
            NotificationType::SMS => "/v2/notifications/sms",
        };
        let token = auth::create_jwt(&self.api_key).unwrap();
        let auth_header: &str = &["Bearer ", token.as_str()].concat();

        if let Some(p) = personalisation {
            body.insert("personalisation".to_string(), Value::Object(p));
        }

        if let Some(r) = reference {
            body.insert("reference".to_string(), Value::String(r));
        }

        if let Some(s_id) = sms_sender_id {
            body.insert("sms_sender_id".to_string(), Value::String(s_id));
        }

        self.client
            .post(self.notify_server.clone() + url)
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
        let template_id = String::from("782fa66b-e092-4806-90d6-16782a791eb0");
        let mut personalisation = Map::new();
        personalisation.insert(
            "my_variable".to_string(),
            Value::String("some value".to_string()),
        );
        let response = client()
            .send_email(email_address, template_id, Some(personalisation), None)
            .await
            .unwrap();
        assert_eq!(response.status(), 201)
    }

    #[tokio::test]
    async fn send_email_without_personalisation() {
        let email_address = String::from("john.doe@example.com");
        let template_id = String::from("217a419e-6a7d-482a-9596-718b889dffce");
        let reference = String::from("ref_unique_xyz");
        let response = client()
            .send_email(email_address, template_id, None, Some(reference))
            .await
            .unwrap();
        assert_eq!(response.status(), 201)
    }

    #[tokio::test]
    async fn send_sms_with_personalisation() {
        let phone_number = String::from("+447900900123");
        let template_id = String::from("a4dcf0f1-2eb4-44e7-a8a1-145801e47afe");
        let mut personalisation = Map::new();
        personalisation.insert(
            "my_variable".to_string(),
            Value::String("some value".to_string()),
        );
        let response = client()
            .send_sms(phone_number, template_id, Some(personalisation), None, None)
            .await
            .unwrap();
        assert_eq!(response.status(), 201)
    }

    #[tokio::test]
    async fn send_sms_without_personalisation() {
        let phone_number = String::from("+447900900123");
        let template_id = String::from("14306c9d-8cad-4eaf-aaa4-3dae1a1df7e2");
        let sms_sender_id = String::from("b8f5bba5-5528-4bf6-b9a8-911a257f0cd4");
        let reference = String::from("ref_unique_abc");
        let response = client()
            .send_sms(
                phone_number,
                template_id,
                None,
                Some(reference),
                Some(sms_sender_id),
            )
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
