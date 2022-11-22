# GOV.UK Notify Rust client (Pre-Alpha)

Use this (unofficial) client to send emails and SMS text messages using the [GOV.UK Notify](https://www.notifications.service.gov.uk) API in Rust.

Not yet implemented:
- Emailing files
- Sending physical letters

## Usage

```rust
 use govuk_notify::NotifyClient;
 use serde_json::{Map, Value};

 async fn mailer() {
     let api_key = String::from("my_test_key-26785a09-ab16-4eb0-8407-a37497a57506-3d844edf-8d35-48ac-975b-e847b4f122b0");
     let notify_client = NotifyClient::new(api_key);
     let mut personalisation = Map::new();
     let mut personalisation_values = Map::new();
     personalisation_values.insert("my_var".to_string(), Value::String("my value".to_string()));
     personalisation.insert("personalisation".to_string(), Value::Object(personalisation_values));
     let email_address = String::from("john.doe@example.com");
     let template_id = String::from("217a419e-6a7d-482a-9596-718b889dffce");

     notify_client.send_email(email_address, template_id, Some(personalisation)).await;
 }

 async fn texter() {
     let api_key = String::from("my_test_key-26785a09-ab16-4eb0-8407-a37497a57506-3d844edf-8d35-48ac-975b-e847b4f122b0");
     let notify_client = NotifyClient::new(api_key);
     let phone_number = String::from("+447900900123");
     let template_id = String::from("217a419e-6a7d-482a-9596-718b889dffce");

     notify_client.send_sms(phone_number, template_id, None).await;
 }
```
