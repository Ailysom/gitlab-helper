use std::collections::BTreeMap;

use anyhow::{bail, Result};

use crate::ObjectType;
use crate::client::GitlabClient;
use crate::models::{Group, Project};

pub async fn run(
    client: &GitlabClient,
    namespace: Option<&str>,
    recursive: bool,
    object_type: ObjectType,
    name: Option<&str>,
) -> Result<()> {
    match object_type {
        ObjectType::Group => {
            let path = match (namespace, name) {
                (Some(ns), Some(n)) => format!("{}/{}", ns, n),
                (Some(ns), None) => ns.to_string(),
                (None, Some(n)) => n.to_string(),
                (None, None) => bail!("`get group` requires -n <group> or a name"),
            };
            let group = client.get_group(&path).await?;
            print_group(&group);

            let (groups, projects) = if recursive {
                (
                    client.list_descendant_groups(&path).await?,
                    client.list_group_projects(&path, true).await?,
                )
            } else {
                (
                    client.list_subgroups(&path).await?,
                    client.list_group_projects(&path, false).await?,
                )
            };

            if !groups.is_empty() || !projects.is_empty() {
                println!();
                println!("{}/", group.path);
                let tree = build_tree(&path, &groups, &projects);
                print_node(&tree, "");
            }
        }
        ObjectType::Repo => {
            let name = name.ok_or_else(|| anyhow::anyhow!("`get repo` requires a name"))?;
            let path = match namespace {
                Some(ns) => format!("{}/{}", ns, name),
                None => name.to_string(),
            };
            let project = client.get_project(&path).await?;
            print_project(&project);
        }
    }
    Ok(())
}

fn print_group(g: &Group) {
    println!("id:          {}", g.id);
    println!("name:        {}", g.name);
    println!("full_path:   {}", g.full_path);
    if let Some(v) = &g.visibility {
        println!("visibility:  {}", v);
    }
    if let Some(d) = &g.description {
        if !d.is_empty() {
            println!("description: {}", d);
        }
    }
    if let Some(w) = &g.web_url {
        println!("web_url:     {}", w);
    }
    if let Some(p) = g.parent_id {
        println!("parent_id:   {}", p);
    }
}

fn print_project(p: &Project) {
    println!("id:                  {}", p.id);
    println!("name:                {}", p.name);
    println!("path_with_namespace: {}", p.path_with_namespace);
    if let Some(b) = &p.default_branch {
        println!("default_branch:      {}", b);
    }
    if let Some(v) = &p.visibility {
        println!("visibility:          {}", v);
    }
    if let Some(d) = &p.description {
        if !d.is_empty() {
            println!("description:         {}", d);
        }
    }
    if let Some(w) = &p.web_url {
        println!("web_url:             {}", w);
    }
    println!("ssh_url_to_repo:     {}", p.ssh_url_to_repo);
    println!("http_url_to_repo:    {}", p.http_url_to_repo);
}

#[derive(Default)]
struct Node {
    children: BTreeMap<String, Node>,
    repos: Vec<String>,
}

fn build_tree(root_path: &str, groups: &[Group], projects: &[Project]) -> Node {
    let mut root = Node::default();

    for g in groups {
        let segments = relative_segments(&g.full_path, root_path);
        if segments.is_empty() {
            continue;
        }
        let mut cur = &mut root;
        for seg in &segments {
            cur = cur.children.entry(seg.clone()).or_default();
        }
    }

    for p in projects {
        let segments = relative_segments(&p.path_with_namespace, root_path);
        if segments.is_empty() {
            continue;
        }
        let (leaf, parents) = segments.split_last().unwrap();
        let mut cur = &mut root;
        for seg in parents {
            cur = cur.children.entry(seg.clone()).or_default();
        }
        cur.repos.push(leaf.clone());
    }

    root
}

fn relative_segments(path: &str, prefix: &str) -> Vec<String> {
    if path == prefix {
        return Vec::new();
    }
    let marker = format!("{}/", prefix);
    let Some(rest) = path.strip_prefix(&marker) else {
        return Vec::new();
    };
    if rest.is_empty() {
        Vec::new()
    } else {
        rest.split('/').map(|s| s.to_string()).collect()
    }
}

enum Entry<'a> {
    Group(&'a str, &'a Node),
    Repo(&'a str),
}

fn print_node(node: &Node, prefix: &str) {
    let mut entries: Vec<Entry> = Vec::new();
    for (name, child) in &node.children {
        entries.push(Entry::Group(name, child));
    }
    let mut repos: Vec<&str> = node.repos.iter().map(String::as_str).collect();
    repos.sort();
    for repo in repos {
        entries.push(Entry::Repo(repo));
    }
    entries.sort_by(|a, b| entry_name(a).cmp(entry_name(b)));

    let total = entries.len();
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i + 1 == total;
        let connector = if is_last { "└── " } else { "├── " };
        match entry {
            Entry::Group(name, child) => {
                println!("{}{}{}/", prefix, connector, name);
                let child_prefix = if is_last {
                    format!("{}    ", prefix)
                } else {
                    format!("{}│   ", prefix)
                };
                print_node(child, &child_prefix);
            }
            Entry::Repo(name) => {
                println!("{}{}{}", prefix, connector, name);
            }
        }
    }
}

fn entry_name<'a>(e: &Entry<'a>) -> &'a str {
    match e {
        Entry::Group(name, _) => name,
        Entry::Repo(name) => name,
    }
}
