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

fn main() -> VoidResultChip8 {
    let result = do_main();

    match &result {
        Ok(_) => println!("Success"),
        Err(err) => println!("Failed: {}", err),
    };

    let mut buf = [0];
    io::stdin().read_exact(&mut buf)?;

    result
}

fn do_main() -> VoidResultChip8 {
    ansi_term::enable_ansi_support()
        .map_err(|x| Error::new(format!("Unable to turn on ANSI support: Error code {}", x)))?;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return print_help();
    }

    match args[1].as_str() {
        "run" => run(&args),
        "view" => disassemble(&args),
        "test-display" => test_display(),
        "test-input" => test_input(),
        _ => print_help(),
    }
}

fn print_help() -> VoidResultChip8 {
    println!("chip8 run <path>");
    println!("\temulate the ROM located at <path>");
    println!("chip8 view [-o] <path>");
    println!("\tprint a disassembly of the ROM located at <path>");
    println!("\t-o: Offset output by 1 byte");
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

    cpu.tick_loop()
}

fn disassemble(args: &Vec<String>) -> VoidResultChip8 {
    if args.len() < 3 || args.len() > 4 {
        return print_help();
    }

    let (mut file, offset) = if args.len() == 3 {
        (File::open(&args[2])?, false)
    } else if args[2] != "-o" {
        return print_help();
    } else {
        (File::open(&args[3])?, true)
    };

    let mut buffer = Vec::with_capacity(0x1000);
    file.read_to_end(&mut buffer)?;

    let mut i = 0;
    while i < buffer.len() {
        let addr = Address::new(0x0200 + i as u16);

        print!("{} | ", Blue.paint(addr.to_string()));

        if i == 0 && offset {
            println!("__{:02X}: Lone byte at the start of file", buffer[i]);
        } else if i + 1 >= buffer.len() {
            println!("{:02X}__: Lone byte at the end of file", buffer[i]);
        } else {
            let value = u16::from_be_bytes([buffer[i], buffer[i + 1]]);
            print!("{:04X}: ", value);

            match Opcode::decode(value) {
                Err(x) => println!("{} {}", Red.paint("ERROR"), Red.paint(x.to_string())),
                Ok(x) => println!("{}", color_opcode(x)),
            };
        };

        i += if i == 0 && offset { 1 } else { 2 };
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
