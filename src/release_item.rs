use crate::target_process::Assignable;
use anyhow::anyhow;
use git2::{Commit, Oid};
use lazy_static::lazy_static;
use regex::CaptureMatches;
use std::iter::Iterator;

#[derive(Debug, Clone)]
pub enum ReleaseIdentifier {
    RRQ(u32),
    TargetProgress(u32),
}

#[derive(Debug, Clone)]
pub struct ReleaseItem {
    pub sha1: Oid,
    pub commit_summary: String,
    pub ids: Vec<ReleaseIdentifier>,
    pub assignables: Option<Vec<Assignable>>,
}

impl ReleaseItem {
    pub fn new(commit: Commit<'_>) -> anyhow::Result<Self> {
        let mut r = Self {
            sha1: commit.id(),
            commit_summary: commit
                .message()
                .ok_or(anyhow!("Failed to get commit message for {}", commit.id()))?
                .lines()
                .next()
                .ok_or(anyhow!(
                    "Failed to get first line of commit message for {}",
                    commit.id()
                ))?
                .to_owned(),
            ids: Vec::new(),
            assignables: None,
        };

        use regex::Regex;

        fn extract_matches<'a>(
            captures: CaptureMatches<'a, 'a>,
            ctor: impl Fn(u32) -> ReleaseIdentifier + 'a,
        ) -> impl Iterator<Item = ReleaseIdentifier> + 'a {
            captures
                .filter_map(|i| i.get(2))
                .filter_map(|i| i.as_str().parse().ok())
                .map(ctor)
        };

        lazy_static! {
            static ref IDREGEX: Regex = Regex::new(r"(id|ID):(\d+)").unwrap();
            static ref RRQREGEX: Regex = Regex::new(r"(rrq|RRQ):(\d+)").unwrap();
        }

        r.ids = extract_matches(IDREGEX.captures_iter(&r.commit_summary), |i| {
            ReleaseIdentifier::TargetProgress(i)
        })
        .chain(extract_matches(
            RRQREGEX.captures_iter(&r.commit_summary),
            |i| ReleaseIdentifier::RRQ(i),
        ))
        .collect::<Vec<ReleaseIdentifier>>();

        Ok(r)
    }
}
