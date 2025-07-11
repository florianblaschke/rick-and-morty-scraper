mod fetch_endpoint;

use anyhow::Result;
use fetch_endpoint::fetch_endpoint;
use futures::future::try_join_all;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let endpoints = vec!["character", "location", "episode"];
    let mut threads = Vec::new();

    let start = Instant::now();
    for endpoint in endpoints {
        let task = tokio::spawn(async move { fetch_endpoint(endpoint).await });
        threads.push(task);
    }

    try_join_all(threads).await?;

    let end = Instant::now();
    println!("Total time: {:?}", end - start);

    Ok(())
}
