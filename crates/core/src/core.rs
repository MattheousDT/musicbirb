use crate::api::SubsonicClient;
use crate::models::Track;
use crate::player::Player;
use anyhow::Result;
use image::DynamicImage;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

#[derive(Default)]
struct PlaybackState {
    index: Option<usize>,
    time: f64,
    paused: bool,
}

pub struct Musicbirb {
    api: Arc<SubsonicClient>,
    player: Arc<Player>,
    queue: Arc<Mutex<Vec<Track>>>,
    playback_state: Arc<Mutex<PlaybackState>>,
    current_art: Arc<Mutex<Option<Arc<DynamicImage>>>>,
    art_cache: Arc<Mutex<HashMap<String, Arc<DynamicImage>>>>,
    art_version: Arc<AtomicU64>,
}

impl Musicbirb {
    pub fn new(api: SubsonicClient, player: Player) -> Arc<Self> {
        let core = Arc::new(Self {
            api: Arc::new(api),
            player: Arc::new(player),
            queue: Arc::new(Mutex::new(Vec::new())),
            playback_state: Arc::new(Mutex::new(PlaybackState::default())),
            current_art: Arc::new(Mutex::new(None)),
            art_cache: Arc::new(Mutex::new(HashMap::new())),
            art_version: Arc::new(AtomicU64::new(0)),
        });

        core.start_background_workers();
        core
    }

    fn start_background_workers(&self) {
        let player = Arc::clone(&self.player);
        let pb_state = Arc::clone(&self.playback_state);
        let queue = Arc::clone(&self.queue);
        let current_art = Arc::clone(&self.current_art);
        let art_cache = Arc::clone(&self.art_cache);
        let art_version = Arc::clone(&self.art_version);
        let api = Arc::clone(&self.api);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(33));
            let mut last_index = None;

            loop {
                interval.tick().await;

                let (index, time, paused) = player.get_state();
                if let Ok(mut lock) = pb_state.lock() {
                    lock.index = index;
                    lock.time = time;
                    lock.paused = paused;
                }

                if index != last_index {
                    last_index = index;
                    let track = index.and_then(|i| queue.lock().unwrap().get(i).cloned());

                    if let Some(t) = track {
                        if let Some(art_id) = t.cover_art {
                            let ver = art_version.fetch_add(1, Ordering::SeqCst) + 1;

                            if let Some(cached) = art_cache.lock().unwrap().get(&art_id) {
                                *current_art.lock().unwrap() = Some(Arc::clone(cached));
                                continue;
                            }

                            *current_art.lock().unwrap() = None;

                            let a = Arc::clone(&api);
                            let c_art = Arc::clone(&current_art);
                            let c_cache = Arc::clone(&art_cache);
                            let c_ver = Arc::clone(&art_version);

                            tokio::spawn(async move {
                                if let Ok(bytes) = a.get_cover_art_bytes(&art_id).await {
                                    if c_ver.load(Ordering::SeqCst) == ver {
                                        if let Ok(img) = image::load_from_memory(&bytes) {
                                            let arc_img = Arc::new(img);
                                            c_cache
                                                .lock()
                                                .unwrap()
                                                .insert(art_id, Arc::clone(&arc_img));

                                            if c_ver.load(Ordering::SeqCst) == ver {
                                                *c_art.lock().unwrap() = Some(arc_img);
                                            }
                                        }
                                    }
                                }
                            });
                        } else {
                            *current_art.lock().unwrap() = None;
                        }
                    } else {
                        *current_art.lock().unwrap() = None;
                    }
                }
            }
        });
    }

    pub async fn queue_track(&self, id: &str) -> Result<()> {
        let track = self.api.get_track(id).await?;
        self.enqueue_track(track).await
    }

    pub async fn queue_album(&self, id: &str) -> Result<usize> {
        let tracks = self.api.get_album_tracks(id).await?;
        let count = tracks.len();
        for t in tracks {
            self.enqueue_track(t).await?;
        }
        Ok(count)
    }

    pub async fn queue_playlist(&self, id: &str) -> Result<usize> {
        let tracks = self.api.get_playlist_tracks(id).await?;
        let count = tracks.len();
        for t in tracks {
            self.enqueue_track(t).await?;
        }
        Ok(count)
    }

    async fn enqueue_track(&self, track: Track) -> Result<()> {
        let url = self.api.get_stream_url(&track.id).await?;
        self.player.enqueue(&url)?;
        self.queue.lock().unwrap().push(track);
        Ok(())
    }

    pub fn next(&self) -> Result<()> {
        self.player.next()
    }
    pub fn prev(&self) -> Result<()> {
        self.player.prev()
    }
    pub fn seek(&self, seconds: f64) -> Result<()> {
        self.player.seek_relative(seconds)
    }

    pub fn toggle_pause(&self) -> Result<()> {
        let paused = self.playback_state.lock().unwrap().paused;
        if paused {
            self.player.resume()
        } else {
            self.player.pause()
        }
    }

    pub fn current_track(&self) -> Option<Track> {
        let index = self.playback_state.lock().unwrap().index?;
        self.queue.lock().unwrap().get(index).cloned()
    }

    pub fn queue(&self) -> Vec<Track> {
        self.queue.lock().unwrap().clone()
    }
    pub fn queue_position(&self) -> usize {
        self.playback_state.lock().unwrap().index.unwrap_or(0)
    }
    pub fn playback_time(&self) -> f64 {
        self.playback_state.lock().unwrap().time
    }
    pub fn is_paused(&self) -> bool {
        self.playback_state.lock().unwrap().paused
    }

    pub fn current_cover_art(&self) -> Option<Arc<DynamicImage>> {
        self.current_art.lock().unwrap().clone()
    }

    pub fn raw_player(&self) -> &Player {
        &self.player
    }
}
