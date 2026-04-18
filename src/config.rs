use anyhow::{bail, Result};

pub struct Config {
    pub token: String,
    pub base_url: String,
}

impl Config {
    pub fn from_args(token: Option<String>, url: Option<String>) -> Result<Self> {
        let token = token.unwrap_or_default();
        if token.is_empty() {
            bail!("GitLab token is required. Use --token or set GITLAB_TOKEN env var");
        }

        let base_url = url.unwrap_or_else(|| "https://gitlab.com".to_string());

        Ok(Self { token, base_url })
    }
}
