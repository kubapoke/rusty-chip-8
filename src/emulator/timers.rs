use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};
use rodio::source::SineWave;

pub fn delay_timer_work(delay_timer: Arc<Mutex<u8>>) {
    loop {
        let start = Instant::now();

        let mut timer = delay_timer.lock().expect("Unable to lock delay timer");
        *timer = timer.saturating_sub(1);

        let end = Instant::now();
        sleep(Duration::from_millis(MAX_MILLIS_WAIT).saturating_sub(end.duration_since(start)));
    }
}

pub fn sound_timer_work(sound_timer: Arc<Mutex<u8>>) {
    let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("Unable to open audio stream");
    let player = rodio::Player::connect_new(&handle.mixer());
    let source = SineWave::new(SOUND_FREQUENCY);
    player.append(source);

    loop {
        let start = Instant::now();

        let mut timer = sound_timer.lock().expect("Unable to lock delay timer");

        match *timer {
            0 => { player.pause() },
            _ => { player.play() },
        }

        *timer = timer.saturating_sub(1);

        let end = Instant::now();
        sleep(Duration::from_millis(MAX_MILLIS_WAIT).saturating_sub(end.duration_since(start)));
    }
}

const MAX_MILLIS_WAIT: u64 = 16;
const SOUND_FREQUENCY: f32 = 440.0;
