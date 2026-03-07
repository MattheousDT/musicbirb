use anyhow::{anyhow, Result};
use libmpv::{FileState, Mpv};
use std::sync::Mutex;

pub struct Player {
    mpv: Mutex<Mpv>,
}

impl Player {
    pub fn new() -> Result<Self> {
        let mpv = Mpv::new().map_err(|e| anyhow!("Failed to initialize MPV: {:?}", e))?;
        mpv.set_property("keep-open", "yes")
            .map_err(|e| anyhow!("{:?}", e))?;
        mpv.set_property("vo", "null")
            .map_err(|e| anyhow!("{:?}", e))?;
        Ok(Self {
            mpv: Mutex::new(mpv),
        })
    }

    pub fn enqueue(&self, url: &str) -> Result<()> {
        self.mpv
            .lock()
            .unwrap()
            .playlist_load_files(&[(url, FileState::AppendPlay, None)])
            .map_err(|e| anyhow!("{:?}", e))
    }

    pub fn next(&self) -> Result<()> {
        self.mpv
            .lock()
            .unwrap()
            .command("playlist-next", &["weak"])
            .map_err(|e| anyhow!("{:?}", e))
    }

    pub fn prev(&self) -> Result<()> {
        self.mpv
            .lock()
            .unwrap()
            .command("playlist-prev", &["weak"])
            .map_err(|e| anyhow!("{:?}", e))
    }

    pub fn seek_relative(&self, seconds: f64) -> Result<()> {
        self.mpv
            .lock()
            .unwrap()
            .command("seek", &[&seconds.to_string(), "relative"])
            .map_err(|e| anyhow!("{:?}", e))
    }

    pub fn pause(&self) -> Result<()> {
        self.mpv
            .lock()
            .unwrap()
            .set_property("pause", true)
            .map_err(|e| anyhow!("{:?}", e))
    }

    pub fn resume(&self) -> Result<()> {
        self.mpv
            .lock()
            .unwrap()
            .set_property("pause", false)
            .map_err(|e| anyhow!("{:?}", e))
    }

    pub fn get_state(&self) -> (Option<usize>, f64, bool) {
        let mpv = self.mpv.lock().unwrap();
        let pos: i64 = mpv.get_property("playlist-pos").unwrap_or(-1);
        let time: f64 = mpv.get_property("time-pos").unwrap_or(0.0);
        let paused: bool = mpv.get_property("pause").unwrap_or(false);
        (
            if pos >= 0 { Some(pos as usize) } else { None },
            time,
            paused,
        )
    }
}
