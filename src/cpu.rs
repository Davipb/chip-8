use crate::core::{Address, Error, VoidResultChip8, Word};
use crate::display::VideoMemory;
use crate::input::InputManager;
use crate::memory::{MemoryMapper, ReadMemory, WriteMemory};
use crate::opcodes::{Condition, Opcode, OpcodeParam};
use crate::registers::Registers;
use crate::timers::Timers;
use rand::random;

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
        CPU {
            registers: Registers::new(),
            timers: Timers::new(),
            memory: MemoryMapper::new(),
            stack: Vec::new(),
            vram: VideoMemory::new(),
            input: InputManager::new(),
        }
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

            // Misc
            Opcode::Nop => Ok(()),

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
