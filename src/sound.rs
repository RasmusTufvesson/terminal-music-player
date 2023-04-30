use std::{io::BufReader, fs::{File, self}, time::{Duration, Instant}, path::Path, ffi::OsStr};
use lofty::{read_from_path, AudioFile, LoftyError};
use rand::seq::SliceRandom;
use rodio::{self, Sink, Decoder};

pub struct PlayerState {
    pub song: Song,
    pub play_time: Duration,
    pub index: usize,
    pub paused: bool,
}

#[derive(Clone)]
pub struct Song {
    pub duration: Duration,
    pub name: String,
    pub path: String,
}

impl Song {
    pub fn from_path(path: String) -> Result<Self, LoftyError> {
        let tagged_file = read_from_path(&path)?;
        let properties = tagged_file.properties();
        let name = Path::new(&path).file_stem().unwrap().to_str().unwrap().to_string().replace("_", " ");
        return Ok(Self { duration: properties.duration(), name: name, path: path });
    }
}

pub struct Player {
    pub sink: Sink,
    pub state: PlayerState,
    pub song_selection: Vec<Song>,
    rng: rand::rngs::ThreadRng,
    start_instant: Instant,
}

impl Player {
    pub fn new(sink: Sink, mut songs: Vec<Song>) -> Self {
        let mut rng = rand::thread_rng();
        songs.shuffle(&mut rng);
        let first_song = songs[0].clone();
        let first_song_copy = first_song.clone();
        let state = PlayerState { song: first_song, play_time: Duration::ZERO, index: 0, paused: false };
        let mut player = Self {
            sink: sink,
            state: state,
            song_selection: songs,
            rng: rng,
            start_instant: Instant::now()
        };
        player.pause();
        player.add_to_queue(first_song_copy);
        return player;
    }

    pub fn add_to_queue(&self, song: Song) {
        let audio = Self::audio_from_path(&song.path);
        self.sink.append(audio);
    }

    fn audio_from_path(path: &str) -> Decoder<BufReader<File>> {
        let file = BufReader::new(File::open(path).unwrap());
        let source: Decoder<BufReader<File>> = Decoder::new(file).unwrap();
        return source;
    }

    fn next_song(&mut self) {
        self.state.index += 1;
        if self.state.index == self.song_selection.len() {
            self.state.index = 0;
            self.song_selection.shuffle(&mut self.rng);
        }
        let next_song = &self.song_selection[self.state.index];
        self.add_to_queue(next_song.clone());
        self.state.play_time = Duration::ZERO;
        self.start_instant = Instant::now();
        self.state.song = next_song.clone();
    }

    pub fn skip_song(&mut self) {
        self.sink.clear();
        self.next_song();
        self.sink.play();
        self.state.paused = false;
    }

    pub fn pause(&mut self) {
        self.state.paused = !self.state.paused;
        if self.state.paused {
            self.sink.pause();
            self.state.play_time += self.start_instant.elapsed();
        } else {
            self.sink.play();
            self.start_instant = Instant::now();
        }
    }

    pub fn update(&mut self) {
        if self.sink.len() == 0 {
            self.next_song();
        }
    }

    pub fn get_song_progress(&self) -> f64 {
        if !self.state.paused {
            return (self.state.play_time + self.start_instant.elapsed()).as_millis() as f64 / self.state.song.duration.as_millis() as f64;
        } else {
            return self.state.play_time.as_millis() as f64 / self.state.song.duration.as_millis() as f64;
        }
    }
}

pub fn get_song_selection<'a>(dir_path: &str) -> Vec<Song> {
    let dir = fs::read_dir(dir_path).unwrap();
    let mut song_selection: Vec<Song> = vec![];
    for item in dir {
        match item {
            Ok(entry) => {
                if let Ok(filename) = entry.file_name().into_string() {
                    if let Some(extension) = Path::new(&filename).extension() {
                        if extension == "mp3" {
                            match Song::from_path(dir_path.to_owned()+"/"+&filename) {
                                Ok(song) => {
                                    song_selection.push(song);
                                }
                                Err(_) => {}
                            }
                        }
                    }
                } else {
                    let filename = entry.file_name();
                    let filename_string = filename.to_string_lossy().to_string();
                    if let Some(extension) = Path::new(&filename_string).extension() {
                        if extension == OsStr::new("mp3") {
                            match Song::from_path(dir_path.to_owned()+"/"+&filename_string) {
                                Ok(song) => {
                                    song_selection.push(song);
                                }
                                Err(_) => {}
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }
    }
    return song_selection;
}