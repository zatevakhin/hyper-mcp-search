use anyhow::{Result, anyhow};
use extism_pdk::{HttpRequest, config, http, info};
use html2md;
use regex::Regex;
use url::Url;

/// Strip <style> and <script> elements from HTML
fn strip_styles_and_scripts(html: &str) -> String {
    // Regex to match <style>...</style> and <script>...</script> tags (case insensitive, with attributes, dot matches newlines)
    let style_re = Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();
    let script_re = Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();

    // Remove style and script tags
    let without_styles = style_re.replace_all(html, "");
    let cleaned_html = script_re.replace_all(&without_styles, "");

    cleaned_html.to_string()
}

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

        // Strip <style> and <script> tags from HTML before converting to markdown
        let cleaned_html = strip_styles_and_scripts(&html);

        return Ok(html2md::parse_html(&cleaned_html));
    }

    Err(anyhow!("Too many redirects"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_markdown_strips_styles() {
        let html_with_style = r#"
        <html>
        <head>
        <style>
        body { background-color: red; }
        .class { color: blue; }
        </style>
        </head>
        <body>
        <h1>Title</h1>
        <p>Content</p>
        </body>
        </html>
        "#;

        let cleaned = strip_styles_and_scripts(html_with_style);
        let markdown = html2md::parse_html(&cleaned);
        assert!(!markdown.contains("background-color"));
        assert!(!markdown.contains("color: blue"));
        assert!(markdown.contains("Title"));
        assert!(markdown.contains("Content"));
    }

    #[test]
    fn test_html_to_markdown_strips_scripts() {
        let html_with_script = r#"
        <html>
        <head>
        <script>
        function myFunction() {
            alert('Hello');
        }
        </script>
        </head>
        <body>
        <h1>Title</h1>
        <p>Content</p>
        </body>
        </html>
        "#;

        let cleaned = strip_styles_and_scripts(html_with_script);
        let markdown = html2md::parse_html(&cleaned);
        assert!(!markdown.contains("function myFunction"));
        assert!(!markdown.contains("alert('Hello')"));
        assert!(markdown.contains("Title"));
        assert!(markdown.contains("Content"));
    }

    #[test]
    fn test_html_to_markdown_strips_inline_styles_and_scripts() {
        let html_mixed = r#"
        <html>
        <body>
        <h1 style="color: red;">Title</h1>
        <p>Content</p>
        <script>console.log('test');</script>
        <style>p { font-size: 14px; }</style>
        </body>
        </html>
        "#;

        let cleaned = strip_styles_and_scripts(html_mixed);
        let markdown = html2md::parse_html(&cleaned);
        assert!(!markdown.contains("color: red"));
        assert!(!markdown.contains("console.log"));
        assert!(!markdown.contains("font-size"));
        assert!(markdown.contains("Title"));
        assert!(markdown.contains("Content"));
    }
}
