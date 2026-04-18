use anyhow::{bail, Result};

use crate::ObjectType;
use crate::client::GitlabClient;

pub async fn run(
    client: &GitlabClient,
    namespace: Option<&str>,
    recursive: bool,
    object_type: ObjectType,
    name: Option<&str>,
) -> Result<()> {
    if name.is_some() {
        bail!("`list` does not accept a trailing name; use -n to set the context");
    }

    match object_type {
        ObjectType::Group => list_groups(client, namespace, recursive).await,
        ObjectType::Repo => list_repos(client, namespace, recursive).await,
    }
}

async fn list_groups(
    client: &GitlabClient,
    namespace: Option<&str>,
    recursive: bool,
) -> Result<()> {
    let groups = match (namespace, recursive) {
        (None, false) => client.list_top_level_groups().await?,
        (None, true) => client.list_all_groups().await?,
        (Some(ns), false) => client.list_subgroups(ns).await?,
        (Some(ns), true) => client.list_descendant_groups(ns).await?,
    };

    for g in &groups {
        if recursive {
            println!("{}", g.full_path);
        } else {
            println!("{}", g.path);
        }
    }
    Ok(())
}

async fn list_repos(
    client: &GitlabClient,
    namespace: Option<&str>,
    recursive: bool,
) -> Result<()> {
    let projects = match namespace {
        None if recursive => client.list_all_projects().await?,
        None => client.list_top_level_projects().await?,
        Some(ns) => client.list_group_projects(ns, recursive).await?,
    };

    for p in &projects {
        if recursive {
            println!("{}", p.path_with_namespace);
        } else {
            println!("{}", p.path);
        }
    }
    Ok(())
}
