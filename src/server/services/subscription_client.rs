use log::{info, warn};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCustomerRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCustomerResponse {
    pub customer_id: String,
}

pub struct SubscriptionClient {
    client: Client,
    subs_service_url: String,
    api_key: String,
}

impl SubscriptionClient {
    pub fn new(subs_service_url: String, api_key: String) -> Self {
        SubscriptionClient {
            client: Client::new(),
            subs_service_url,
            api_key,
        }
    }

    // TODO - Change to create_stripe_customer!
    pub async fn create_stripe_customers(
        &self,
        first_name: &str,
        last_name: &str,
        email: &str,
    ) -> anyhow::Result<CreateCustomerResponse> {
        let request_body = CreateCustomerRequest {
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            email: email.to_string(),
        };

        info!(
            "Sending request to create stripe customer to {}",
            self.subs_service_url
        );
        let response = self
            .client
            .post(format!("{}/stripe/create-customer", self.subs_service_url))
            .json(&request_body)
            .header("X-API-Key", &self.api_key) // Include the API key in the header
            .send()
            .await?;

        log::info!("Response code: {:?}", response.status());
        if !response.status().is_success() {
            let error_response = response
                .json::<serde_json::Value>()
                .await
                .unwrap_or(serde_json::json!({"error": "Unknown error", "message": "Failed to create stripe customers"}));
            let message = error_response["message"]
                .as_str()
                .unwrap_or("Failed to create stripe customer")
                .to_string();
            warn!("Customer stripe account creation failed: {}", message);
            return Err(anyhow::anyhow!(message));
        }

        let token_response = response.json::<CreateCustomerResponse>().await?;
        info!(
            "Token validated successfully for user_id: {}",
            token_response.customer_id
        );
        Ok(token_response)
    }
}
