# Yumu

A small, fast, native desktop client for YouTube Music.

The YouTube Music website is slow and heavy, and the desktop apps that exist
mostly wrap it in a browser. Yumu is a real native app written in Rust, in the
spirit of [Spotifast](https://github.com/crmne/spotifast): low memory, near
idle CPU, and an interface built for listening.

> **Status: early prototype.** There is no interface yet. The current build is
> a command-line test that searches and plays the top result.

## Try the prototype

You need Rust and `yt-dlp` (`brew install yt-dlp` on macOS, which also
installs the Deno runtime yt-dlp needs).

```sh
cargo run --release -- "artist song"
```

## Contributing

Bug reports and focused fixes are welcome. Please read
[CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

## Credits

Yumu builds on [Spotifast](https://github.com/crmne/spotifast) and takes ideas
from [Kaset](https://github.com/sozercan/kaset). See
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## License

[MIT](LICENSE). Yumu is an unofficial client. It is not affiliated with,
endorsed by or sponsored by Google or YouTube.
