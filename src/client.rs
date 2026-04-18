use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;

use crate::config::Config;
use crate::models::{Group, Project};

pub struct GitlabClient {
    http: reqwest::Client,
    base_url: String,
}

impl GitlabClient {
    pub fn new(config: &Config) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("PRIVATE-TOKEN", HeaderValue::from_str(&config.token)?);

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            http,
            base_url: config.base_url.trim_end_matches('/').to_string(),
        })
    }

    async fn paginate<T>(&self, path: &str, extra_query: &[(&str, &str)]) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let mut results = Vec::new();
        let mut page: u32 = 1;

        loop {
            let url = format!("{}{}", self.base_url, path);
            let page_str = page.to_string();
            let mut query: Vec<(&str, &str)> = vec![("per_page", "100"), ("page", &page_str)];
            query.extend_from_slice(extra_query);

            let resp = self
                .http
                .get(&url)
                .query(&query)
                .send()
                .await?
                .error_for_status()?;

            let next_page: Option<u32> = resp
                .headers()
                .get("x-next-page")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());

            let batch: Vec<T> = resp.json().await?;
            if batch.is_empty() {
                break;
            }

            results.extend(batch);

            match next_page {
                Some(p) if p > page => page = p,
                _ => break,
            }
        }

        Ok(results)
    }

    pub async fn list_top_level_groups(&self) -> Result<Vec<Group>> {
        let mut groups: Vec<Group> = self
            .paginate(
                "/api/v4/groups",
                &[("top_level_only", "true"), ("all_available", "true")],
            )
            .await?;
        groups.sort_by(|a, b| a.full_path.cmp(&b.full_path));
        Ok(groups)
    }

    pub async fn list_all_groups(&self) -> Result<Vec<Group>> {
        let mut groups: Vec<Group> = self
            .paginate("/api/v4/groups", &[("all_available", "true")])
            .await?;
        groups.sort_by(|a, b| a.full_path.cmp(&b.full_path));
        Ok(groups)
    }

    pub async fn list_subgroups(&self, group_path: &str) -> Result<Vec<Group>> {
        let encoded = urlencoding::encode(group_path);
        let path = format!("/api/v4/groups/{}/subgroups", encoded);
        let mut groups: Vec<Group> = self.paginate(&path, &[]).await?;
        groups.sort_by(|a, b| a.full_path.cmp(&b.full_path));
        Ok(groups)
    }

    pub async fn list_descendant_groups(&self, group_path: &str) -> Result<Vec<Group>> {
        let encoded = urlencoding::encode(group_path);
        let path = format!("/api/v4/groups/{}/descendant_groups", encoded);
        let mut groups: Vec<Group> = self.paginate(&path, &[]).await?;
        groups.sort_by(|a, b| a.full_path.cmp(&b.full_path));
        Ok(groups)
    }

    pub async fn list_group_projects(
        &self,
        group_path: &str,
        include_subgroups: bool,
    ) -> Result<Vec<Project>> {
        let encoded = urlencoding::encode(group_path);
        let path = format!("/api/v4/groups/{}/projects", encoded);
        let include_sub = if include_subgroups { "true" } else { "false" };
        let mut projects: Vec<Project> = self
            .paginate(&path, &[("include_subgroups", include_sub)])
            .await?;
        projects.sort_by(|a, b| a.path_with_namespace.cmp(&b.path_with_namespace));
        Ok(projects)
    }

    pub async fn list_all_projects(&self) -> Result<Vec<Project>> {
        let mut projects: Vec<Project> = self
            .paginate("/api/v4/projects", &[("membership", "true")])
            .await?;
        projects.sort_by(|a, b| a.path_with_namespace.cmp(&b.path_with_namespace));
        Ok(projects)
    }

    pub async fn list_top_level_projects(&self) -> Result<Vec<Project>> {
        let mut projects: Vec<Project> = self
            .paginate("/api/v4/projects", &[("membership", "true")])
            .await?;
        projects.retain(|p| matches!(&p.namespace, Some(ns) if ns.kind == "user"));
        projects.sort_by(|a, b| a.path_with_namespace.cmp(&b.path_with_namespace));
        Ok(projects)
    }

    pub async fn get_group(&self, group_path: &str) -> Result<Group> {
        let encoded = urlencoding::encode(group_path);
        let url = format!("{}/api/v4/groups/{}", self.base_url, encoded);
        let group: Group = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(group)
    }

    pub async fn get_project(&self, project_path: &str) -> Result<Project> {
        let encoded = urlencoding::encode(project_path);
        let url = format!("{}/api/v4/projects/{}", self.base_url, encoded);
        let project: Project = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(project)
    }
}
