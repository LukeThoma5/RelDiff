use git2::{Repository, BranchType, Commit, Oid};
use anyhow::{Result, Context, anyhow};
use std::collections::HashSet;
use crate::{CliOptions, OPTS};

fn check_next_parent<'repo>(
    next_parent: &mut Option<Commit<'repo>>,
    commits: &mut Vec<Commit<'repo>>,
    seen_commits: &mut HashSet<Oid>,
) -> Result<Option<Oid>> {
    if let Some(parent) = next_parent.take() {
        let parent_id = parent.id();
        if seen_commits.contains(&parent_id) {
            return Ok(Some(parent_id));
        }

        seen_commits.insert(parent_id);

        match parent.parent_count() {
            0 => {
                commits.push(parent);
            }
            1 => {
                let next_commit = parent.parent(0).with_context(|| "Failed to find parent")?;
                *next_parent = Some(next_commit);

                commits.push(parent);
            }
            _ => {
                return Err(anyhow!(
                    "Expected 0-1 parents, found {}",
                    parent.parent_count()
                ))
            }
        }
    }

    Ok(None)
}

pub fn get_commits<'repo>(repo: &'repo Repository) -> Result<Vec<Commit<'repo>>> {
    let opts = &OPTS;
    let base_branch = repo.find_branch(&opts.base_branch, BranchType::Local)
        .with_context(|| {
            format!("Failed to find base branch: {}", &opts.base_branch)
        })?;

    let next_release_branch = repo.find_branch(&opts.release_branch, BranchType::Local)
        .with_context(|| {
            format!(
                "Failed to find next release branch: {}",
                &opts.release_branch
            )
        })?;

    let base = base_branch.into_reference().peel_to_commit().with_context(
        || {
            format!("Failed to find the commit for branch: {}", &opts.base_branch)
        },
    )?;

    let release = next_release_branch
        .into_reference()
        .peel_to_commit()
        .with_context(|| {
            format!(
                "Failed to find the commit for branch: {}",
                &opts.release_branch
            )
        })?;

    let mut release_commits = vec![];
    let mut base_commits = vec![];
    let mut seen_commits = HashSet::new();

    if base.id() == release.id() {
        return Err(anyhow!("Branches are identical"))
    }

    let mut next_base = Some(base);
    let mut next_release = Some(release);

    let match_commit: Option<Oid> = loop {
        if let Some(matched) = check_next_parent(
            &mut next_base,
            &mut base_commits,
            &mut seen_commits,
        ).with_context(|| "Failed to get parents of base")?
        {
            break Some(matched);
        }

        if let Some(matched) = check_next_parent(
            &mut next_release,
            &mut release_commits,
            &mut seen_commits,
        ).with_context(|| "Failed to get parents of release")?
        {
            break Some(matched);
        }

        if let (None, None) = (&next_base, &next_release) {
            break None;
        }
    };

    let match_commit = match_commit.ok_or(anyhow!(
        "Failed to find common ancestor of both {} and {}",
        &opts.base_branch,
        &opts.release_branch
    ))?;

    let base_commits = base_commits
        .into_iter()
        .take_while(|c| c.id() != match_commit)
        .collect::<Vec<_>>();

    let release_commits: Vec<_> = release_commits
        .into_iter()
        // Take until we reach the common point
        .take_while(|c| c.id() != match_commit)
        // Filter out any duplicates (e.g. cherry-picks)
        .filter(|rel| {
            !base_commits.iter().any(|base| {

                let b_author = base.author();
                let rel_author = rel.author();
                match (b_author.email(), rel_author.email()) {
                    (Some(b), Some(r)) => b == r && base.message() == rel.message(),
                    _ => false,
                }
            })
        })
        .collect();

    if release_commits.len() == 0 && base_commits.len() > 0 {
        return Err(anyhow!("Branches specified in wrong order"));
    }

    Ok(release_commits)
}