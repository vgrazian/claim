use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::{Result, anyhow};
use serde_json::Value;

#[derive(Debug, Serialize)]
struct MondayRequest {
    query: String,
}

#[derive(Debug, Deserialize)]
struct MondayResponse {
    data: MondayData,
    #[serde(default)]
    errors: Vec<MondayError>,
}

#[derive(Debug, Deserialize)]
struct MondayData {
    me: MondayUser,
}

#[derive(Debug, Deserialize)]
pub struct MondayUser {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
struct MondayError {
    message: String,
    #[serde(default)]
    error_code: String,
}

pub struct MondayClient {
    client: Client,
    api_key: String,
}

// Custom deserializer to handle both string and integer IDs
fn deserialize_id<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    
    match value {
        Value::Number(num) => num
            .as_i64()
            .ok_or_else(|| serde::de::Error::custom("Invalid integer ID")),
        Value::String(s) => s
            .parse::<i64>()
            .map_err(|_| serde::de::Error::custom("Invalid string ID")),
        _ => Err(serde::de::Error::custom("ID must be a string or integer")),
    }
}

impl MondayClient {
    pub fn new(api_key: String) -> Self {
        MondayClient {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get_current_user(&self) -> Result<MondayUser> {
        let query = r#"
        {
            me {
                id
                name
                email
            }
        }
        "#;

        let request_body = MondayRequest {
            query: query.to_string(),
        };

        let response = self.client
            .post("https://api.monday.com/v2")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .header("API-Version", "2023-10")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Monday.com: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Monday.com API error ({}): {}", status, error_text));
        }

        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to read response text: {}", e))?;

        // Debug: print the raw response (first 200 characters)
        if response_text.len() > 200 {
            let preview = format!("{}...", &response_text[..200]);
            println!("Raw API response: {}", preview);
        } else {
            println!("Raw API response: {}", response_text);
        }

        let monday_response: MondayResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Monday.com response: {}", e))?;

        // Check for API errors
        if !monday_response.errors.is_empty() {
            let error_messages: Vec<String> = monday_response.errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!("Monday.com API errors: {}", error_messages.join(", ")));
        }

        Ok(monday_response.data.me)
    }

    pub async fn test_connection(&self) -> Result<()> {
        self.get_current_user().await?;
        Ok(())
    }
}