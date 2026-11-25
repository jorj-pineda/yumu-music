//! Which YouTube clients still serve a downloadable audio stream?
use rustypipe::client::{ClientType, RustyPipe};

#[tokio::main]
async fn main() {
    let id = std::env::args().nth(1).unwrap_or("wU26xVT_vBU".into());
    let rp = RustyPipe::builder().storage_dir("cache").botguard_bin(".tools/bin/rustypipe-botguard").build().unwrap();
    for ct in [ClientType::DesktopMusic, ClientType::Desktop, ClientType::Mobile,
               ClientType::Tv, ClientType::Android, ClientType::Ios] {
        let player = match rp.query().player_from_client(&id, ct).await {
            Ok(p) => p,
            Err(e) => { println!("{ct:?}: player error: {e}"); continue; }
        };
        let Some(s) = player.audio_streams.iter().find(|s| s.itag == 140)
            .or(player.audio_streams.first()) else {
            println!("{ct:?}: no audio streams"); continue;
        };
        let http = reqwest::Client::builder()
            .user_agent(rp.query().user_agent(ct).as_ref()).build().unwrap();
        let status = http.get(format!("{}&range=0-65535", s.url)).send().await
            .map(|r| r.status().to_string()).unwrap_or_else(|e| e.to_string());
        println!("{ct:?}: itag {} -> {status}", s.itag);
    }
}
