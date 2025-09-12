mod browse;
mod pdk;
mod searxng;

use crate::browse::browse;
use crate::searxng::{SearXNGClient, SearXNGConfig};
use extism_pdk::*;
use pdk::types::*;
use serde_json::{Value, json};

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "search" => search(input),
        "browse" => browse_tool(input),
        _ => Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(format!("Unknown tool: {}", input.params.name)),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        }),
    }
}

fn search(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let query = match args.get("query") {
        Some(Value::String(q)) if !q.is_empty() => q,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Please provide a non-empty query string".into()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    let config = SearXNGConfig::default();
    let client = SearXNGClient::new(config);
    match client.test_connection() {
        Ok(true) => match client.simple_search(query) {
            Ok(response) => Ok(CallToolResult {
                is_error: None,
                content: vec![Content {
                    annotations: None,
                    text: Some(
                        serde_json::to_string(&response)
                            .unwrap_or_else(|_| "Serialization error".into()),
                    ),
                    mime_type: Some("application/json".into()),
                    r#type: ContentType::Text,
                    data: None,
                }],
            }),
            Err(e) => Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Search failed: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }),
        },
        Ok(false) => Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some("Unable to connect to SearXNG server".into()),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        }),
        Err(e) => Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(format!("Connection test failed: {}", e)),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        }),
    }
}

fn browse_tool(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let url = match args.get("url") {
        Some(Value::String(u)) if !u.is_empty() => u,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Please provide a non-empty url string".into()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    match browse(url) {
        Ok(html) => Ok(CallToolResult {
            is_error: None,
            content: vec![Content {
                annotations: None,
                text: Some(html),
                mime_type: Some("text/markdown".into()),
                r#type: ContentType::Text,
                data: None,
            }],
        }),
        Err(e) => Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(format!("Browse failed: {}", e)),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        }),
    }
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    // Log available engines on plugin load
    let config = SearXNGConfig::default();
    let client = SearXNGClient::new(config);
    match client.get_engines(crate::searxng::EngineFilter::Enabled) {
        Ok(engines) => {
            let engine_list = engines
                .keys()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            info!("Available SearXNG engines: {}", engine_list);
        }
        Err(e) => {
            warn!("Failed to fetch SearXNG engines: {}", e);
        }
    }

    Ok(ListToolsResult {
        tools: vec![
            ToolDescription {
                name: "search".into(),
                description: "Perform web search using SearXNG".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query",
                        },
                    },
                    "required": ["query"],
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "browse".into(),
                description: "Fetch content from a URL as Markdown".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to browse",
                        },
                    },
                    "required": ["url"],
                })
                .as_object()
                .unwrap()
                .clone(),
            },
        ],
    })
}
