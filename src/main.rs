//! Prototype: search YouTube Music and play the top result.
//!
//! Usage: cargo run --release -- "artist song"

mod player;

use std::{error::Error, time::Instant};

use rustypipe::client::RustyPipe;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let query = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    if query.is_empty() {
        eprintln!("usage: yumu <search terms>");
        std::process::exit(2);
    }

    // Search only, so rustypipe never needs its PO token helper.
    let rp = RustyPipe::builder()
        .storage_dir("cache")
        .no_botguard()
        .build()?;

    let t = Instant::now();
    let results = rp.query().music_search_tracks(&query).await?;
    println!("search: {} ms", t.elapsed().as_millis());
    for (i, track) in results.items.items.iter().take(5).enumerate() {
        let artists: Vec<_> = track.artists.iter().map(|a| a.name.as_str()).collect();
        println!("  {}. {} - {}", i + 1, artists.join(", "), track.name);
    }
    let track = results.items.items.first().ok_or("no results")?;

    println!("playing: {} (Ctrl-C to stop)", track.name);
    let id = track.id.clone();
    // Playback blocks until the track ends; keep it off the async runtime.
    tokio::task::spawn_blocking(move || player::play(&id)).await??;
    Ok(())
}
