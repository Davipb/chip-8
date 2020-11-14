use super::{InputBuffer, KEY_NUM};
use crate::core::{Error, VoidResultChip8};
use winapi::{
    shared::minwindef::DWORD,
    um::{
        consoleapi,
        handleapi::INVALID_HANDLE_VALUE,
        processenv, winbase, wincon,
        wincon::ENABLE_PROCESSED_INPUT,
        wincontypes::{INPUT_RECORD, KEY_EVENT, KEY_EVENT_RECORD},
        winnls::CP_UTF8,
        winnt::HANDLE,
        winuser,
    },
};

pub struct NativeInputManager {
    handle: HANDLE,
    old_mode: DWORD,
}

impl NativeInputManager {
    pub fn new() -> NativeInputManager {
        unsafe {
            let handle = processenv::GetStdHandle(winbase::STD_INPUT_HANDLE);
            if handle == INVALID_HANDLE_VALUE {
                panic!("Unable to get a handle to STDIN");
            }

            let mut old_mode: DWORD = 0;
            if consoleapi::GetConsoleMode(handle, &mut old_mode) == 0 {
                panic!("Unable to read console mode");
            }

            if consoleapi::SetConsoleMode(handle, ENABLE_PROCESSED_INPUT) == 0 {
                panic!("Unable to set console mode");
            }

            if wincon::SetConsoleCP(CP_UTF8) == 0 {
                panic!("Unable to set console input to UTF-8");
            }

            if wincon::SetConsoleOutputCP(CP_UTF8) == 0 {
                panic!("Unable to set console output to UTF-8");
            }

            NativeInputManager { handle, old_mode }
        }
    }

    pub fn tick(&mut self, buffer: &mut InputBuffer) -> VoidResultChip8 {
        unsafe {
            loop {
                let mut pending: DWORD = 0;
                if consoleapi::GetNumberOfConsoleInputEvents(self.handle, &mut pending) == 0 {
                    return Err(Error::new_str(
                        "Unable to read number of pending console events",
                    ));
                }

                if pending == 0 {
                    break;
                }

                let mut events: [INPUT_RECORD; 256] = std::mem::zeroed();
                let mut events_len: DWORD = 0;
                if consoleapi::ReadConsoleInputW(self.handle, &mut events[0], 256, &mut events_len)
                    == 0
                {
                    return Err(Error::new_str("Unable to read pending console events"));
                }

                for i in 0..events_len {
                    let event = events[i as usize];
                    if event.EventType != KEY_EVENT {
                        continue;
                    }

                    let event = event.Event.KeyEvent();
                    self.handle_event(event, buffer)?;
                }
            }
        }
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: &KEY_EVENT_RECORD,
        buffer: &mut InputBuffer,
    ) -> VoidResultChip8 {
        let chip8_key: i32 = match event.wVirtualKeyCode as i32 {
            winuser::VK_NUMPAD0 => 0x0,
            winuser::VK_SPACE => 0x0,

            winuser::VK_NUMPAD1 => 0x1,
            0x5A => 0x1, // Z

            winuser::VK_NUMPAD2 => 0x2,
            winuser::VK_DOWN => 0x2,
            0x53 => 0x2, // S

            winuser::VK_NUMPAD3 => 0x3,
            0x43 => 0x3, // C

            winuser::VK_NUMPAD4 => 0x4,
            winuser::VK_LEFT => 0x4,
            0x41 => 0x4, // A

            winuser::VK_NUMPAD5 => 0x5,
            0x58 => 0x5, // X

            winuser::VK_NUMPAD6 => 0x6,
            winuser::VK_RIGHT => 0x6,
            0x44 => 0x6, // D

            winuser::VK_NUMPAD7 => 0x7,
            0x51 => 0x7, // Q

            winuser::VK_NUMPAD8 => 0x8,
            winuser::VK_UP => 0x8,
            0x57 => 0x8, // W

            winuser::VK_NUMPAD9 => 0x9,
            0x45 => 0x9, // E

            winuser::VK_DECIMAL => 0xA,
            winuser::VK_SEPARATOR => 0xA,
            winuser::VK_OEM_COMMA => 0xA,
            winuser::VK_OEM_PERIOD => 0xA,
            0x31 => 0xA, // 1
            0xC2 => 0xA, // Additional decimal separator in some keyboard layouts

            winuser::VK_DIVIDE => 0xB,
            0x32 => 0xB, // 2

            winuser::VK_MULTIPLY => 0xC,
            0x33 => 0xC, // 3

            winuser::VK_SUBTRACT => 0xD,
            0x52 => 0xD, // R

            winuser::VK_ADD => 0xE,
            0x46 => 0xE, // F

            winuser::VK_RETURN => 0xF,
            0x56 => 0xF, // V

            _ => -1,
        };

        if chip8_key >= 0 && chip8_key < KEY_NUM as i32 {
            if event.bKeyDown == 1 {
                buffer.hold(chip8_key)?;
            } else {
                buffer.release(chip8_key)?;
            }
        }

        Ok(())
    }
}

impl Drop for NativeInputManager {
    fn drop(&mut self) {
        unsafe {
            consoleapi::SetConsoleMode(self.handle, self.old_mode);
        }
    }
}
