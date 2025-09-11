use anyhow::{Result, anyhow};
use extism_pdk::config;
use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parse comma-separated string into vector
fn parse_comma_separated_from_string(s: &str) -> Vec<String> {
    s.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Engine filter options
#[derive(Debug, Clone)]
pub enum EngineFilter {
    Enabled,
    Disabled,
    All,
}

/// Safe search options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SafeSearch {
    None = 0,
    Moderate = 1,
    Strict = 2,
}

/// SearXNG client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearXNGConfig {
    pub base_url: String,
    pub default_engine: Option<String>,
    pub default_categories: Vec<String>,
    pub default_engines: Vec<String>,
    pub language: String,
    pub safe_search: SafeSearch,
    pub user_agent: String,
    pub num_results: u32,
}

impl Default for SearXNGConfig {
    fn default() -> Self {
        let base_url = config::get("SEARXNG_BASE_URL")
            .ok()
            .flatten()
            .unwrap_or_else(|| "http://localhost:8080".to_string());
        let default_engine = config::get("SEARXNG_DEFAULT_ENGINE").ok().flatten();

        // Direct empty string handling for categories
        let default_categories_env = config::get("SEARXNG_DEFAULT_CATEGORIES")
            .ok()
            .flatten()
            .unwrap_or_default();
        let default_categories = parse_comma_separated_from_string(&default_categories_env);

        // Direct empty string handling for engines
        let default_engines_env = config::get("SEARXNG_DEFAULT_ENGINES")
            .ok()
            .flatten()
            .unwrap_or_default();
        let default_engines = parse_comma_separated_from_string(&default_engines_env);

        let language = config::get("SEARXNG_DEFAULT_LANGUAGE")
            .ok()
            .flatten()
            .unwrap_or_else(|| "en".to_string());
        let safe_search_str = config::get("SEARXNG_SAFE_SEARCH")
            .ok()
            .flatten()
            .unwrap_or_else(|| "0".to_string());
        let safe_search = match safe_search_str.as_str() {
            "0" => SafeSearch::None,
            "2" => SafeSearch::Strict,
            _ => SafeSearch::Moderate,
        };
        let user_agent = config::get("SEARXNG_USER_AGENT")
            .ok()
            .flatten()
            .unwrap_or_else(|| format!("searxng-rs/{}", VERSION));
        let num_results = config::get("SEARXNG_NUM_RESULTS")
            .ok()
            .flatten()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(5);

        info!("SearXNG base_url: {}", base_url);
        info!("SearXNG default_engine: {:?}", default_engine);
        info!("SearXNG default_categories: {:?}", default_categories);
        info!("SearXNG default_engines: {:?}", default_engines);
        info!("SearXNG language: {}", language);
        info!("SearXNG safe_search: {:?}", safe_search);
        info!("SearXNG user_agent: {}", user_agent);
        info!("SearXNG num_results: {}", num_results);

        Self {
            base_url,
            default_engine,
            default_categories,
            default_engines,
            language,
            safe_search,
            user_agent,
            num_results,
        }
    }
}

/// SearXNG search result
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    #[serde(skip_serializing)]
    pub engine: String,
    #[serde(skip_serializing)]
    pub parsed_url: Vec<String>,
    #[serde(skip_serializing)]
    pub template: String,
    #[serde(skip_serializing)]
    pub engines: Vec<String>,
    #[serde(skip_serializing)]
    pub positions: Vec<u32>,
    #[serde(skip_serializing)]
    pub score: f64,
    pub category: String,
}

/// SearXNG full response
#[derive(Debug, Serialize, Deserialize)]
pub struct SearXNGResponse {
    #[serde(skip_serializing)]
    pub query: String,
    pub results: Vec<SearchResult>,
    #[serde(skip_serializing)]
    pub number_of_results: u32,
    #[serde(skip_serializing)]
    pub answers: Vec<String>,
    #[serde(skip_serializing)]
    pub corrections: Vec<String>,
    #[serde(skip_serializing)]
    pub infoboxes: Vec<serde_json::Value>,
    pub suggestions: Vec<String>,
    #[serde(skip_serializing)]
    pub unresponsive_engines: Vec<Vec<String>>,
}

/// Query params
#[derive(Debug, Default)]
pub struct SearchParams {
    pub query: String,
    pub categories: Option<String>,
    pub engines: Option<String>,
    pub language: Option<String>,
    pub pageno: Option<u32>,
    pub time_range: Option<String>,
    pub format: Option<String>,
    pub safe_search: Option<SafeSearch>,
}

/// SearXNG client
pub struct SearXNGClient {
    config: SearXNGConfig,
}

impl SearXNGClient {
    /// New client instance
    pub fn new(config: SearXNGConfig) -> Self {
        Self { config }
    }

    /// Perform search with given parameters
    pub fn search(&self, params: SearchParams) -> Result<SearXNGResponse> {
        let mut url = Url::parse(&format!("{}/search", self.config.base_url))?;

        // Build search params
        let mut query_params = vec![("q", params.query.clone()), ("format", "json".to_string())];

        if let Some(categories) = params.categories {
            query_params.push(("categories", categories));
        }

        if let Some(engines) = params.engines {
            query_params.push(("engines", engines));
        }

        let language = params.language.as_ref().unwrap_or(&self.config.language);
        query_params.push(("language", language.clone()));

        if let Some(pageno) = params.pageno {
            query_params.push(("pageno", pageno.to_string()));
        }

        if let Some(time_range) = params.time_range {
            query_params.push(("time_range", time_range));
        }

        let safe_search = params.safe_search.unwrap_or(self.config.safe_search);
        query_params.push(("safesearch", (safe_search as u8).to_string()));

        url.query_pairs_mut().extend_pairs(query_params);

        let request = HttpRequest::new(url.as_str())
            .with_method("GET")
            .with_header("User-Agent", &self.config.user_agent);

        let response = http::request::<Vec<u8>>(&request, None)
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

        // BUG: extism_pdk sometimes returns status 0 even for successful requests
        let is_success = (200..300).contains(&response.status())
            || (response.status() == 0 && !response.body().is_empty());

        if !is_success {
            let body = String::from_utf8(response.body().to_vec())
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("HTTP Error: {} - {}", response.status(), body));
        }

        let search_response: SearXNGResponse = serde_json::from_slice(&response.body())
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

        Ok(search_response)
    }

    /// Simple search with just a query
    pub fn simple_search(&self, query: &str) -> Result<SearXNGResponse> {
        let mut params = SearchParams {
            query: query.to_string(),
            ..Default::default()
        };

        // Set default engines if configured
        if !self.config.default_engines.is_empty() {
            params.engines = Some(self.config.default_engines.join(","));
        }

        // Set default categories if configured
        if !self.config.default_categories.is_empty() {
            params.categories = Some(self.config.default_categories.join(","));
        }

        let mut response = self.search(params)?;

        // Sort results by score (highest first)
        response.results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Truncate results to configured limit
        if response.results.len() > self.config.num_results as usize {
            let original_count = response.results.len();
            response.results.truncate(self.config.num_results as usize);
            response.number_of_results = response.results.len() as u32;
            info!(
                "Results truncated from {} to {} (limit: {})",
                original_count,
                response.results.len(),
                self.config.num_results
            );
        }

        // Log the result titles and scores for debugging
        for (i, result) in response.results.iter().enumerate() {
            info!(
                "Result {}: {} (score: {:.3})",
                i + 1,
                result.title,
                result.score
            );
        }

        Ok(response)
    }

    /// Test connection
    pub fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/config", self.config.base_url);
        let request = HttpRequest::new(&url)
            .with_method("GET")
            .with_header("User-Agent", &self.config.user_agent);

        let response = http::request::<Vec<u8>>(&request, None)
            .map_err(|e| anyhow!("Connection test failed: {}", e))?;

        // BUG: extism_pdk sometimes returns status 0 even for successful requests
        let is_success = (200..300).contains(&response.status())
            || (response.status() == 0 && !response.body().is_empty());

        Ok(is_success)
    }

    /// Get available search engines
    pub fn get_engines(&self, filter: EngineFilter) -> Result<HashMap<String, serde_json::Value>> {
        let url = format!("{}/config", self.config.base_url);
        let request = HttpRequest::new(&url)
            .with_method("GET")
            .with_header("User-Agent", &self.config.user_agent);

        let response = http::request::<Vec<u8>>(&request, None)
            .map_err(|e| anyhow!("Failed to get engines: {}", e))?;

        // BUG: extism_pdk sometimes returns status 0 even for successful requests
        let is_success = (200..300).contains(&response.status())
            || (response.status() == 0 && !response.body().is_empty());

        if !is_success {
            return Err(anyhow!("Unable to get search engines"));
        }

        let config: serde_json::Value = serde_json::from_slice(&response.body())
            .map_err(|e| anyhow!("Failed to parse config: {}", e))?;
        if let Some(engines) = config.get("engines").and_then(|e| e.as_array()) {
            let mut result = HashMap::new();
            for engine in engines {
                if let Some(name) = engine.get("name").and_then(|n| n.as_str()) {
                    let include = match filter {
                        EngineFilter::All => true,
                        EngineFilter::Enabled => engine
                            .get("enabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                        EngineFilter::Disabled => !engine
                            .get("enabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true),
                    };

                    if include {
                        result.insert(name.to_string(), engine.clone());
                    }
                }
            }
            Ok(result)
        } else {
            Err(anyhow!("Unexpected response format"))
        }
    }
}
