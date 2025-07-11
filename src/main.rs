use std::{fs, time::Instant};

use anyhow::Result;
use futures::future::try_join_all;
use reqwest;
use serde::{Deserialize, Serialize};

const CHARACTERS: &str = "https://rickandmortyapi.com/api/character?page=1";

#[derive(Serialize, Debug, Deserialize)]
struct Info {
    count: i64,
    pages: i64,
    next: Option<String>,
    prev: Option<String>,
}

#[derive(Serialize, Debug, Deserialize)]
struct Origin {
    name: String,
    url: String, //contains API reference locations
}

#[derive(Serialize, Debug, Deserialize)]
struct Location {
    name: String,
    url: String, //contains API reference locations
}

#[derive(Serialize, Debug, Deserialize)]
struct CharacterResult {
    id: i64,
    name: String,
    status: String,
    species: String,
    r#type: String,
    gender: String,
    origin: Origin,
    location: Location,
    image: String,
    episode: Vec<String>,
    url: String,     // contains API refernce to the character,
    created: String, //date
}

#[derive(Serialize, Debug, Deserialize)]
struct ApiResult {
    info: Info,
    results: Vec<CharacterResult>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut threads = Vec::new();

    let start = Instant::now();
    let pages = reqwest::get(CHARACTERS)
        .await?
        .json::<ApiResult>()
        .await?
        .info
        .pages;

    for page in 1..=pages {
        let task = tokio::spawn(async move {
            let res = reqwest::get(format!(
                "https://rickandmortyapi.com/api/character?page={}",
                page
            ))
            .await?
            .json::<ApiResult>()
            .await?
            .results;

            Ok::<Vec<CharacterResult>, anyhow::Error>(res)
        });
        threads.push(task);
    }

    let mut all_characters = try_join_all(threads)
        .await?
        .into_iter()
        .filter_map(|result| result.ok())
        .flatten()
        .collect::<Vec<CharacterResult>>();

    all_characters.sort_by_key(|character| character.id);

    let _res = fs::write(
        "src/characters.json",
        serde_json::to_string(&all_characters).unwrap(),
    );

    let finish = Instant::now();
    println!("This took: {:?}", finish - start);

    Ok(())
}
