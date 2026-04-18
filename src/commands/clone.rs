use std::path::Path;
use std::process::Command;

use anyhow::{bail, Result};

use crate::ObjectType;
use crate::client::GitlabClient;
use crate::models::Project;

pub async fn run(
    client: &GitlabClient,
    namespace: Option<&str>,
    recursive: bool,
    use_http: bool,
    object_type: ObjectType,
    name: Option<&str>,
) -> Result<()> {
    let (context, projects) =
        collect_projects(client, namespace, recursive, object_type, name).await?;

    if projects.is_empty() {
        println!("No repositories to clone.");
        return Ok(());
    }

    println!("Cloning {} repositories...\n", projects.len());

    let mut failed = Vec::new();
    for p in &projects {
        let url = if use_http {
            &p.http_url_to_repo
        } else {
            &p.ssh_url_to_repo
        };
        let target = target_path(&context, p);

        println!("Cloning {} -> ./{}", p.path_with_namespace, target);

        if let Some(parent) = Path::new(&target).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let status = Command::new("git").args(["clone", url, &target]).status();

        match status {
            Ok(s) if s.success() => println!("  OK\n"),
            Ok(s) => {
                eprintln!("  FAILED (exit code: {})\n", s.code().unwrap_or(-1));
                failed.push(p.path_with_namespace.as_str());
            }
            Err(e) => {
                eprintln!("  FAILED ({})\n", e);
                failed.push(p.path_with_namespace.as_str());
            }
        }
    }

    println!(
        "Done. {}/{} cloned successfully.",
        projects.len() - failed.len(),
        projects.len()
    );

    if !failed.is_empty() {
        eprintln!("\nFailed to clone:");
        for n in &failed {
            eprintln!("  - {}", n);
        }
    }

    Ok(())
}

fn target_path(context: &str, p: &Project) -> String {
    let context = context.trim_matches('/');
    if context.is_empty() {
        return p.path_with_namespace.clone();
    }
    if let Some(rest) = p.path_with_namespace.strip_prefix(context) {
        let rest = rest.trim_start_matches('/');
        if rest.is_empty() {
            return p.path.clone();
        }
        return rest.to_string();
    }
    p.path_with_namespace.clone()
}

async fn collect_projects(
    client: &GitlabClient,
    namespace: Option<&str>,
    recursive: bool,
    object_type: ObjectType,
    name: Option<&str>,
) -> Result<(String, Vec<Project>)> {
    match object_type {
        ObjectType::Repo => {
            let name = name.ok_or_else(|| anyhow::anyhow!("`clone repo` requires a name"))?;
            let path = match namespace {
                Some(ns) => format!("{}/{}", ns, name),
                None => name.to_string(),
            };
            let p = client.get_project(&path).await?;
            Ok((path, vec![p]))
        }
        ObjectType::Group => {
            let group_path = match (namespace, name) {
                (Some(ns), Some(n)) => format!("{}/{}", ns, n),
                (Some(ns), None) => ns.to_string(),
                (None, Some(n)) => n.to_string(),
                (None, None) => bail!("`clone group` requires -n <group> or a name"),
            };
            let projects = client.list_group_projects(&group_path, recursive).await?;
            Ok((group_path, projects))
        }
    }
}
