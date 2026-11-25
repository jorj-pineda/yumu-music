//! How much of an iOS-client stream can be fetched without a PO token?
use rustypipe::client::{ClientType, RustyPipe};

#[tokio::main]
async fn main() {
    let id = std::env::args().nth(1).unwrap_or("wU26xVT_vBU".into());
    let rp = RustyPipe::builder()
        .storage_dir("cache")
        .botguard_bin(".tools/bin/rustypipe-botguard")
        .build()
        .unwrap();
    let p = rp
        .query()
        .player_from_client(&id, ClientType::Ios)
        .await
        .unwrap();
    let s = p.audio_streams.iter().find(|s| s.itag == 140).unwrap();
    let http = reqwest::Client::builder()
        .user_agent(rp.query().user_agent(ClientType::Ios).as_ref())
        .build()
        .unwrap();
    for (a, b) in [
        (0u64, 524_287u64),
        (0, 1_048_575),
        (0, 2_097_151),
        (0, 4_194_303),
        (1_048_576, 1_572_863),
        (2_097_152, 2_621_439),
        (4_194_304, 4_718_591),
    ] {
        let r = http
            .get(format!("{}&range={a}-{b}", s.url))
            .send()
            .await
            .unwrap();
        let st = r.status();
        let n = r.bytes().await.map(|b| b.len()).unwrap_or(0);
        println!(
            "range {:>4}K-{:>4}K -> {st} ({} KB)",
            a / 1024,
            (b + 1) / 1024,
            n / 1024
        );
    }
}
