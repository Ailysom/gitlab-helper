use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Group {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub full_path: String,
    #[serde(default)]
    pub parent_id: Option<u64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub web_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub path_with_namespace: String,
    #[serde(default)]
    pub default_branch: Option<String>,
    pub ssh_url_to_repo: String,
    pub http_url_to_repo: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub web_url: Option<String>,
    #[serde(default)]
    pub namespace: Option<Namespace>,
}

#[derive(Debug, Deserialize)]
pub struct Namespace {
    pub kind: String,
}
