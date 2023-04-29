use std::{io::BufReader, fs::File};

use rodio::{self, Sink, Decoder};

pub struct Player {
    pub sink: Sink
}

impl Player {
    pub fn new(sink: Sink) -> Self {
        let player = Self {
            sink: sink
        };
        return player;
    }

    pub fn add_to_queue(&self, path: &str) {
        let audio = Self::audio_from_path(path);
        self.sink.append(audio);
    }

    pub fn audio_from_path(path: &str) -> Decoder<BufReader<File>> {
        let file = BufReader::new(File::open(path).unwrap());
        let source: Decoder<BufReader<File>> = Decoder::new(file).unwrap();
        return source;
    }

    pub fn wait_for_audio(&self) {
        self.sink.sleep_until_end();
    }
}