//! Attach a botguard PO token to an iOS stream URL by hand.
use rustypipe::client::{ClientType, RustyPipe};
#[tokio::main]
async fn main() {
    let id = "wU26xVT_vBU";
    let rp = RustyPipe::builder()
        .storage_dir("cache")
        .no_botguard()
        .build()
        .unwrap();
    let visitor = rp.query().get_visitor_data(false).await.unwrap();
    let out = std::process::Command::new(".tools/bin/rustypipe-botguard")
        .args([id, visitor.as_str()])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    let tokens: Vec<&str> = text.split_whitespace().collect();
    println!("botguard printed {} fields", tokens.len());
    let p = rp
        .query()
        .visitor_data(&visitor)
        .player_from_client(id, ClientType::Ios)
        .await
        .unwrap();
    let s = p.audio_streams.iter().find(|s| s.itag == 140).unwrap();
    let http = reqwest::Client::builder()
        .user_agent(rp.query().user_agent(ClientType::Ios).as_ref())
        .build()
        .unwrap();
    for (label, tok) in [("content", tokens.first()), ("session", tokens.get(1))] {
        let Some(tok) = tok else { continue };
        let url = format!("{}&pot={}&range=2097152-2621439", s.url, tok);
        let st = http.get(url).send().await.unwrap().status();
        println!("iOS + {label} token, range 2-2.5MB -> {st}");
    }
}
