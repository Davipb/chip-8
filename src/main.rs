mod core;
mod cpu;
mod display;
mod memory;
mod opcodes;
mod registers;
mod timers;

use crate::core::Address;
use crate::display::{TerminalVideoListener, VideoMemory};
use crate::opcodes::Opcode;

use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use ansi_term::{
    self, ANSIString,
    Color::{Black, Blue, Green, Purple, Red, Yellow},
};

use ctrlc;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return print_help();
    }

    match args[1].as_str() {
        "run" => unimplemented!(),
        "view" => disassemble(&args),
        "test_display" => test_display(),
        _ => print_help(),
    }
}

fn print_help() {
    println!("chip8 run <path>");
    println!("\temulate the ROM located at <path>");
    println!("chip8 view <path>");
    println!("\tprint a disassembly of the ROM located at <path>");
    println!("chip8 test_display");
    println!("\ttests the terminal display mode");
}

fn disassemble(args: &Vec<String>) {
    if args.len() != 3 {
        return print_help();
    }

    let mut file = File::open(&args[2]).unwrap();

    let mut buffer = Vec::with_capacity(0x1000);
    file.read_to_end(&mut buffer).unwrap();
    if buffer.len() % 2 != 0 {
        panic!("File size isn't a multiple of two")
    }

    ansi_term::enable_ansi_support().unwrap();

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

fn test_display() {
    ansi_term::enable_ansi_support().unwrap();

    let mut vram = VideoMemory::new();
    let id = vram.attach(TerminalVideoListener::new()).unwrap();

    let (tx, rx) = mpsc::sync_channel(0);
    ctrlc::set_handler(move || tx.send(()).unwrap()).unwrap();

    'main: loop {
        for y in 0..VideoMemory::BIT_HEIGHT {
            for x in 0..VideoMemory::BIT_WIDTH {
                if rx.try_recv().is_ok() {
                    break 'main;
                }

                vram.flip(x, y).unwrap();
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    vram.detach(id).unwrap();
}
