//! Integration tests for provider probing functionality.

use aicred_core::error::Error;
use aicred_core::plugins::ProviderPlugin;
use aicred_core::providers::OpenRouterPlugin;
use mockito::Server;

/// Helper function to create a mock OpenRouter models response
fn create_mock_models_response() -> serde_json::Value {
    serde_json::json!({
        "data": [
            {
                "id": "openai/gpt-4",
                "name": "GPT-4",
                "description": "OpenAI's most capable model",
                "context_length": 8192,
                "pricing": {
                    "prompt": "0.00003",
                    "completion": "0.00006"
                },
                "architecture": {
                    "modality": "text",
                    "tokenizer": "cl100k_base",
                    "instruct_type": "chatml"
                }
            },
            {
                "id": "anthropic/claude-3-opus",
                "name": "Claude 3 Opus",
                "description": "Anthropic's most powerful model",
                "context_length": 200000,
                "pricing": {
                    "prompt": "0.000015",
                    "completion": "0.000075"
                },
                "architecture": {
                    "modality": "text+image",
                    "tokenizer": "claude",
                    "instruct_type": "claude"
                }
            }
        ]
    })
}

#[tokio::test]
async fn test_openrouter_probe_success() {
    let mut server = Server::new_async().await;

    // Create mock endpoint
    let mock = server
        .mock("GET", "/models")
        .match_header("authorization", "Bearer test-api-key")
        .match_header("content-type", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(create_mock_models_response().to_string())
        .create_async()
        .await;

    let plugin = OpenRouterPlugin;
    let result = plugin
        .probe_models_async("test-api-key", Some(&server.url()))
        .await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 2);

    // Verify first model
    assert_eq!(models[0].id, "openai/gpt-4");
    assert_eq!(models[0].name, "GPT-4");
    assert_eq!(models[0].context_length, Some(8192));
    assert!(models[0].pricing.is_some());

    // Verify second model
    assert_eq!(models[1].id, "anthropic/claude-3-opus");
    assert_eq!(models[1].name, "Claude 3 Opus");
    assert_eq!(models[1].context_length, Some(200000));
}

#[tokio::test]
async fn test_openrouter_probe_authentication_failure() {
    let mut server = Server::new_async().await;

    // Create mock endpoint that returns 401
    let mock = server
        .mock("GET", "/models")
        .match_header("authorization", "Bearer invalid-key")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Invalid API key"}"#)
        .create_async()
        .await;

    let plugin = OpenRouterPlugin;
    let result = plugin
        .probe_models_async("invalid-key", Some(&server.url()))
        .await;

    mock.assert_async().await;
    assert!(result.is_err());

    match result {
        Err(Error::ApiError(msg)) => {
            assert!(msg.contains("Authentication failed"));
        }
        _ => panic!("Expected ApiError for authentication failure"),
    }
}

#[tokio::test]
async fn test_openrouter_probe_malformed_response() {
    let mut server = Server::new_async().await;

    // Create mock endpoint with malformed JSON
    let mock = server
        .mock("GET", "/models")
        .match_header("authorization", "Bearer test-api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"invalid": "response"}"#)
        .create_async()
        .await;

    let plugin = OpenRouterPlugin;
    let result = plugin
        .probe_models_async("test-api-key", Some(&server.url()))
        .await;

    mock.assert_async().await;
    assert!(result.is_err());

    match result {
        Err(Error::SerializationError(_)) => {
            // Expected error type
        }
        _ => panic!("Expected SerializationError for malformed response"),
    }
}

#[tokio::test]
async fn test_openrouter_probe_network_error() {
    let plugin = OpenRouterPlugin;

    // Use an invalid URL to trigger network error
    let result = plugin
        .probe_models_async(
            "test-api-key",
            Some("http://invalid-url-that-does-not-exist.local"),
        )
        .await;

    assert!(result.is_err());

    match result {
        Err(Error::HttpError(_)) => {
            // Expected error type
        }
        _ => panic!("Expected HttpError for network failure"),
    }
}

#[tokio::test]
async fn test_openrouter_probe_server_error() {
    let mut server = Server::new_async().await;

    // Create mock endpoint that returns 500
    let mock = server
        .mock("GET", "/models")
        .match_header("authorization", "Bearer test-api-key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal server error"}"#)
        .create_async()
        .await;

    let plugin = OpenRouterPlugin;
    let result = plugin
        .probe_models_async("test-api-key", Some(&server.url()))
        .await;

    mock.assert_async().await;
    assert!(result.is_err());

    match result {
        Err(Error::ApiError(msg)) => {
            assert!(msg.contains("500"));
        }
        _ => panic!("Expected ApiError for server error"),
    }
}

#[tokio::test]
async fn test_openrouter_probe_empty_response() {
    let mut server = Server::new_async().await;

    // Create mock endpoint with empty models list
    let mock = server
        .mock("GET", "/models")
        .match_header("authorization", "Bearer test-api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"data": []}"#)
        .create_async()
        .await;

    let plugin = OpenRouterPlugin;
    let result = plugin
        .probe_models_async("test-api-key", Some(&server.url()))
        .await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 0);
}

#[tokio::test]
async fn test_openrouter_probe_with_minimal_model_data() {
    let mut server = Server::new_async().await;

    // Create mock with minimal model data
    let minimal_response = serde_json::json!({
        "data": [
            {
                "id": "minimal/model"
            }
        ]
    });

    let mock = server
        .mock("GET", "/models")
        .match_header("authorization", "Bearer test-api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(minimal_response.to_string())
        .create_async()
        .await;

    let plugin = OpenRouterPlugin;
    let result = plugin
        .probe_models_async("test-api-key", Some(&server.url()))
        .await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "minimal/model");
    assert_eq!(models[0].name, "Unknown");
    assert!(models[0].pricing.is_none());
    assert!(models[0].architecture.is_none());
}
