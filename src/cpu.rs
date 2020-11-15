use crate::core::{Address, Error, VoidResultChip8, Word};
use crate::display::VideoMemory;
use crate::input::{InputManager, KEY_NUM};
use crate::memory::{ByteArrayMemory, MemoryMapper, MemoryRange, ReadMemory, WriteMemory};
use crate::opcodes::{Condition, Opcode, OpcodeParam};
use crate::registers::Registers;
use crate::timers::Timers;
use rand::random;

const DIGITS_ROM_DATA: &[u8; 0x50] = include_bytes!["digits.bin"];

pub struct CPU {
    pub registers: Registers,
    pub timers: Timers,
    pub memory: MemoryMapper,
    pub stack: Vec<Address>,
    pub vram: VideoMemory,
    pub input: InputManager,
}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu = CPU {
            registers: Registers::new(),
            timers: Timers::new(),
            memory: MemoryMapper::new(),
            stack: Vec::new(),
            vram: VideoMemory::new(),
            input: InputManager::new(),
        };

        let digits_rom = ByteArrayMemory::new(DIGITS_ROM_DATA);
        cpu.memory.add_read(
            digits_rom,
            MemoryRange::new(0x000, DIGITS_ROM_DATA.len() - 1),
            "Digits ROM",
        );

        cpu
    }

    pub fn tick(&mut self) -> VoidResultChip8 {
        self.timers.tick();
        self.input.tick()?;

        let opcode_high = self.memory.get(self.registers.program_counter)?.into();
        let opcode_low = self
            .memory
            .get(self.registers.program_counter + 1u16)?
            .into();
        let opcode = Opcode::decode(u16::from_be_bytes([opcode_high, opcode_low]))?;
        self.interpret(opcode)?;

        Ok(())
    }

    fn interpret(&mut self, opcode: Opcode) -> VoidResultChip8 {
        let mut increment_pc = true;

        match opcode {
            // Value registers
            Opcode::Assign {
                left_reg,
                right,
                op,
            } => {
                let (result, carry) = op.evaluate(
                    self.registers.values[left_reg as usize],
                    self.get_value(right),
                );

                self.registers.values[left_reg as usize] = result;

                match (carry, right) {
                    (Some(c), OpcodeParam::Register(_)) => {
                        self.registers.values[0xF] = Word::new(if c { 1 } else { 0 })
                    }
                    _ => {}
                };
                Ok(())
            }

            Opcode::Random { reg, mask } => {
                self.registers.values[reg as usize] = Word::new(random::<u8>()) & mask;
                Ok(())
            }

            // Address Register
            Opcode::AssignAddress(addr) => {
                self.registers.address = addr;
                Ok(())
            }

            Opcode::GetCharacterAddress(reg) => {
                let value = self.registers.values[reg as usize];
                self.registers.address = (value * 5).into();
                Ok(())
            }

            // Flow Control
            Opcode::Return => {
                let addr = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::new_str("Tried to return from an empty stack"))?;
                self.registers.program_counter = addr;
                Ok(())
            }

            Opcode::Jump(addr) => {
                increment_pc = false;
                self.registers.program_counter = addr;
                Ok(())
            }

            Opcode::OffsetJump(addr) => {
                increment_pc = false;
                self.registers.program_counter = addr + self.registers.values[0];
                Ok(())
            }

            Opcode::Call(addr) => {
                increment_pc = false;
                self.stack.push(self.registers.program_counter);
                self.registers.program_counter = addr;
                Ok(())
            }

            Opcode::CondJump { left, right, cond } => {
                if cond.evaluate(self.get_value(left), self.get_value(right)) {
                    self.registers.program_counter += 2;
                }
                Ok(())
            }

            // Graphics
            Opcode::ClearScreen => self.vram.clear(),

            Opcode::Draw {
                x: x_reg,
                y: y_reg,
                height,
            } => {
                let x: usize = self.registers.values[x_reg as usize].into();
                let y: usize = self.registers.values[y_reg as usize].into();

                let sprite = self
                    .memory
                    .get_range(MemoryRange::new_len(self.registers.address, height))?;

                self.registers.values[0xF] = 0.into();

                for dy in 0..height {
                    let byte = sprite[dy as usize];
                    for dx in 0..8 {
                        let bit = ((byte >> (7 - dx)) & 1) == 1.into();
                        if !bit {
                            continue;
                        }

                        let unset = self.vram.flip(x + dx, y + (dy as usize))?;
                        if unset {
                            self.registers.values[0xF] = 1.into();
                        }
                    }
                }

                Ok(())
            }

            // IO
            Opcode::BlockOnKey(reg) => {
                increment_pc = false;

                for i in 0..KEY_NUM {
                    if self.input.is_down(i)? {
                        self.registers.values[reg as usize] = i.into();
                        increment_pc = true;
                        break;
                    }
                }
                Ok(())
            }

            // Misc
            Opcode::Nop => Ok(()),

            Opcode::WriteBCD(reg) => {
                let value = self.registers.values[reg as usize];
                let base_addr = self.registers.address + 2;

                for i in 0..=2 {
                    let addr = base_addr - i;
                    let digit = (value / 10u8.pow(i)) % 10;
                    self.memory.set(addr, digit)?;
                }

                Ok(())
            }

            Opcode::DumpValueRegisters(end) => {
                for i in 0..=end {
                    let addr = self.registers.address + i;
                    self.memory.set(addr, self.registers.values[i as usize])?;
                }
                Ok(())
            }

            Opcode::LoadValueRegisters(end) => {
                for i in 0..=end {
                    let addr = self.registers.address + i;
                    self.registers.values[i as usize] = self.memory.get(addr)?;
                }
                Ok(())
            }

            x => unimplemented!("Opcode not supported: {}", x),
        }?;

        if increment_pc {
            self.registers.program_counter += 2u16;
        }

        Ok(())
    }

    fn get_value(&self, param: OpcodeParam) -> Word {
        match param {
            OpcodeParam::Immediate(x) => x,
            OpcodeParam::Register(i) => self.registers.values[i as usize],
        }
    }
}
