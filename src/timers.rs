use crate::core::Word;

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Timers {
    pub delay_timer: Word,
    pub sound_timer: Word,
    last_tick: Instant,
}

impl Timers {
    pub fn new() -> Timers {
        Timers {
            delay_timer: Word::ZERO,
            sound_timer: Word::ZERO,
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();

        if now.duration_since(self.last_tick) > Duration::from_secs_f64(1f64 / 60f64) {
            self.last_tick = now;
        }
    }

    pub fn try_decrement(x: &mut Word) {
        if *x > Word::ZERO {
            *x -= Word::new(1);
        }
    }
}
