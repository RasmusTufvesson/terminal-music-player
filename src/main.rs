use std::{time::Duration, fs};
use serde::Deserialize;
use rodio::{OutputStream, Sink};

mod sound;
mod app;

#[derive(Deserialize)]
struct Config {
   dir: String,
}

fn main() {
    let config = fs::read_to_string("config.toml")
        .expect("Should have been able to read the file");
    let config: Config = toml::from_str(&config).unwrap();
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let songs = sound::get_song_selection(&config.dir);
    let player = sound::Player::new(sink, songs);
    let mut app = app::App::new(player);
    app.run(Duration::from_millis(25));
}
