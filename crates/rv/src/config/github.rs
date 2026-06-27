//! GitHub API utilities for rv.
//!
//! This module provides shared functionality for interacting with GitHub's API,
//! including authentication token retrieval.

/// The recommended GitHub API version header value.
/// See: <https://docs.github.com/en/rest/overview/api-versions>
pub const GITHUB_API_VERSION: &str = "2022-11-28";

/// Retrieves a GitHub authentication token from environment variables.
///
/// Checks `GITHUB_TOKEN` first (automatically available in GitHub Actions),
/// then falls back to `GH_TOKEN` (used by GitHub CLI and for general use).
///
/// Returns `None` if neither environment variable is set.
pub fn github_token() -> Option<String> {
    std::env::var("GITHUB_TOKEN")
        .ok()
        .or_else(|| std::env::var("GH_TOKEN").ok())
}

/// Builds a `reqwest::RequestBuilder` for a GitHub API endpoint with standard headers
/// and optional authentication.
pub fn github_api_get(
    client: &reqwest::Client,
    url: impl reqwest::IntoUrl + AsRef<str>,
) -> Result<reqwest::RequestBuilder, url::ParseError> {
    use tracing::debug;

    let url = url::Url::parse(url.as_ref())?;
    let should_authenticate = is_allowed_github_auth_url(&url);
    let mut builder = client
        .get(url)
        .header("User-Agent", "rv-cli")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", GITHUB_API_VERSION);

    if should_authenticate {
        if let Some(token) = github_token() {
            debug!("Using authenticated GitHub API request");
            builder = builder.header("Authorization", format!("Bearer {}", token));
        } else {
            debug!("No GitHub token found, using unauthenticated API request");
        }
    }

    Ok(builder)
}

/// Returns true when a bearer token may be sent to this GitHub URL.
///
/// Tokens are restricted to HTTPS requests for the GitHub hosts that `rv` uses for
/// release metadata and archive downloads.
pub fn is_allowed_github_auth_url(url: &url::Url) -> bool {
    if url.scheme() != "https" {
        return false;
    }

    let Some(host) = url.host_str() else {
        return false;
    };

    host.eq_ignore_ascii_case("github.com") || host.eq_ignore_ascii_case("api.github.com")
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::AUTHORIZATION;
    use std::{
        ffi::OsString,
        sync::{Mutex, MutexGuard},
    };

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct GithubTokenEnv {
        _guard: MutexGuard<'static, ()>,
        github_token: Option<OsString>,
        gh_token: Option<OsString>,
    }

    impl GithubTokenEnv {
        fn new() -> Self {
            let guard = ENV_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            Self {
                _guard: guard,
                github_token: std::env::var_os("GITHUB_TOKEN"),
                gh_token: std::env::var_os("GH_TOKEN"),
            }
        }

        fn set_github_token(&self, value: &str) {
            unsafe {
                std::env::set_var("GITHUB_TOKEN", value);
            }
        }

        fn set_gh_token(&self, value: &str) {
            unsafe {
                std::env::set_var("GH_TOKEN", value);
            }
        }

        fn remove_github_token(&self) {
            unsafe {
                std::env::remove_var("GITHUB_TOKEN");
            }
        }

        fn remove_gh_token(&self) {
            unsafe {
                std::env::remove_var("GH_TOKEN");
            }
        }
    }

    impl Drop for GithubTokenEnv {
        fn drop(&mut self) {
            unsafe {
                match &self.github_token {
                    Some(value) => std::env::set_var("GITHUB_TOKEN", value),
                    None => std::env::remove_var("GITHUB_TOKEN"),
                }
                match &self.gh_token {
                    Some(value) => std::env::set_var("GH_TOKEN", value),
                    None => std::env::remove_var("GH_TOKEN"),
                }
            }
        }
    }

    fn request_has_authorization(url: &str) -> bool {
        let client = reqwest::Client::new();
        let request = github_api_get(&client, url)
            .unwrap_or_else(|error| panic!("failed to parse request URL {url}: {error}"))
            .build()
            .unwrap_or_else(|error| panic!("failed to build request for {url}: {error}"));
        request.headers().contains_key(AUTHORIZATION)
    }

    fn parsed_url(url: &str) -> url::Url {
        url::Url::parse(url)
            .unwrap_or_else(|error| panic!("failed to parse test URL {url}: {error}"))
    }

    #[test]
    fn test_github_api_version_is_valid() {
        // Ensure the API version follows the expected format (YYYY-MM-DD)
        assert_eq!(GITHUB_API_VERSION.len(), 10);
        assert!(GITHUB_API_VERSION.chars().nth(4) == Some('-'));
        assert!(GITHUB_API_VERSION.chars().nth(7) == Some('-'));
    }

    #[test]
    fn test_github_token_prefers_github_token_over_gh_token() {
        let env = GithubTokenEnv::new();
        env.remove_github_token();
        env.set_gh_token("gh_token_value");
        assert_eq!(github_token(), Some("gh_token_value".to_string()));
        env.set_github_token("github_token_value");
        assert_eq!(github_token(), Some("github_token_value".to_string()));
    }

    #[test]
    fn test_github_token_falls_back_to_gh_token() {
        let env = GithubTokenEnv::new();
        env.remove_github_token();
        env.remove_gh_token();
        assert_eq!(github_token(), None);
        env.set_gh_token("gh_token_value");
        assert_eq!(github_token(), Some("gh_token_value".to_string()));
    }

    #[test]
    fn test_github_token_returns_none_when_neither_set() {
        let env = GithubTokenEnv::new();
        env.remove_github_token();
        env.remove_gh_token();
        assert_eq!(github_token(), None);
    }

    #[test]
    fn test_github_token_uses_github_token_when_gh_token_not_set() {
        let env = GithubTokenEnv::new();
        env.set_github_token("only_github_token");
        env.remove_gh_token();
        assert_eq!(github_token(), Some("only_github_token".to_string()));
    }

    #[test]
    fn test_github_api_get_only_authenticates_allowed_urls() {
        let env = GithubTokenEnv::new();
        env.set_github_token("github_token_value");
        env.remove_gh_token();

        assert!(request_has_authorization(
            "https://api.github.com/repos/owner/repo"
        ));
        assert!(!request_has_authorization(
            "http://api.github.com/repos/owner/repo"
        ));
        assert!(!request_has_authorization("http://github.com/owner/repo"));
        assert!(!request_has_authorization(
            "https://raw.github.com/owner/repo/main/file"
        ));
    }

    #[test]
    fn test_is_allowed_github_auth_url_for_expected_hosts() {
        assert!(is_allowed_github_auth_url(&parsed_url(
            "https://github.com/owner/repo"
        )));
        assert!(is_allowed_github_auth_url(&parsed_url(
            "https://github.com/spinel-coop/rv-ruby/releases/latest/download/ruby-3.3.0.tar.gz"
        )));
        assert!(is_allowed_github_auth_url(&parsed_url(
            "https://api.github.com/repos/owner/repo"
        )));
    }

    #[test]
    fn test_is_allowed_github_auth_url_rejects_insecure_schemes() {
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "http://github.com/owner/repo"
        )));
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "http://api.github.com/repos/owner/repo"
        )));
    }

    #[test]
    fn test_is_allowed_github_auth_url_rejects_unapproved_subdomains() {
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "https://raw.github.com/owner/repo/main/file"
        )));
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "https://objects.github.com/something"
        )));
    }

    #[test]
    fn test_is_allowed_github_auth_url_is_case_insensitive() {
        assert!(is_allowed_github_auth_url(&parsed_url(
            "https://GITHUB.COM/owner/repo"
        )));
        assert!(is_allowed_github_auth_url(&parsed_url(
            "https://GitHub.com/owner/repo"
        )));
        assert!(is_allowed_github_auth_url(&parsed_url(
            "https://API.GITHUB.COM/repos"
        )));
    }

    #[test]
    fn test_is_allowed_github_auth_url_rejects_fake_domains() {
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "https://fakegithub.com/owner/repo"
        )));
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "https://notgithub.com/owner/repo"
        )));
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "https://github.com.evil.com/owner/repo"
        )));
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "https://mygithub.company.com/owner/repo"
        )));
    }

    #[test]
    fn test_is_allowed_github_auth_url_rejects_github_in_path() {
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "https://example.com/github.com/owner/repo"
        )));
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "https://mirror.example.com/proxy/github.com/file"
        )));
    }

    #[test]
    fn test_is_allowed_github_auth_url_rejects_non_github() {
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "https://gitlab.com/owner/repo"
        )));
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "https://bitbucket.org/owner/repo"
        )));
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "https://example.com/file.tar.gz"
        )));
    }

    #[test]
    fn test_is_allowed_github_auth_url_rejects_missing_hosts() {
        assert!(!is_allowed_github_auth_url(&parsed_url(
            "file:///tmp/archive"
        )));
    }

    #[test]
    fn test_github_api_get_rejects_invalid_urls() {
        let client = reqwest::Client::new();
        assert!(github_api_get(&client, "not a url").is_err());
        assert!(github_api_get(&client, "").is_err());
        assert!(github_api_get(&client, "github.com/owner/repo").is_err());
    }
}
