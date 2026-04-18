mod client;
mod commands;
mod config;
mod models;

use anyhow::Result;
use clap::{Parser, ValueEnum};

use client::GitlabClient;
use config::Config;

#[derive(Parser)]
#[command(
    name = "gitlab_helper",
    about = "CLI tool to read and manage GitLab",
    override_usage = "gitlab_helper [-n <group>] [-r] <command> <object_type> [<name>]"
)]
struct Cli {
    /// GitLab personal access token
    #[arg(long, env = "GITLAB_TOKEN", hide_env_values = true)]
    token: Option<String>,

    /// GitLab instance URL (default: https://gitlab.com)
    #[arg(long, env = "GITLAB_URL")]
    url: Option<String>,

    /// Target group as the operation context; omit to operate at root
    #[arg(short = 'n', value_name = "group")]
    namespace: Option<String>,

    /// Execute the command recursively (descend into subgroups)
    #[arg(short = 'r')]
    recursive: bool,

    /// Clone using HTTP instead of SSH
    #[arg(long)]
    http: bool,

    /// Command to run
    #[arg(value_enum)]
    command: Command,

    /// Object type to operate on
    #[arg(value_enum, value_name = "object_type")]
    object_type: ObjectType,

    /// Optional trailing name addressing a specific object within the context
    #[arg(value_name = "name")]
    name: Option<String>,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum Command {
    /// Print objects of the given type
    List,
    /// Fetch metadata for the specified object
    Get,
    /// Clone repo(s); when applied to a group, clone every repo it contains
    Clone,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum ObjectType {
    Group,
    Repo,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cli {
        token,
        url,
        namespace,
        recursive,
        http,
        command,
        object_type,
        name,
    } = Cli::parse();

    let config = Config::from_args(token, url)?;
    let client = GitlabClient::new(&config)?;

    let namespace = namespace.as_deref();
    let name = name.as_deref();

    match command {
        Command::List => commands::list::run(&client, namespace, recursive, object_type, name).await?,
        Command::Get => commands::get::run(&client, namespace, recursive, object_type, name).await?,
        Command::Clone => {
            commands::clone::run(&client, namespace, recursive, http, object_type, name).await?
        }
    }

    Ok(())
}
