use rodio::{OutputStream, Sink};

mod sound;
mod app;

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let player = sound::Player::new(sink);
}
