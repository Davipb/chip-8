use crate::core::Word;

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Timers {
    pub delay_timer: Word,
    pub sound_timer: Word,
    last_tick: Instant,
    delay_accumulator: Duration,
}

const DELAY_FREQUENCY: Duration = Duration::from_nanos(1000000000 / 60);

impl Timers {
    pub fn new() -> Timers {
        Timers {
            delay_timer: 0.into(),
            sound_timer: 0.into(),
            last_tick: Instant::now(),
            delay_accumulator: Duration::from_nanos(0),
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.delay_accumulator += elapsed;
        self.last_tick = now;

        while self.delay_accumulator >= DELAY_FREQUENCY {
            self.try_decrement_delay();
            self.delay_accumulator -= DELAY_FREQUENCY;
        }
    }

    fn try_decrement_delay(&mut self) {
        if self.delay_timer > 0.into() {
            self.delay_timer -= 1;
        }
    }
}
