use crate::core::{Error, ResultChip8, VoidResultChip8};
use std::convert::TryInto;

#[cfg_attr(target_family = "windows", path = "windows.rs")]
#[cfg_attr(target_family = "unix", path = "linux.rs")]
mod native;

pub struct InputBuffer {
    keys: [KeyState; 0x10],
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum KeyState {
    Released,
    Pressed,
    Held,
}

pub const KEY_NUM: usize = 0x10;

fn to_key_index(value: impl TryInto<usize>) -> ResultChip8<usize> {
    let index = match value.try_into() {
        Ok(x) => x,
        Err(_) => return Err(Error::new_str("Unable to convert argument to usize")),
    };

    if index >= KEY_NUM {
        return Err(Error::new_str("Key index out of bounds"));
    }

    Ok(index)
}

impl InputBuffer {
    pub fn new() -> InputBuffer {
        InputBuffer {
            keys: [KeyState::Released; KEY_NUM],
        }
    }

    pub fn is_down(&self, index_into: impl TryInto<usize>) -> ResultChip8<bool> {
        let index = to_key_index(index_into)?;
        Ok(self.keys[index] != KeyState::Released)
    }

    pub fn tick(&mut self) {
        for i in 0..KEY_NUM {
            self.keys[i] = match self.keys[i] {
                KeyState::Held => KeyState::Held,
                _ => KeyState::Released,
            };
        }
    }

    pub fn press(&mut self, index: impl TryInto<usize>) -> VoidResultChip8 {
        self.set_state(index, KeyState::Pressed)
    }

    pub fn hold(&mut self, index: impl TryInto<usize>) -> VoidResultChip8 {
        self.set_state(index, KeyState::Held)
    }

    pub fn release(&mut self, index: impl TryInto<usize>) -> VoidResultChip8 {
        self.set_state(index, KeyState::Released)
    }

    fn set_state(&mut self, index_into: impl TryInto<usize>, state: KeyState) -> VoidResultChip8 {
        let index = to_key_index(index_into)?;
        self.keys[index] = state;
        Ok(())
    }
}

pub struct InputManager {
    native: native::NativeInputManager,
    buffer: InputBuffer,
}

impl InputManager {
    pub fn new() -> InputManager {
        InputManager {
            native: native::NativeInputManager::new(),
            buffer: InputBuffer::new(),
        }
    }

    pub fn is_down(&self, index_into: impl TryInto<usize>) -> ResultChip8<bool> {
        self.buffer.is_down(index_into)
    }

    pub fn tick(&mut self) -> VoidResultChip8 {
        self.native.tick(&mut self.buffer)?;
        self.buffer.tick();
        Ok(())
    }
}
