use std::{cmp::Ordering, collections::HashMap, fs, time::Instant};

use anyhow::Result;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
struct Info {
    count: u64,
    pages: u64,
    next: Option<String>,
    prev: Option<String>,
}

#[derive(Serialize, Debug, Deserialize, Eq, PartialEq)]
struct ResultWithId {
    id: u64,
    #[serde(flatten)]
    data: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Debug, Deserialize)]
struct ApiResult {
    info: Info,
    results: Vec<ResultWithId>,
}

impl PartialOrd for ResultWithId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResultWithId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

pub async fn fetch_endpoint(endpoint: &str) -> Result<()> {
    let mut threads = Vec::new();
    let constructed_endpoint = format!("https://rickandmortyapi.com/api/{}", endpoint);

    let start = Instant::now();
    let pages = reqwest::get(constructed_endpoint.clone())
        .await?
        .json::<ApiResult>()
        .await?
        .info
        .pages;

    for page in 1..=pages {
        let cloned_endpoint = constructed_endpoint.clone();
        let task = tokio::spawn(async move {
            let res = reqwest::get(format!("{}?page={}", cloned_endpoint, page))
                .await?
                .json::<ApiResult>()
                .await?
                .results;

            Ok::<Vec<ResultWithId>, anyhow::Error>(res)
        });
        threads.push(task);
    }

    let mut all_characters = try_join_all(threads)
        .await?
        .into_iter()
        .filter_map(|result| result.ok())
        .flatten()
        .collect::<Vec<ResultWithId>>();

    all_characters.sort_unstable();

    let _res = fs::write(
        format!("src/{}.json", endpoint),
        serde_json::to_string(&all_characters).unwrap(),
    );

    let finish = Instant::now();
    println!("{endpoint} took: {:?}", finish - start);

    Ok(())
}
