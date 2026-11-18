//! GitHub API client for fetching repository information
//!
//! Provides functions to fetch tags and download files from GitHub repositories

use crate::{dure_debug, dure_info};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// GitHub tag information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubTag {
    pub name: String,
    #[serde(default)]
    pub tarball_url: String,
}

/// Fetch latest tags from GitHub repository
///
/// # Arguments
/// * `repo` - Repository in format "owner/repo"
/// * `count` - Number of tags to fetch
///
/// # Returns
/// Vector of GitHubTag sorted by version (newest first)
pub async fn fetch_latest_tags(repo: &str, count: usize) -> Result<Vec<GitHubTag>> {
    let url = format!(
        "https://api.github.com/repos/{}/tags?per_page={}",
        repo, count
    );

    dure_debug!("Fetching GitHub tags from: {}", url);

    // Use ehttp with cache
    let response = fetch_with_cache(&url, 3600).await?;

    let tags: Vec<GitHubTag> =
        serde_json::from_slice(&response.bytes).context("Failed to parse GitHub tags JSON")?;

    dure_info!("Fetched {} tags from {}", tags.len(), repo);
    Ok(tags)
}

/// Download .env.example file from dure-mycart repository
///
/// # Arguments
/// * `version` - Version identifier: "v0.0.15", "main", "test", etc.
///
/// # Returns
/// String content of .env.example file
pub async fn download_env_example(version: &str) -> Result<String> {
    let url = env_example_url(version);
    dure_debug!("Downloading .env.example from: {}", url);

    let response = fetch_with_cache(&url, 3600).await?;
    let content =
        String::from_utf8(response.bytes).context("Failed to decode .env.example as UTF-8")?;

    dure_info!("Downloaded .env.example ({} bytes)", content.len());
    Ok(content)
}

/// Generate URL for .env.example file based on version
fn env_example_url(version: &str) -> String {
    let base = "https://raw.githubusercontent.com/dure-one/dure-mycart";

    // Determine ref type (branch or tag)
    let ref_path = if version == "main" || version == "test" {
        format!("refs/heads/{}", version)
    } else {
        format!("refs/tags/{}", version)
    };

    format!("{}/{}/prosody-mycart-stack/.env.example", base, ref_path)
}

/// Fetch URL with caching using ehttp
async fn fetch_with_cache(url: &str, ttl_seconds: u64) -> Result<ehttp::Response> {
    use std::sync::Arc;

    let (tx, rx) = smol::channel::bounded(1);
    let url_owned = url.to_string();

    // Spawn ehttp request on main thread (required by ehttp)
    std::thread::spawn(move || {
        let cache = crate::api::ehttp_cache::EhttpCache::new(None, ttl_seconds);

        // Check cache first
        if let Some(response) = cache.get(&url_owned) {
            let _ = tx.try_send(Ok(response));
            return;
        }

        // Fetch from network
        let request = ehttp::Request::get(&url_owned);
        ehttp::fetch(request, move |result| {
            let response = match result {
                Ok(resp) => {
                    if resp.ok {
                        cache.put(&resp, Some(ttl_seconds));
                        Ok(resp)
                    } else {
                        Err(anyhow::anyhow!("HTTP {} {}", resp.status, resp.status_text))
                    }
                }
                Err(e) => Err(anyhow::anyhow!("Network error: {}", e)),
            };

            let _ = tx.try_send(response);
        });
    });

    // Wait for response
    rx.recv().await.context("Failed to receive HTTP response")?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_example_url_generation() {
        // Test tag version
        let url = env_example_url("v0.0.15");
        assert_eq!(
            url,
            "https://raw.githubusercontent.com/dure-one/dure-mycart/refs/tags/v0.0.15/prosody-mycart-stack/.env.example"
        );

        // Test main branch
        let url = env_example_url("main");
        assert_eq!(
            url,
            "https://raw.githubusercontent.com/dure-one/dure-mycart/refs/heads/main/prosody-mycart-stack/.env.example"
        );

        // Test test branch
        let url = env_example_url("test");
        assert_eq!(
            url,
            "https://raw.githubusercontent.com/dure-one/dure-mycart/refs/heads/test/prosody-mycart-stack/.env.example"
        );
    }

    #[test]
    fn test_github_tags_parsing() {
        let json = r#"[
            {"name":"v0.0.15","tarball_url":"https://example.com/v0.0.15.tar.gz"},
            {"name":"v0.0.14","tarball_url":"https://example.com/v0.0.14.tar.gz"}
        ]"#;

        let tags: Vec<GitHubTag> = serde_json::from_str(json).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "v0.0.15");
        assert_eq!(tags[1].name, "v0.0.14");
    }

    #[test]
    fn test_github_tag_struct() {
        let tag = GitHubTag {
            name: "v1.0.0".to_string(),
            tarball_url: "https://example.com/v1.0.0.tar.gz".to_string(),
        };

        assert_eq!(tag.name, "v1.0.0");
        assert_eq!(tag.tarball_url, "https://example.com/v1.0.0.tar.gz");
    }
}
