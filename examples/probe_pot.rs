use rustypipe::client::{ClientType, RustyPipe};
#[tokio::main]
async fn main() {
    let rp = RustyPipe::builder().storage_dir("cache")
        .botguard_bin(".tools/bin/rustypipe-botguard").build().unwrap();
    println!("botguard version: {:?}", rp.version_botguard().await);
    let p = rp.query().player_from_client("wU26xVT_vBU", ClientType::Ios).await.unwrap();
    let s = p.audio_streams.iter().find(|s| s.itag == 140).unwrap();
    println!("iOS url has pot param: {}", s.url.contains("&pot="));
}
