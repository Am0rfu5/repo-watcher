use clap::Parser;
use git2::{Repository, Error};

/// Monitors a GitHub repository for changes and pulls them
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// GitHub repository URL to monitor
    #[clap(short, long)]
    repo_url: String,

    /// Local repository path
    #[clap(short, long)]
    local_path: String,
}

fn pull_repo(repo_path: &str) -> Result<(), Error> {
    let repo = Repository::open(repo_path)?;
    let mut remote = repo.find_remote("origin")?;

    remote.fetch(&["master"], None, None)?;
    // Additional logic to merge fetched changes

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let repo_url = &cli.repo_url;
    let local_path = &cli.local_path;

    println!("Monitoring {} for changes", repo_url);
}

// TODO Logging
// TODO Monitor
// TODO Pull or Fetch
// TODO Merge
// TODO .env file command line option
// TODO Github API integration
// TODO Github webhook integration
// TODO Github webhook secret
// TODO Github webhook event type
// TODO Github webhook event action
// TODO Github webhook event repository
// TODO Github webhook event sender
// TODO Github webhook event organization
// TODO Github webhook event installation
