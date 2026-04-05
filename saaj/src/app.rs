// === src/app.rs ===
use ratatui::widgets::ListState;
use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::process::Command;

#[derive(Default, PartialEq, Eq)]
pub enum AppMode {
    #[default]
    Splash,
    Main,
    DownloadPrompt,
    Downloading,
}

#[derive(Default, PartialEq, Eq)]
pub enum RepeatMode {
    #[default]
    None,
    One,
    All,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TrackJson {
    pub duration: String,
    pub id: String,
    pub title: String,
    pub uploader: String,
}

#[derive(Clone)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: u16,
    pub duration_secs: u64,
    pub format: String,
    pub bitrate: String,
    pub id: String,
}

pub struct App {
    pub mode: AppMode,
    pub tracks: Vec<Track>,
    pub list_state: ListState,
    pub selected: usize,
    pub current_index: usize,
    pub elapsed_secs: f64,
    pub volume: u8,
    pub is_playing: bool,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub spectrum_data: Vec<u64>,
    pub should_quit: bool,
    pub lcg_seed: u64,
    pub download_input: String,
    pub audio_stream: Option<OutputStream>,
    pub audio_sink: Option<Sink>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            mode: AppMode::Splash,
            tracks: vec![],
            list_state: ListState::default(),
            selected: 0,
            current_index: 0,
            elapsed_secs: 0.0,
            volume: 72,
            is_playing: false,
            shuffle: false,
            repeat: RepeatMode::None,
            spectrum_data: vec![0; 16],
            should_quit: false,
            lcg_seed: 123456789,
            download_input: String::new(),
            audio_stream: None,
            audio_sink: None,
        };
        app.init_audio();
        app.load_tracks();
        app
    }

    pub fn init_audio(&mut self) {
        if let Ok((stream, handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&handle) {
                self.audio_stream = Some(stream);
                self.audio_sink = Some(sink);
                self.set_sink_volume();
            }
        }
    }

    pub fn set_sink_volume(&self) {
        if let Some(sink) = &self.audio_sink {
            sink.set_volume(self.volume as f32 / 100.0);
        }
    }

    pub fn load_tracks(&mut self) {
        let info_path = "../Info_files/info.json";
        let mut loaded = false;
        
        if let Ok(file) = File::open(info_path) {
            let reader = BufReader::new(file);
            if let Ok(json_tracks) = serde_json::from_reader::<_, Vec<TrackJson>>(reader) {
                self.tracks = json_tracks.into_iter().map(|t| Track {
                    title: t.title.clone(),
                    artist: t.uploader.clone(),
                    album: "Local Download".to_string(),
                    year: 2024,
                    duration_secs: t.duration.parse().unwrap_or(0),
                    format: "MP3".to_string(),
                    bitrate: "320kbps".to_string(),
                    id: t.id.clone(),
                }).collect();
                loaded = true;
            }
        }
        
        if !loaded || self.tracks.is_empty() {
            self.tracks.push(Track {
                title: "No tracks found".to_string(),
                artist: "Press 'd' to download".to_string(),
                album: "N/A".to_string(),
                year: 0,
                duration_secs: 0,
                format: "N/A".to_string(),
                bitrate: "N/A".to_string(),
                id: "".to_string(),
            });
        } else {
            // Validate against audioloc to ensure file exists (optional, keeping it simple)
        }
        
        if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
            self.selected = 0;
        }
    }

    pub fn play_current_track(&mut self) {
        if self.tracks.is_empty() { return; }
        let track = &self.tracks[self.current_index];
        if track.id.is_empty() { return; }
        
        let file_path = format!("../audioloc/{}.mp3", track.id);
        if let Some(sink) = &self.audio_sink {
            sink.stop(); 
            if let Ok(file) = File::open(&file_path) {
                if let Ok(source) = Decoder::new(BufReader::new(file)) {
                    sink.append(source);
                    sink.play();
                    self.is_playing = true;
                    self.elapsed_secs = 0.0;
                }
            }
        }
    }

    pub fn toggle_play(&mut self) {
        if let Some(sink) = &self.audio_sink {
            if self.is_playing {
                sink.pause();
            } else {
                sink.play();
            }
        }
        self.is_playing = !self.is_playing;
    }

    pub fn progress_percent(&self) -> u16 {
        if self.tracks.is_empty() { return 0; }
        let duration = self.tracks[self.current_index].duration_secs as f64;
        if duration <= 0.0 { return 0; }
        let p = (self.elapsed_secs / duration * 100.0) as u16;
        p.min(100)
    }

    pub fn elapsed_str(&self) -> String {
        let mins = (self.elapsed_secs / 60.0) as u64;
        let secs = (self.elapsed_secs % 60.0) as u64;
        format!("{}:{:02}", mins, secs)
    }

    pub fn duration_str(&self) -> String {
        if self.tracks.is_empty() { return "0:00".to_string(); }
        let total = self.tracks[self.current_index].duration_secs;
        let mins = total / 60;
        let secs = total % 60;
        format!("{}:{:02}", mins, secs)
    }

    pub fn current_track(&self) -> &Track {
        &self.tracks[self.current_index]
    }

    pub fn select_next(&mut self) {
        if self.tracks.is_empty() { return; }
        self.selected = (self.selected + 1) % self.tracks.len();
        self.list_state.select(Some(self.selected));
    }

    pub fn select_prev(&mut self) {
        if self.tracks.is_empty() { return; }
        if self.selected == 0 {
            self.selected = self.tracks.len() - 1;
        } else {
            self.selected -= 1;
        }
        self.list_state.select(Some(self.selected));
    }

    pub fn seek_forward(&mut self, secs: f64) {
        if self.tracks.is_empty() { return; }
        let duration = self.tracks[self.current_index].duration_secs as f64;
        self.elapsed_secs = (self.elapsed_secs + secs).min(duration);
    }

    pub fn seek_backward(&mut self, secs: f64) {
        self.elapsed_secs = (self.elapsed_secs - secs).max(0.0);
    }

    pub fn volume_up(&mut self) {
        self.volume = self.volume.saturating_add(5).min(100);
        self.set_sink_volume();
    }

    pub fn volume_down(&mut self) {
        self.volume = self.volume.saturating_sub(5);
        self.set_sink_volume();
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
    }

    pub fn cycle_repeat(&mut self) {
        self.repeat = match self.repeat {
            RepeatMode::None => RepeatMode::One,
            RepeatMode::One => RepeatMode::All,
            RepeatMode::All => RepeatMode::None,
        };
    }

    fn next_rand(&mut self) -> u64 {
        self.lcg_seed = self.lcg_seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.lcg_seed
    }

    pub fn tick(&mut self) {
        if self.is_playing {
            if !self.tracks.is_empty() {
                let duration = self.tracks[self.current_index].duration_secs as f64;
                self.elapsed_secs += 0.1;
                
                // Track actual elapsed time from sink or just estimate it
                if self.elapsed_secs >= duration {
                    self.elapsed_secs = duration;
                    if self.repeat == RepeatMode::One {
                        self.play_current_track();
                    } else {
                        self.is_playing = false; 
                    }
                }
            }

            for i in 0..16 {
                let r = self.next_rand() % 101;
                self.spectrum_data[i] = r;
            }
        } else {
            for i in 0..16 {
                if self.spectrum_data[i] > 0 {
                    self.spectrum_data[i] = self.spectrum_data[i].saturating_sub(5);
                }
            }
        }
    }
}

/// Background download function 
pub fn download_song(query: String) -> bool {
    let output = Command::new("yt-dlp")
        .arg(format!("ytsearch1:{}", query))
        .arg("--get-id")
        .output();
    
    if let Ok(out) = output {
        let id = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if id.is_empty() { return false; }
        
        let video_url = format!("https://www.youtube.com/watch?v={}", id);
        
        let json_cmd = Command::new("yt-dlp")
            .arg(&video_url)
            .arg("--print")
            .arg("{\"title\": \"%(title)s\", \"uploader\": \"%(uploader)s\", \"duration\": \"%(duration)s\"}")
            .output();
            
        if let Ok(json_out) = json_cmd {
            let json_str = String::from_utf8_lossy(&json_out.stdout).trim().to_string();
            
            // Handle potentially multiple lines or errors
            if let Ok(mut parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                parsed["id"] = serde_json::Value::String(id.clone());
                
                let info_path = "../Info_files/info.json";
                let mut all_data = if let Ok(file) = File::open(info_path) {
                    serde_json::from_reader(BufReader::new(file)).unwrap_or_else(|_| vec![])
                } else {
                    vec![]
                };
                
                all_data.push(parsed);
                if let Ok(out_file) = File::create(info_path) {
                    let _ = serde_json::to_writer_pretty(out_file, &all_data);
                }
                
                let _dl_cmd = Command::new("yt-dlp")
                    .arg("-x")
                    .arg("-q")
                    .arg("--write-thumbnail")
                    .arg("--audio-quality")
                    .arg("0")
                    .arg("--audio-format")
                    .arg("mp3")
                    .arg("-o")
                    .arg(format!("../audioloc/{}.%(ext)s", id))
                    .arg(&video_url)
                    .output();
                    
                return true;
            }
        }
    }
    false
}