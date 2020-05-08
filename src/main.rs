use structopt::StructOpt;
use std::path::PathBuf;
use git2::Repository;
use anyhow::{Context, Result};

mod git;

#[derive(Debug, StructOpt)]
#[structopt(name = "Release Diff", about = "Generate a release summary")]
pub struct CliOptions {
    /// Input repo
    #[structopt(parse(from_os_str),
    default_value = "./",
    short = "r",
    long = "repo"
    )]
    repo: PathBuf,

    #[structopt(name = "BASE BRANCH")]
    base_branch: String,

    #[structopt(name = "RELEASE BRANCH")]
    release_branch: String,

}


fn main() -> Result<()> {
    let opt = CliOptions::from_args();

    let repo = Repository::open(&opt.repo).with_context(|| {
        format!("failed to open repository: {:?}", opt.repo)
    })?;

    let commits = git::get_commits(&repo, &opt).with_context(
        || "Failed to find commit difference between branches",
    )?;

    println!("Done {:?}", commits);

    Ok(())
}
