use clap::Parser;

use git2::{Repository, RemoteCallbacks, Cred, MergeOptions, FetchOptions, Error};
use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use std::process;
use dotenv::dotenv;
use std::env;

/// Monitors a GitHub repository for changes and pulls them
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Local repository path
    #[clap(short, long, value_parser)]
    local_path: Option<PathBuf>,
    
    /// GitHub repository URL to monitor
    #[clap(short, long)]
    remote: Option<String>,
    
    /// Branch to monitor
    #[clap(short, long)]
    branch: Option<String>,

    /// Path to the SSH key for authentication
    #[clap(short, long, value_parser)]
    ssh_key_path: Option<PathBuf>,

    /// Path to the .env file
    #[clap(short, long, value_parser)]
    env_file: Option<PathBuf>,
        
}

// fn validate_args(args: &Cli) -> Result<()> {
//     // Check if local_path exists and is a directory
//     if !args.local_path.exists() {
//         eprintln!("Error: Path does not exist");
//         process::exit(1);
//     }
//     if !args.local_path.is_dir() {
//         eprintln!("Error: Path is not a directory");
//         process::exit(1);
//     }

//     // Validate local_path is a Git repo
//     let repo = Repository::open(&args.local_path)
//         .map_err(|e| {
//             anyhow!("Error: Path is not a valid Git repository: {}", e)
//         })?;

//     // Validate branch is a valid branch for the local_path repo
//     repo.find_branch(&args.branch, git2::BranchType::Local)
//         .map_err(|_| {
//             anyhow!("Error: Branch '{}' is not a valid branch for the given Git repository", args.branch)
//         })?;
        
//     // Validate remote is a valid remote for the given Git repo
//     repo.find_remote(&args.remote)
//         .map_err(|_| {
//             anyhow!("Error: Remote '{}' is not a valid remote for the given Git repository", args.remote)
//         })?;

//     Ok(())
// }

fn fetch_latest_commit_sha(local_path: &Path, ssh_key_path: &Path, remote: &str, branch: &str) -> Result<String, Error> {
    let repo = Repository::open(local_path)?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            ssh_key_path,
            None,
        )
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    repo.find_remote(remote)?
        .fetch(&[branch], Some(&mut fetch_options), None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

    Ok(fetch_commit.id().to_string())
}

fn check_for_new_commits(repo_path: &Path, latest_sha: &str) -> Result<bool, Error> {
    let repo = Repository::open(repo_path)?;
    let head = repo.head()?.peel_to_commit()?;
    let local_sha = head.id().to_string();

    Ok(local_sha != latest_sha)
}

fn pull_repo(local_path: &Path, remote: &str, branch: &str) -> Result<(), Error> {
    let repo = Repository::open(local_path)?;
    let mut remote = repo.find_remote(remote)?;

    remote.fetch(&[branch], None, None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD").unwrap();
    let merge_commit = repo.reference_to_annotated_commit(&fetch_head).unwrap();

    let mut merge_options = MergeOptions::new();
    merge_options.fail_on_conflict(true);
    repo.merge(&[&merge_commit], Some(&mut merge_options), None).unwrap();
    Ok(())
}


fn main() -> Result<()> {
    let args = Cli::parse();
    // validate_args(&args)?;
        
    run(&args)?;
    Ok(())
}

fn run(args: &Cli) -> Result<(), Error> {
    // Load configurations from .env file if provided
    if let Some(env_path) = args.env_file {
        dotenv::from_path(env_path).ok();
    }

    // Override with command-line arguments or use .env values
    let local_path = args.local_path.unwrap_or_else(|| PathBuf::from(env::var("LOCAL_PATH").expect("Local path not set")));
    let remote = args.remote.unwrap_or_else(|| env::var("REMOTE").expect("Remote not set"));
    let branch = args.branch.unwrap_or_else(|| env::var("BRANCH").expect("Branch not set"));
    let ssh_key_path = args.ssh_key_path.unwrap_or_else(|| PathBuf::from(env::var("SSH_KEY_PATH").expect("SSH key path not set")));
    
    // let path_buf = PathBuf::from(local_path);
    let path = local_path.as_path();
    let ssh_key_path = ssh_key_path.as_path();
    
    let latest_sha = fetch_latest_commit_sha(&path, &ssh_key_path, &remote, &branch)
        .context("Failed to fetch the latest commit SHA")?;    
    
    let has_new_commits = check_for_new_commits(&path, &latest_sha)
        .context("Failed to check for new commits")?;

    if has_new_commits {
        pull_repo(&path, &remote, &branch).context("Failed to pull new commits")?;        
    }
    
   
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn _test_values() -> Cli{
        let env_path = Path::new(".local.env");
        dotenv::from_path(env_path).ok();
        
        let local_path = PathBuf::from(env::var("LOCAL_PATH").expect("Local path not set"));
        let remote = env::var("REMOTE").expect("Remote not set");
        let branch = env::var("BRANCH").expect("Branch not set");
        let ssh_key_path = PathBuf::from(env::var("SSH_KEY_PATH").expect("SSH key path not set"));
        
        // let path_buf = PathBuf::from(local_path);
        let path = local_path.as_path();
        let ssh_key_path = ssh_key_path.as_path();
        
        
        Cli {
            local_path: Some(PathBuf::from(env::var("LOCAL_PATH").expect("Local path not set"))),
            remote: Some("github".to_string()),
            branch: Some("master".to_string()),
            ssh_key_path: Some(PathBuf::from("test_key")),
            env_file: None,
        }
    
    }
    
    // #[test]
    // fn test_validate_args_valid() {
    //     // Test with valid arguments
    //     let args = _test_values();
    //     assert!(validate_args(&args).is_ok());

    // }
    
    #[test]
    fn test_pull_repo() {
        let args = _test_values();

        let result = pull_repo(&args);

        assert!(result.is_ok());
    }

    #[test]
    fn test_fetch_latest_commit_sha() {
        let args = _test_values();
        let expected_sha = "449022de3b3ebcfbbbb010f2ca91f724df03b33e";


        let actual_sha = match fetch_latest_commit_sha(&args) {
            Ok(sha) => sha,
            Err(e) => {
                println!("Error occurred: {}", e); // Print error message
                panic!("Test failed due to error: {}", e); // Panic with error message
            },
        };
        assert_eq!(expected_sha, actual_sha);
    }

    #[test]
    fn test_check_for_new_commits() {
        let local_path = "test_repo.git";
        let path = Path::new(local_path);
        let latest_sha = "449022de3b3ebcfbbbb010f2ca91f724df03b33e";
        // let branch = "master";
        
        let has_new_commits = check_for_new_commits(path, latest_sha).unwrap();

        assert_eq!(false, has_new_commits);
    }
}