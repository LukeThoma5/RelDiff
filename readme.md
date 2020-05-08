# Release Diff

## What is it?

Generate a formatted diff of 2 git branches to determine what has changed as part of a release. Pulls in extra data from Target Process based on RRQ: and id: in commit messages.

## Usage

From within the git repo run.
`reldiff old_release new_release`

All options:

```
Release Diff 0.1.0
Generate a release summary

USAGE:
    reldiff [FLAGS] [OPTIONS] <BASE BRANCH> <RELEASE BRANCH>

FLAGS:
    -h, --help       Prints help information
        --offline
    -V, --version    Prints version information

OPTIONS:
    -o, --output <output-file>
    -r, --repo <repo>             Input repo [default: ./]

ARGS:
    <BASE BRANCH>
    <RELEASE BRANCH>
```

## Installation

1. Install rust https://www.rust-lang.org/learn/get-started
2. cargo install --git https://github.com/LukeThoma5/RelDiff
