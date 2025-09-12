use anyhow::{Result, anyhow};
use extism_pdk::{HttpRequest, config, http, info};
use html2md;
use url::Url;

pub fn browse(url: &str) -> Result<String> {
    let follow_redirects_str = config::get("BROWSE_FOLLOW_REDIRECTS")
        .ok()
        .flatten()
        .unwrap_or_else(|| "false".to_string());
    let follow_redirects = follow_redirects_str == "true";

    let max_redirects_str = config::get("BROWSE_MAX_REDIRECTS")
        .ok()
        .flatten()
        .unwrap_or_else(|| "10".to_string());
    let max_redirects: usize = max_redirects_str.parse().unwrap_or(10);

    let mut current_url = url.to_string();

    for _ in 0..max_redirects {
        info!("Browsing: {}", current_url);
        let request = HttpRequest::new(&current_url).with_method("GET");

        let response = http::request::<Vec<u8>>(&request, None)
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

        let status = response.status_code();

        if status >= 300 && status < 400 && follow_redirects {
            if let Some(location) = response.headers().get("location") {
                let location_str = location.clone();
                let new_url = if location_str.starts_with("http") {
                    location_str
                } else {
                    // relative URL, resolve against current_url
                    let base = Url::parse(&current_url)
                        .map_err(|e| anyhow!("Failed to parse current URL: {}", e))?;
                    base.join(&location_str)
                        .map_err(|e| anyhow!("Failed to resolve relative URL: {}", e))?
                        .to_string()
                };
                current_url = new_url;
                continue;
            }
        }

        // Not a redirect or not following redirects, process the response
        let is_success =
            (200..300).contains(&status) || (status == 0 && !response.body().is_empty());

        if !is_success {
            let body = String::from_utf8(response.body().to_vec())
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("HTTP Error: {} - {}", status, body));
        }

        let html = String::from_utf8(response.body().to_vec())
            .map_err(|e| anyhow!("Failed to decode response body: {}", e))?;

        return Ok(html2md::parse_html(&html));
    }

    Err(anyhow!("Too many redirects"))
}
