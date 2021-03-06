use crate::core::{Error, ResultChip8, VoidResultChip8};
use std::collections::HashMap;
use std::io::{self, Write};

pub trait VideoListener {
    fn on_attach(&mut self, _memory: &mut VideoMemory) -> VoidResultChip8 {
        Ok(())
    }

    fn on_change(&mut self, _x: usize, _y: usize, _value: bool) -> VoidResultChip8 {
        Ok(())
    }

    fn on_clear(&mut self) -> VoidResultChip8 {
        Ok(())
    }
    fn on_detach(&mut self, _memory: &mut VideoMemory) -> VoidResultChip8 {
        Ok(())
    }
}

pub struct VideoMemory {
    data: [u8; VideoMemory::VRAM_LEN],
    listeners: HashMap<u8, Box<dyn VideoListener>>,
    next_listener_id: u8,
}

impl VideoMemory {
    pub const BIT_WIDTH: usize = 64;
    pub const BIT_HEIGHT: usize = 32;
    const VRAM_LEN: usize = (VideoMemory::BIT_WIDTH * VideoMemory::BIT_HEIGHT) / 8;

    pub fn new() -> VideoMemory {
        VideoMemory {
            data: [0; VideoMemory::VRAM_LEN],
            listeners: HashMap::new(),
            next_listener_id: 0,
        }
    }

    pub fn get(&self, x: impl Into<usize>, y: impl Into<usize>) -> ResultChip8<bool> {
        let (byte_index, bit_offset) = self.get_index_offset(x.into(), y.into())?;

        let bit = (self.data[byte_index] >> bit_offset) & 1;
        Ok(bit == 1)
    }

    pub fn set(
        &mut self,
        x_into: impl Into<usize>,
        y_into: impl Into<usize>,
        value: bool,
    ) -> ResultChip8<bool> {
        let (x, y) = (x_into.into(), y_into.into());

        let (byte_index, bit_offset) = self.get_index_offset(x, y)?;
        let old_bit = (self.data[byte_index] >> bit_offset) & 1;

        let mask = 1 << bit_offset;
        if value {
            self.data[byte_index] |= mask;
        } else {
            self.data[byte_index] &= !mask;
        }

        for listener in self.listeners.values_mut() {
            listener.on_change(x, y, value)?;
        }

        Ok(old_bit == 1)
    }

    pub fn flip(
        &mut self,
        x_into: impl Into<usize>,
        y_into: impl Into<usize>,
    ) -> ResultChip8<bool> {
        let (x, y) = (x_into.into(), y_into.into());
        let (byte_index, bit_offset) = self.get_index_offset(x, y)?;

        self.data[byte_index] ^= 1 << bit_offset;
        let bit = (self.data[byte_index] >> bit_offset) & 1;

        for listener in self.listeners.values_mut() {
            listener.on_change(x, y, bit == 1)?;
        }

        Ok(bit == 1)
    }

    pub fn clear(&mut self) -> VoidResultChip8 {
        self.data = [0; VideoMemory::VRAM_LEN];

        for listener in self.listeners.values_mut() {
            listener.on_clear()?;
        }

        Ok(())
    }

    fn get_index_offset(&self, x: usize, y: usize) -> ResultChip8<(usize, usize)> {
        let x = x % VideoMemory::BIT_WIDTH;
        let y = y % VideoMemory::BIT_HEIGHT;

        let bit_index = x + (y * VideoMemory::BIT_WIDTH);
        let byte_index = bit_index / 8;
        let bit_offset = bit_index % 8;

        Ok((byte_index, bit_offset))
    }

    pub fn attach<T>(&mut self, listener: T) -> ResultChip8<u8>
    where
        T: VideoListener + 'static,
    {
        let mut listener_box = Box::new(listener);
        listener_box.on_attach(self)?;

        let id = self.next_listener_id;
        self.next_listener_id += 1;

        self.listeners.insert(id, listener_box);
        Ok(id)
    }

    pub fn detach(&mut self, id: u8) -> VoidResultChip8 {
        match self.listeners.remove(&id) {
            None => Ok(()),
            Some(mut x) => x.on_detach(self),
        }
    }
}

impl Drop for VideoMemory {
    fn drop(&mut self) {
        let ids: Vec<u8> = self.listeners.keys().map(Clone::clone).collect();
        for id in ids {
            self.detach(id)
                .expect("Unable to detach listeners when dropping");
        }
    }
}

pub struct TerminalVideoListener {
    started: bool,
}

fn flush() -> VoidResultChip8 {
    io::stdout().flush()?;
    Ok(())
}

fn csi(buf: &[u8]) -> VoidResultChip8 {
    io::stdout().write(b"\x1B[")?;
    io::stdout().write(buf)?;
    Ok(())
}

impl TerminalVideoListener {
    pub fn new() -> TerminalVideoListener {
        TerminalVideoListener { started: false }
    }
}

impl VideoListener for TerminalVideoListener {
    fn on_attach(&mut self, _: &mut VideoMemory) -> VoidResultChip8 {
        if self.started {
            return Ok(());
        }
        csi(b"?1049h")?; // Enable alternative screen buffer
        csi(b"m")?; // Reset formatting
        csi(b"2J")?; // Clear screen
        csi(b"H")?; // Cursor to top-left
        csi(b"?25l")?; // Hide cursor
        io::stdout().write(b"\x1B]2;CHIP8\x07")?; // Set window title
        flush()?;

        self.started = true;
        Ok(())
    }

    fn on_detach(&mut self, _: &mut VideoMemory) -> VoidResultChip8 {
        if !self.started {
            return Ok(());
        }
        csi(b"?25h")?; // Show cursor
        csi(b"?1049l")?; // Disable alternative screen buffer
        flush()?;

        self.started = false;
        Ok(())
    }

    fn on_change(&mut self, x: usize, y: usize, value: bool) -> VoidResultChip8 {
        // Cursor to (x; y)
        csi(b"")?;
        print!("{};{}H", y + 1, x + 1);

        if value {
            csi(b"7m")?;
        } else {
            csi(b"27m")?;
        }
        print!(" ");
        flush()?;
        Ok(())
    }

    fn on_clear(&mut self) -> VoidResultChip8 {
        csi(b"27m")?; // Set color to black
        csi(b"2J")?; // Clear screen
        flush()?;
        Ok(())
    }
}
