use structopt::StructOpt;
use std::path::PathBuf;
use git2::{Repository, Oid, Commit};
use anyhow::{Context, Result, anyhow};
use lazy_static::lazy_static;
use regex::CaptureMatches;
use std::fmt;
use std::borrow::Borrow;
use serde::Deserialize;

use dotenv;
use crate::{ReleaseItem, ReleaseIdentifier};

use futures::stream::{FuturesUnordered, StreamExt};

use serde_json::Value;


#[serde(rename_all = "PascalCase")]
#[derive(Deserialize, Debug)]
struct PagedResponse<T>
{
    items: Vec<T>
}

#[serde(rename_all = "PascalCase")]
#[derive(Deserialize, Debug, Clone)]
pub struct Assignable
{
    pub id: u32,
    pub name: String,
    pub description: String,
    //#[serde(with = "serde_with::json::nested")]
    entity_type: EntityType
}


#[serde(rename_all = "PascalCase")]
#[derive(Deserialize, Debug, Clone)]
pub struct EntityType
{
    id: u32
}


pub struct TargetProcessSettings
{
    pub url: String,
    pub access_token: String
}

pub fn load_environment_settings() -> Option<TargetProcessSettings>
{
    let url = dotenv::var("RD_TARAGET_PROCESS_URL").ok()?;
    let access_token = dotenv::var("RD_ACCESS_TOKEN").ok()?;

    Some(TargetProcessSettings
    {
        access_token,
        url
    })
}

use url::Url;
use std::collections::HashMap;

async fn add_data_async(item: &mut ReleaseItem, url: &Url, settings: &TargetProcessSettings) -> anyhow::Result<()> {
    use reqwest::{Client};

    let mut assignables = vec![];

    for id  in item.ids.iter()
        {
        let url = url.clone();

        let filter =
            match id  {
                ReleaseIdentifier::RRQ(rrq) => format!("Name contains 'RRQ:{}'", rrq),
                ReleaseIdentifier::TargetProgress(tp) => format!("id eq {}", tp)
            };

            let url = url.join("api/v1/Assignables")?;
            let client = Client::new();
            let response = client.get(url.as_str())
                .query(&[("format", "json"),
                    ("access_token", &settings.access_token),
                    ("where", &filter),
                    ("take", "1"),
                    ("include", "[Id,Name,Description,InboundAssignables,OutboundAssignables,MasterRelations,SlaveRelations]")
                ])
                .send()
                .await
                .context("Target Process API call failed")?;

            let mut result: PagedResponse<Assignable> = response
                .json::<_>()
                .await?;

            assignables.append(&mut result.items);
    }

    if assignables.len() > 0 {
        item.assignables = Some(assignables);
    }

    Ok(())
}

pub async fn add_tp_data_async(items: &mut Vec<ReleaseItem>, settings: &TargetProcessSettings) -> anyhow::Result<()> {

    let tp_base = Url::parse(&settings.url).context("Failed to parse TP url")?;
    let mut futures = items
        .iter_mut()
        .filter(|i| i.ids.len() > 0)
        .map(|item|  add_data_async(item, &tp_base, settings))
        .collect::<FuturesUnordered<_>>();

    while let Some(f) = futures.next().await {
        if let (Err(e)) = f {
            eprintln!("Api Error {:?}", e);
        }
    }

    Ok(())
}