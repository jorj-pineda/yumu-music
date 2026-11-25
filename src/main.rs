//! Spike: search YouTube Music, fetch one track's audio, play it.
//!
//! Usage: cargo run --release -- "artist song"

use std::{error::Error, io::Cursor, time::Instant};

use rustypipe::{
    client::RustyPipe,
    model::{AudioCodec, AudioFormat},
    param::StreamFilter,
};

/// YouTube throttles single large requests; fetch the file in ranges.
const CHUNK: u64 = 4 * 1024 * 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let query = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    if query.is_empty() {
        eprintln!("usage: yumu <search terms>");
        std::process::exit(2);
    }

    let rp = RustyPipe::builder().storage_dir("cache").build()?;

    let t = Instant::now();
    let results = rp.query().music_search_tracks(&query).await?;
    println!("search: {} ms", t.elapsed().as_millis());
    for (i, track) in results.items.items.iter().take(5).enumerate() {
        let artists: Vec<_> = track.artists.iter().map(|a| a.name.as_str()).collect();
        println!(
            "  {}. {} - {} [{}]",
            i + 1,
            artists.join(", "),
            track.name,
            track.id
        );
    }
    let track = results.items.items.first().ok_or("no results")?;

    let t = Instant::now();
    let player = rp.query().player(&track.id).await?;
    println!(
        "player: {} ms via {:?}, {} audio streams",
        t.elapsed().as_millis(),
        player.client_type,
        player.audio_streams.len()
    );

    // AAC in m4a decodes with symphonia; Opus in webm does not yet.
    let filter = StreamFilter::new()
        .no_video()
        .audio_formats([AudioFormat::M4a])
        .audio_codecs([AudioCodec::Mp4a]);
    let stream = player.select_audio_stream(&filter).ok_or("no m4a stream")?;
    println!(
        "stream: itag {} {} kbps, {} KB",
        stream.itag,
        stream.bitrate / 1000,
        stream.size / 1024
    );

    let t = Instant::now();
    // googlevideo rejects a download whose User-Agent does not match the
    // client that asked for the stream.
    let http = reqwest::Client::builder()
        .user_agent(rp.query().user_agent(player.client_type).as_ref())
        .build()?;
    let mut bytes = Vec::with_capacity(stream.size as usize);
    let mut start = 0;
    while start < stream.size {
        let end = (start + CHUNK).min(stream.size) - 1;
        let resp = http
            .get(format!("{}&range={start}-{end}", stream.url))
            .send()
            .await?
            .error_for_status()?;
        bytes.extend_from_slice(&resp.bytes().await?);
        start = end + 1;
    }
    println!(
        "download: {} ms, {} KB",
        t.elapsed().as_millis(),
        bytes.len() / 1024
    );

    let sink = rodio::DeviceSinkBuilder::open_default_sink()?;
    let out = rodio::Player::connect_new(sink.mixer());
    let len = bytes.len() as u64;
    let decoder = rodio::Decoder::builder()
        .with_data(Cursor::new(bytes))
        .with_byte_len(len)
        .with_hint("m4a")
        .with_seekable(true)
        .build()?;
    println!("playing: {} (Ctrl-C to stop)", track.name);
    out.append(decoder);
    out.sleep_until_end();
    Ok(())
}
