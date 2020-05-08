use anyhow::{Context, Result};
use git2::{Commit, Repository};
use lazy_static::lazy_static;

use std::fmt;

use structopt::StructOpt;

mod cli;
mod git;
mod release_item;
mod target_process;

pub use cli::*;
pub use release_item::*;

lazy_static! {
    static ref OPTS: CliOptions = CliOptions::from_args();
}

pub struct ItemWriter<'item>(&'item Vec<ReleaseItem>);

impl fmt::Display for ItemWriter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Release {} -> {}\n",
            OPTS.base_branch, OPTS.release_branch
        )?;
        for (index, item) in self.0.iter().enumerate() {
            write!(f, "{}) {}\n", index + 1, item.commit_summary)?;
            if let Some(ref assignables) = item.assignables {
                for assignable in assignables {
                    write!(
                        f,
                        "\tRR Ref: {}\n\tName: {}\n\tDescription: {}\n",
                        assignable.id,
                        assignable.name,
                        assignable.nice_description()
                    )?;
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _dotenv_path = dotenv::dotenv().ok();

    let repo = Repository::open(&OPTS.repo)
        .with_context(|| format!("failed to open repository: {:?}", OPTS.repo))?;

    let commits: Vec<Commit> = git::get_commits(&repo)
        .with_context(|| "Failed to find commit difference between branches")?;

    let mut items = commits
        .into_iter()
        .map(ReleaseItem::new)
        .collect::<Result<Vec<_>>>()
        .context("Failed to convert commits")?;

    let tp = target_process::load_environment_settings();

    match (tp, OPTS.offline) {
        (Some(tp), false) => {
            if let Err(e) = target_process::add_tp_data_async(&mut items, &tp).await {
                eprintln!("{:?}", e);
            }
        },
        (None, false) => {
            eprintln!("Failed to load environment variables for TP integration. Run with --offline to hide this warning.")
        }
        _ => {}
    }

    let writer = ItemWriter(&items);

    println!("{}", writer);

    Ok(())
}
