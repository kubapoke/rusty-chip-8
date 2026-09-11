use rodio::source::SineWave;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub fn delay_timer_work(delay_timer: Arc<AtomicU8>) {
    loop {
        let start = Instant::now();

        let current = delay_timer.load(Ordering::Relaxed);
        let next = current.saturating_sub(1);

        _ = delay_timer.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed);

        let end = Instant::now();
        sleep(Duration::from_millis(MAX_MILLIS_WAIT).saturating_sub(end.duration_since(start)));
    }
}

pub fn sound_timer_work(sound_timer: Arc<AtomicU8>) {
    let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("Unable to open audio stream");
    let player = rodio::Player::connect_new(handle.mixer());
    let source = SineWave::new(SOUND_FREQUENCY);
    player.append(source);

    loop {
        let start = Instant::now();

        let current = sound_timer.load(Ordering::Relaxed);
        let next = current.saturating_sub(1);

        _ = sound_timer.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed);

        match current {
            0 => { player.pause() }
            _ => { player.play() }
        }

        let end = Instant::now();
        sleep(Duration::from_millis(MAX_MILLIS_WAIT).saturating_sub(end.duration_since(start)));
    }
}

const MAX_MILLIS_WAIT: u64 = 16;
const SOUND_FREQUENCY: f32 = 440.0;
