//! Plays a track by letting yt-dlp fetch its audio and piping the bytes into
//! rodio while they arrive.
//!
//! yt-dlp is the only reliable way to get full YouTube streams today, and its
//! stream URLs are rejected when another HTTP client reuses them, so it has to
//! download the bytes itself. We read its stdout on a thread into a growing
//! buffer; the decoder reads that buffer and waits whenever it gets ahead of
//! the download.

use std::{
    fmt,
    io::{self, Read, Seek, SeekFrom},
    process::{Command, Stdio},
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Instant,
};

/// The AAC stream in an m4a container. symphonia decodes it; Opus in WebM
/// does not decode yet.
const AUDIO_FORMAT: &str = "140";

/// What went wrong while starting or playing a track.
#[derive(Debug)]
pub enum PlayError {
    /// yt-dlp is not installed or not on PATH.
    YtDlpMissing,
    /// yt-dlp could not be started for another reason.
    Spawn(io::Error),
    /// No audio output device could be opened.
    Output(String),
    /// The downloaded bytes could not be decoded.
    Decode(String),
}

impl fmt::Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::YtDlpMissing => write!(
                f,
                "yt-dlp was not found. Install it (on macOS: brew install yt-dlp) and try again."
            ),
            Self::Spawn(e) => write!(f, "could not start yt-dlp: {e}"),
            Self::Output(e) => write!(f, "could not open the audio output: {e}"),
            Self::Decode(e) => write!(f, "could not decode the track: {e}"),
        }
    }
}

impl std::error::Error for PlayError {}

/// Bytes downloaded so far, shared between the download thread and readers.
#[derive(Default)]
struct State {
    data: Vec<u8>,
    done: bool,
    error: Option<String>,
}

#[derive(Default)]
struct Shared {
    state: Mutex<State>,
    grew: Condvar,
}

/// A download that fills in on a background thread.
#[derive(Clone, Default)]
pub struct Download {
    shared: Arc<Shared>,
}

impl Download {
    /// Copy `source` into the buffer on a new thread until it ends.
    pub fn spawn<R: Read + Send + 'static>(mut source: R) -> Self {
        let download = Self::default();
        let filler = download.clone();
        thread::spawn(move || {
            let mut chunk = vec![0; 64 * 1024];
            loop {
                match source.read(&mut chunk) {
                    Ok(0) => break filler.finish(None),
                    Ok(n) => filler.push(&chunk[..n]),
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => break filler.finish(Some(e.to_string())),
                }
            }
        });
        download
    }

    fn push(&self, bytes: &[u8]) {
        let mut state = self.shared.state.lock().unwrap();
        state.data.extend_from_slice(bytes);
        self.shared.grew.notify_all();
    }

    fn finish(&self, error: Option<String>) {
        let mut state = self.shared.state.lock().unwrap();
        state.done = true;
        state.error = error;
        self.shared.grew.notify_all();
    }

    /// A reader over the download that starts at the first byte.
    pub fn reader(&self) -> StreamReader {
        StreamReader {
            download: self.clone(),
            pos: 0,
        }
    }

    /// Bytes downloaded so far.
    pub fn len(&self) -> usize {
        self.shared.state.lock().unwrap().data.len()
    }
}

/// Reads a [`Download`], waiting for bytes that have not arrived yet.
pub struct StreamReader {
    download: Download,
    pos: u64,
}

impl Read for StreamReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let shared = &self.download.shared;
        let state = shared
            .grew
            .wait_while(shared.state.lock().unwrap(), |s| {
                !s.done && self.pos >= s.data.len() as u64
            })
            .unwrap();
        if self.pos >= state.data.len() as u64 {
            return match &state.error {
                Some(e) => Err(io::Error::other(e.clone())),
                None => Ok(0),
            };
        }
        let start = self.pos as usize;
        let n = buf.len().min(state.data.len() - start);
        buf[..n].copy_from_slice(&state.data[start..start + n]);
        self.pos += n as u64;
        Ok(n)
    }
}

impl Seek for StreamReader {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let target = match to {
            SeekFrom::Start(n) => Some(n),
            SeekFrom::Current(d) => self.pos.checked_add_signed(d),
            // The decoder is non-seekable, so it never asks for the end.
            SeekFrom::End(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "the end of a download in progress is unknown",
                ));
            }
        };
        self.pos = target
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek before the start"))?;
        Ok(self.pos)
    }
}

/// Start yt-dlp for a YouTube Music track and return its download.
pub fn fetch(video_id: &str) -> Result<Download, PlayError> {
    let url = format!("https://music.youtube.com/watch?v={video_id}");
    let mut child = Command::new("yt-dlp")
        .args(["--quiet", "--no-warnings", "--no-playlist"])
        .args(["--format", AUDIO_FORMAT, "--output", "-", &url])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => PlayError::YtDlpMissing,
            _ => PlayError::Spawn(e),
        })?;
    let stdout = child.stdout.take().expect("stdout is piped");
    let download = Download::spawn(stdout);
    // Reap the process when it exits so it does not linger as a zombie.
    thread::spawn(move || child.wait());
    Ok(download)
}

/// Fetch a track and play it to the end on the default output device.
pub fn play(video_id: &str) -> Result<(), PlayError> {
    let started = Instant::now();
    let download = fetch(video_id)?;

    let sink = rodio::DeviceSinkBuilder::open_default_sink()
        .map_err(|e| PlayError::Output(e.to_string()))?;
    let out = rodio::Player::connect_new(sink.mixer());

    // Not seekable: the m4a header comes first, so the decoder can start on
    // the first bytes instead of scanning to the end of a file still arriving.
    let decoder = rodio::Decoder::builder()
        .with_data(download.reader())
        .with_hint("m4a")
        .with_mime_type("audio/mp4")
        .with_seekable(false)
        .build()
        .map_err(|e| PlayError::Decode(e.to_string()))?;
    out.append(decoder);
    println!(
        "first sound after {} ms ({} KB downloaded)",
        started.elapsed().as_millis(),
        download.len() / 1024
    );
    out.sleep_until_end();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, time::Duration};

    /// A source that yields whatever chunks the test sends, then ends.
    struct ChannelSource {
        chunks: mpsc::Receiver<io::Result<Vec<u8>>>,
        pending: Vec<u8>,
    }

    impl Read for ChannelSource {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.pending.is_empty() {
                match self.chunks.recv() {
                    Ok(chunk) => self.pending = chunk?,
                    Err(_) => return Ok(0),
                }
            }
            let n = buf.len().min(self.pending.len());
            buf[..n].copy_from_slice(&self.pending[..n]);
            self.pending.drain(..n);
            Ok(n)
        }
    }

    fn channel_download() -> (mpsc::Sender<io::Result<Vec<u8>>>, Download) {
        let (tx, rx) = mpsc::channel();
        let source = ChannelSource {
            chunks: rx,
            pending: Vec::new(),
        };
        (tx, Download::spawn(source))
    }

    #[test]
    fn reads_everything_in_order() {
        let download = Download::spawn(io::Cursor::new(b"hello world".to_vec()));
        let mut out = String::new();
        download.reader().read_to_string(&mut out).unwrap();
        assert_eq!(out, "hello world");
    }

    #[test]
    fn read_waits_for_bytes_that_have_not_arrived() {
        let (tx, download) = channel_download();
        let mut reader = download.reader();
        let waiting = thread::spawn(move || {
            let mut buf = [0; 3];
            reader.read_exact(&mut buf).unwrap();
            buf
        });
        thread::sleep(Duration::from_millis(50));
        assert!(!waiting.is_finished(), "read returned before data arrived");
        tx.send(Ok(b"abc".to_vec())).unwrap();
        assert_eq!(&waiting.join().unwrap(), b"abc");
    }

    #[test]
    fn seek_from_start_and_current() {
        let download = Download::spawn(io::Cursor::new(b"0123456789".to_vec()));
        let mut reader = download.reader();
        reader.seek(SeekFrom::Start(4)).unwrap();
        let mut buf = [0; 2];
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"45");
        reader.seek(SeekFrom::Current(-4)).unwrap();
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"23");
        assert!(reader.seek(SeekFrom::Current(-10)).is_err());
    }

    #[test]
    fn download_error_surfaces_after_the_bytes_before_it() {
        let (tx, download) = channel_download();
        tx.send(Ok(b"ok".to_vec())).unwrap();
        tx.send(Err(io::Error::other("connection reset"))).unwrap();
        let mut reader = download.reader();
        let mut buf = [0; 2];
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"ok");
        let err = reader.read(&mut buf).unwrap_err();
        assert!(err.to_string().contains("connection reset"));
    }
}
