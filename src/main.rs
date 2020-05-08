use structopt::StructOpt;
use std::path::PathBuf;
use git2::{Repository, Oid, Commit};
use anyhow::{Context, Result, anyhow};
use lazy_static::lazy_static;
use regex::CaptureMatches;

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

#[derive(Debug, Clone)]
pub enum ReleaseIdentifier
{
    RRQ(u32),
    TargetProgress(u32)
}

#[derive(Debug, Clone)]
pub struct ReleaseItem
{
    pub sha1: Oid,
    pub commit_summary: String,
    pub ids: Vec<ReleaseIdentifier>
}

impl ReleaseItem {
    pub fn new(commit: Commit<'_>) -> Result<Self> {

        let mut r = Self {
            sha1: commit.id(),
            commit_summary: commit.message()
                .ok_or(anyhow!("Failed to get commit message for {}", commit.id()))?
                .to_owned(),
            ids: Vec::new()
        };


        use regex::Regex;

        fn extract_matches<'a>(captures: CaptureMatches<'a, 'a>, ctor: impl Fn(u32) -> ReleaseIdentifier + 'a) -> impl Iterator<Item = ReleaseIdentifier> + 'a {
            captures
                .filter_map(|i| i.get(2))
                .filter_map(|i| i.as_str().parse().ok())
                .map(ctor)
        };

        lazy_static! {
            static ref IDREGEX: Regex = Regex::new(r"(id|ID):(\d+)").unwrap();
            static ref RRQREGEX: Regex = Regex::new(r"(rrq|RRQ):(\d+)").unwrap();

        }

        r.ids = extract_matches(IDREGEX.captures_iter(&r.commit_summary), |i| ReleaseIdentifier::TargetProgress(i))
            .chain(extract_matches(RRQREGEX.captures_iter(&r.commit_summary), |i| ReleaseIdentifier::RRQ(i)))
            .collect::<Vec<ReleaseIdentifier>>();

        Ok(r)
    }
}


fn main() -> Result<()> {
    let opt = CliOptions::from_args();

    let repo = Repository::open(&opt.repo).with_context(|| {
        format!("failed to open repository: {:?}", opt.repo)
    })?;

    let commits: Vec<Commit> = git::get_commits(&repo, &opt).with_context(
        || "Failed to find commit difference between branches",
    )?;

    dbg!("Commits {:?}", &commits);

    let items = commits.into_iter()
        .map(ReleaseItem::new)
        .collect::<Result<Vec<_>>>()
        .context("Failed to convert commits")?;


    dbg!("items {:?}", &items);

    Ok(())
}
