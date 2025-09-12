# hyper-mcp-search

A WebAssembly plugin for [hyper-mcp](https://github.com/tuananh/hyper-mcp) that provides privacy-focused web search capabilities using SearXNG.

## Features

- 🔍 **Privacy-focused search** using SearXNG metasearch engine
- 🕸️ **Web browsing** with HTML to Markdown conversion
- ⚙️ **Highly configurable** - engines, categories, safe search, result limits
- 🛡️ **Secure execution** - runs in WebAssembly sandbox
- 📊 **Structured results** - JSON output with titles, URLs, content, and scores

## Tools

- **search**: Perform web search using SearXNG
- **browse**: Fetch content from a URL as Markdown

## Installation

### Prerequisites

- [hyper-mcp](https://github.com/tuananh/hyper-mcp) server
- SearXNG instance (or use a public instance)

### Setup

1. **Build the plugin:**
   ```bash
   cargo build --release --target wasm32-wasip1
   ```

2. **Configure hyper-mcp:**

   Add to your hyper-mcp config (`~/.config/hyper-mcp/config.yaml`):

   ```yaml
   plugins:
     search:
       url: oci://ghcr.io/zatevakhin/hyper-mcp-search-plugin:v0.1.0
       runtime_config:
         allowed_hosts:
           - localhost
         env_vars:
           SEARXNG_BASE_URL: http://localhost:8080
           SEARXNG_DEFAULT_ENGINES: google,duckduckgo
           SEARXNG_NUM_RESULTS: "10"
   ```

## Usage

### With AI Agents

Configure hyper-mcp to work with your favorite AI agent or code editor:

#### Cursor IDE
Create `.cursor/mcp.json` in your project root:
```json
{
  "mcpServers": {
    "hyper-mcp": {
      "command": "/path/to/hyper-mcp",
      "args": ["--config", "/path/to/config.yaml"]
    }
  }
}
```

#### Claude Desktop
Add to your Claude Desktop MCP configuration:
```json
{
  "mcpServers": {
    "hyper-mcp": {
      "command": "/path/to/hyper-mcp",
      "args": ["--config", "/path/to/config.yaml"]
    }
  }
}
```

#### Other MCP-compatible tools
Most MCP-compatible tools support the `stdio`, `see` or `streamable-http` transport. Configure them to run the hyper-mcp binary with your config file.

## Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `SEARXNG_BASE_URL` | `http://localhost:8080` | SearXNG server URL |
| `SEARXNG_DEFAULT_ENGINES` | `""` | Comma-separated list of search engines |
| `SEARXNG_DEFAULT_CATEGORIES` | `""` | Comma-separated list of categories |
| `SEARXNG_LANGUAGE` | `"en"` | Search language |
| `SEARXNG_SAFE_SEARCH` | `"0"` | Safe search level (0=none, 1=moderate, 2=strict) |
| `SEARXNG_NUM_RESULTS` | `"5"` | Maximum number of results to return |
| `SEARXNG_USER_AGENT` | `"searxng-rs/{version}"` | HTTP user agent string |
| `BROWSE_FOLLOW_REDIRECTS` | `"false"` | Whether to follow HTTP redirects when browsing |
| `BROWSE_MAX_REDIRECTS` | `"10"` | Maximum number of redirects to follow when browsing |


## Development

### Prerequisites

- **Rust**: Follow the official installation instructions at [rustup.rs](https://rustup.rs/)
- **Nix (optional)**: If you have Nix, use `nix develop` for a pre-configured development environment

### Building

```bash
# Add WASM target
rustup target add wasm32-wasip1

# Build
cargo build --release --target wasm32-wasip1
```

### Configuration
   Add to your hyper-mcp config (`/path/to/config.yaml`):

   ```yaml
   plugins:
     search:
       url: file:///path/to/hyper-mcp-search/target/wasm32-wasip1/release/plugin.wasm"
       runtime_config:
         allowed_hosts:
           - localhost
         env_vars:
           SEARXNG_BASE_URL: http://localhost:8080
   ```

### Testing

```bash
# Run tests
cargo test

# Test with hyper-mcp
hyper-mcp --config path/to/config.yaml
```


## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built for the [hyper-mcp](https://github.com/tuananh/hyper-mcp) ecosystem
- Powered by [SearXNG](https://searxng.org/) for privacy-focused search
- Uses [Extism](https://extism.io/) for secure plugin execution
