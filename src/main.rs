use structopt::StructOpt;
use std::path::PathBuf;
use git2::{Repository, Oid, Commit};
use anyhow::{Context, Result, anyhow};
use lazy_static::lazy_static;
use regex::CaptureMatches;
use std::fmt;
use std::borrow::Borrow;

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

    #[structopt(parse(from_os_str),
    short = "o",
    long = "output"
    )]
    output_file: Option<PathBuf>,

    #[structopt(long = "offline")]
    no_network: bool
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
lazy_static! {
    static ref OPTS: CliOptions = CliOptions::from_args();
}

pub struct ItemWriter<'item>(&'item Vec<ReleaseItem>);

impl fmt::Display for ItemWriter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Release {} -> {}\n", OPTS.base_branch, OPTS.release_branch)?;
        for item in self.0 {
            write!(f, "{}", item.commit_summary)?;
        }

        Ok(())
    }
}

use dotenv;

struct TargetProcessSettings
{
    pub url: String,
    pub access_token: String
}

fn load_environment_settings() -> Option<TargetProcessSettings>
{
    let url = dotenv::var("RD_TARAGET_PROCESS_URL").ok()?;
    let access_token = dotenv::var("RD_ACCESS_TOKEN").ok()?;

    Some(TargetProcessSettings
    {
        access_token,
        url
    })
}

async fn add_tp_data_async(items: &mut Vec<ReleaseItem>, settings: &TargetProcessSettings) {

}

#[tokio::main]
async fn main() -> Result<()> {
    let dotenv_path = dotenv::dotenv().ok();
    dbg!("Loading environment from {:?}", dotenv_path);

    let repo = Repository::open(&OPTS.repo).with_context(|| {
        format!("failed to open repository: {:?}", OPTS.repo)
    })?;

    let commits: Vec<Commit> = git::get_commits(&repo).with_context(
        || "Failed to find commit difference between branches",
    )?;

    // dbg!("Commits {:?}", &commits);

    let mut items = commits.into_iter()
        .map(ReleaseItem::new)
        .collect::<Result<Vec<_>>>()
        .context("Failed to convert commits")?;

    dbg!("items {:?}", &items);


    let tp = load_environment_settings();

    match (tp, OPTS.no_network) {
        (Some(tp), false) => {
            add_tp_data_async(&mut items, &tp).await;
        },
        (None, true) => {
            eprintln!("Failed to load environment variables for TP integration. Run with --offline to hide this warning.")
        }
        _ => {}
    }

    let writer = ItemWriter(&items);

    println!("{}", writer);

    Ok(())
}
