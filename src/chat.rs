use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::io::{self, Write};

/// MCP SSE message types
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum McpMessage {
    #[serde(rename = "endpoint")]
    Endpoint {
        #[allow(dead_code)]
        uri: String,
    },
    #[serde(rename = "message")]
    Message { role: String, content: MessageContent },
    #[serde(rename = "error")]
    Error { error: String },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Deserialize)]
struct ContentPart {
    #[serde(rename = "type")]
    part_type: String,
    text: Option<String>,
}

/// MCP Chat Client for Monday.com
pub struct McpChatClient {
    client: Client,
    sse_url: String,
    auth_token: String,
    board_id: String,
}

impl McpChatClient {
    /// Create a new MCP chat client
    pub fn new(sse_url: String, auth_token: String, board_id: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            sse_url,
            auth_token,
            board_id,
        }
    }

    /// Send a chat message and get response
    pub async fn send_message(&self, message: &str) -> Result<String> {
        // Construct the prompt with board context
        let prompt = format!(
            "You are helping with Monday.com board {}. User question: {}",
            self.board_id, message
        );

        // Create the request payload
        let payload = json!({
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        // Send POST request to MCP SSE endpoint
        let response = self
            .client
            .post(&self.sse_url)
            .header("Authorization", &self.auth_token)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "MCP request failed with status: {}",
                response.status()
            ));
        }

        // Parse SSE response
        let response_text = response.text().await?;
        self.parse_sse_response(&response_text)
    }

    /// Parse SSE response and extract assistant message
    fn parse_sse_response(&self, response: &str) -> Result<String> {
        let mut assistant_message = String::new();

        for line in response.lines() {
            if line.starts_with("data: ") {
                let data = &line[6..]; // Skip "data: " prefix
                
                // Skip empty data or [DONE] marker
                if data.trim().is_empty() || data.trim() == "[DONE]" {
                    continue;
                }

                // Try to parse as JSON
                if let Ok(msg) = serde_json::from_str::<McpMessage>(data) {
                    match msg {
                        McpMessage::Message { role, content } => {
                            if role == "assistant" {
                                match content {
                                    MessageContent::Text(text) => {
                                        assistant_message.push_str(&text);
                                    }
                                    MessageContent::Parts(parts) => {
                                        for part in parts {
                                            if part.part_type == "text" {
                                                if let Some(text) = part.text {
                                                    assistant_message.push_str(&text);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        McpMessage::Error { error } => {
                            return Err(anyhow!("MCP error: {}", error));
                        }
                        _ => {}
                    }
                }
            }
        }

        if assistant_message.is_empty() {
            Err(anyhow!("No response received from MCP server"))
        } else {
            Ok(assistant_message)
        }
    }
}

/// Handle the chat command
pub async fn handle_chat_command(
    board_id: Option<String>,
    verbose: bool,
) -> Result<()> {
    // Load MCP configuration
    let mcp_config = load_mcp_config()?;
    
    // Use provided board_id or default to the hardcoded one
    let board_id = board_id.unwrap_or_else(|| "6500270039".to_string());
    
    if verbose {
        println!("Initializing chat with Monday.com board: {}", board_id);
        println!("MCP Server: {}", mcp_config.url);
    }

    let chat_client = McpChatClient::new(
        mcp_config.url,
        mcp_config.auth_token,
        board_id.clone(),
    );

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║          Monday.com Board Chat (MCP-powered)              ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!("\nBoard ID: {}", board_id);
    println!("\nYou can ask questions about the board, entries, or request actions.");
    println!("Type 'exit' or 'quit' to end the chat session.\n");

    // Chat loop
    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Check for exit commands
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("\nGoodbye! 👋");
            break;
        }

        // Skip empty input
        if input.is_empty() {
            continue;
        }

        // Show thinking indicator
        print!("Assistant: ");
        io::stdout().flush()?;

        // Send message and get response
        match chat_client.send_message(input).await {
            Ok(response) => {
                println!("{}\n", response);
            }
            Err(e) => {
                eprintln!("Error: {}\n", e);
                if verbose {
                    eprintln!("Debug info: {:?}\n", e);
                }
            }
        }
    }

    Ok(())
}

/// MCP configuration structure
#[derive(Debug, Deserialize)]
struct McpConfig {
    url: String,
    auth_token: String,
}

/// Load MCP configuration from .bob/mcp.json
fn load_mcp_config() -> Result<McpConfig> {
    let config_path = std::path::Path::new(".bob/mcp.json");
    
    if !config_path.exists() {
        return Err(anyhow!(
            "MCP configuration not found at .bob/mcp.json. Please create it with your Monday.com MCP server details."
        ));
    }

    let config_data = std::fs::read_to_string(config_path)?;
    let config_json: serde_json::Value = serde_json::from_str(&config_data)?;

    // Extract monday server configuration
    let monday_config = config_json
        .get("mcpServers")
        .and_then(|s| s.get("monday"))
        .ok_or_else(|| anyhow!("Monday.com MCP server not configured in .bob/mcp.json"))?;

    let url = monday_config
        .get("url")
        .and_then(|u| u.as_str())
        .ok_or_else(|| anyhow!("MCP server URL not found in configuration"))?
        .to_string();

    let auth_token = monday_config
        .get("headers")
        .and_then(|h| h.get("Authorization"))
        .and_then(|a| a.as_str())
        .ok_or_else(|| anyhow!("Authorization token not found in MCP configuration"))?
        .to_string();

    Ok(McpConfig { url, auth_token })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_chat_client_creation() {
        let client = McpChatClient::new(
            "https://mcp.monday.com/sse".to_string(),
            "test-token".to_string(),
            "123456".to_string(),
        );
        assert_eq!(client.board_id, "123456");
    }

    #[test]
    fn test_parse_sse_response_with_text() {
        let client = McpChatClient::new(
            "https://test.com".to_string(),
            "token".to_string(),
            "123".to_string(),
        );

        let sse_data = r#"data: {"type":"message","role":"assistant","content":"Hello, this is a test response"}

data: [DONE]
"#;

        let result = client.parse_sse_response(sse_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, this is a test response");
    }

    #[test]
    fn test_parse_sse_response_with_error() {
        let client = McpChatClient::new(
            "https://test.com".to_string(),
            "token".to_string(),
            "123".to_string(),
        );

        let sse_data = r#"data: {"type":"error","error":"Something went wrong"}
"#;

        let result = client.parse_sse_response(sse_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Something went wrong"));
    }
}

// Made with Bob
