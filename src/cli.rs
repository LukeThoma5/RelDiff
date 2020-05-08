use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Release Diff", about = "Generate a release summary")]
pub struct CliOptions {
    /// Input repo
    #[structopt(parse(from_os_str),
    default_value = "./",
    short = "r",
    long = "repo"
    )]
    pub repo: PathBuf,

    #[structopt(name = "BASE BRANCH")]
    pub base_branch: String,

    #[structopt(name = "RELEASE BRANCH")]
    pub release_branch: String,

    #[structopt(parse(from_os_str),
    short = "o",
    long = "output"
    )]
    pub output_file: Option<PathBuf>,

    #[structopt(long = "offline")]
    pub offline: bool
}
