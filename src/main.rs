mod core;
mod cpu;
mod display;
mod input;
mod memory;
mod opcodes;
mod registers;
mod timers;

use crate::core::{Address, Error, ResultChip8, VoidResultChip8, Word};
use crate::cpu::CPU;
use crate::display::{TerminalVideoListener, VideoMemory};
use crate::input::{InputManager, KEY_NUM};
use crate::memory::{ByteArrayMemory, MemoryRange, WriteMemory};
use crate::opcodes::Opcode;

use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use ansi_term::{
    self, ANSIString,
    Color::{Black, Blue, Green, Purple, Red, Yellow},
};

use ctrlc;

fn main() {
    ansi_term::enable_ansi_support().unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return print_help().unwrap();
    }

    match args[1].as_str() {
        "run" => run(&args),
        "view" => disassemble(&args),
        "test-display" => test_display(),
        "test-input" => test_input(),
        _ => print_help(),
    }
    .unwrap();
}

fn print_help() -> VoidResultChip8 {
    println!("chip8 run <path>");
    println!("\temulate the ROM located at <path>");
    println!("chip8 view <path>");
    println!("\tprint a disassembly of the ROM located at <path>");
    println!("chip8 test-display");
    println!("\ttests the terminal display mode");
    println!("chip8 test-input");
    println!("\ttests the terminal input manager");
    Ok(())
}

fn run(args: &Vec<String>) -> VoidResultChip8 {
    if args.len() != 3 {
        return print_help();
    }

    let mut file = File::open(&args[2])?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut cpu = CPU::new();
    cpu.memory.add(
        ByteArrayMemory::zero(0x1000 - 0x200),
        MemoryRange::new(0x200, 0xFFF),
        "Main Memory",
    )?;

    for i in 0..buffer.len() {
        let addr = Address::new(0x200 + i as u16);
        let word = Word::new(buffer[i]);
        cpu.memory.set(addr, word)?;
    }

    cpu.vram.attach(TerminalVideoListener::new())?;

    loop {
        cpu.tick()?;
    }

    Ok(())
}

fn disassemble(args: &Vec<String>) -> VoidResultChip8 {
    if args.len() != 3 {
        return print_help();
    }

    let mut file = File::open(&args[2])?;

    let mut buffer = Vec::with_capacity(0x1000);
    file.read_to_end(&mut buffer)?;
    if buffer.len() % 2 != 0 {
        return Err(Error::new_str("File size isn't a multiple of two"));
    }

    let mut i = 0;
    while i < buffer.len() {
        let addr = Address::new(0x0200 + i as u16);
        let value = u16::from_be_bytes([buffer[i], buffer[i + 1]]);

        print!("{} | {:04X}: ", Blue.paint(addr.to_string()), value);

        match Opcode::decode(value) {
            Err(x) => println!("{} {}", Red.paint("ERROR"), Red.paint(x.to_string())),
            Ok(x) => println!("{}", color_opcode(x)),
        }
        i += 2;
    }

    Ok(())
}

fn color_opcode<'a>(code: Opcode) -> ANSIString<'a> {
    let s = code.to_string();
    match code {
        Opcode::Nop => Black.bold().paint(s),
        Opcode::Return | Opcode::Jump(_) | Opcode::Call(_) | Opcode::CallNative(_) => {
            Purple.paint(s)
        }
        Opcode::CondJump { .. } => Green.paint(s),
        _ => Yellow.paint(s),
    }
}

fn test_display() -> VoidResultChip8 {
    let mut vram = VideoMemory::new();
    let id = vram.attach(TerminalVideoListener::new())?;

    let (tx, rx) = mpsc::sync_channel(0);
    ctrlc::set_handler(move || tx.send(()).unwrap())?;

    'main: loop {
        for y in 0..VideoMemory::BIT_HEIGHT {
            for x in 0..VideoMemory::BIT_WIDTH {
                if rx.try_recv().is_ok() {
                    break 'main;
                }

                vram.flip(x, y)?;
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    vram.detach(id)?;
    Ok(())
}

fn test_input() -> VoidResultChip8 {
    let mut input = InputManager::new();
    let (tx, rx) = mpsc::sync_channel(0);
    ctrlc::set_handler(move || tx.send(()).unwrap())?;

    // Clear screen and hide cursor
    io::stdout().write(b"\x1B[m\x1B[2J\x1B[?25l")?;

    loop {
        if rx.try_recv().is_ok() {
            break;
        }

        input.tick()?;

        // Cursor to top-left
        io::stdout().write(b"\x1B[H\x1B[?25l")?;

        for i in 0..KEY_NUM {
            let state = input.is_down(i)?;
            println!("{:X}: {}", i, if state { "Down" } else { "Up  " })
        }

        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
