# GOV.UK Notify Rust client (Pre-Alpha)

Use this (unofficial) client to send emails using the [GOV.UK Notify](https://www.notifications.service.gov.uk) API in Rust.

SMS and Letter sending functionality is not yet implemented.

## Usage

```rust
use govuk_notify::NotifyClient;
use serde_json::{Map, Value};

let notify_client = NotifyClient::new(api_key);
let mut personalisation = Map::new();
let mut personalisation_values = Map::new();
personalisation_values.insert("my_var".to_string(), Value::String("my value".to_string()));
personalisation.insert("personalisation".to_string(), Value::Object(personalisation_values));

notify_client.send_email(email, template_id, Some(personalisation)).await
```
