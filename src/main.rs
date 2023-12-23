use clap::Parser;
use git2::{Repository, MergeOptions, Error};
use anyhow::{Context, Result};

/// Monitors a GitHub repository for changes and pulls them
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {

    /// Local repository path
    #[clap(short, long)]
    local_path: String,
    
    /// GitHub repository URL to monitor
    #[clap(short, long)]
    remote: String,
    
    /// Branch to monitor
    #[clap(short, long)]
    branch: String,
    
}

fn pull_repo(repo_path: &str, remote_name: &str, branch: &str) -> Result<(), Error> {
    let repo = Repository::open(repo_path)?;
    let mut remote = repo.find_remote(remote_name)?;

    remote.fetch(&[branch], None, None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD").unwrap();
    let merge_commit = repo.reference_to_annotated_commit(&fetch_head).unwrap();

    let mut merge_options = MergeOptions::new();
    merge_options.fail_on_conflict(true);
    repo.merge(&[&merge_commit], Some(&mut merge_options), None).unwrap();
    Ok(())
}

fn fetch_latest_commit_sha(repo_path: &str, remote_name: &str) -> Result<String, Error> {
    let repo = Repository::open(repo_path)?;
    let mut remote = repo.find_remote(remote_name)?;

    remote.fetch(&["branch"], None, None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD").unwrap();
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head).unwrap();
    
    Ok(fetch_commit.id().to_string())
}

fn check_for_new_commits(repo_path: &str, latest_sha: &str) -> Result<bool, Error> {
    let repo = Repository::open(repo_path)?;
    let head = repo.head()?.peel_to_commit()?;
    let local_sha = head.id().to_string();

    Ok(local_sha != latest_sha)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let local_path = &cli.local_path;
    let remote = &cli.remote;
    let branch = &cli.branch;
    
    let latest_sha = fetch_latest_commit_sha(local_path, remote)
        .context("Failed to fetch the latest commit SHA")?;
    let has_new_commits = check_for_new_commits(&cli.local_path, &latest_sha)
        .context("Failed to check for new commits")?;

    if has_new_commits {
        pull_repo(&cli.local_path, remote, branch).context("Failed to pull new commits")?;        
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_latest_commit_sha() {
        let local_path = "test_repo";
        let remote_name = "origin";
        let expected_sha = "a4a7dce85cf63874e984719f4fdd239f5145052f";

        let actual_sha = fetch_latest_commit_sha(local_path, remote_name).unwrap();

        assert_eq!(expected_sha, actual_sha);
    }

    #[test]
    fn test_check_for_new_commits() {
        let local_path = "test_repo";
        let latest_sha = "a4a7dce85cf63874e984719f4fdd239f5145052f";

        let has_new_commits = check_for_new_commits(local_path, latest_sha).unwrap();

        assert_eq!(false, has_new_commits);
    }
}