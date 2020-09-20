use crate::core::{Address, VoidResultChip8};
use crate::display::VideoMemory;
use crate::memory::MemoryMapper;
use crate::registers::Registers;
use crate::timers::Timers;

pub struct CPU {
    pub registers: Registers,
    pub timers: Timers,
    pub memory: MemoryMapper,
    pub stack: Vec<Address>,
    pub vram: VideoMemory,
}

impl CPU {
    fn new() -> CPU {
        CPU {
            registers: Registers::new(),
            timers: Timers::new(),
            memory: MemoryMapper::new(),
            stack: Vec::new(),
            vram: VideoMemory::new(),
        }
    }

    fn tick(&mut self) -> VoidResultChip8 {
        self.timers.tick();
        Ok(())
    }
}
