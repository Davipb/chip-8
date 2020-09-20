use crate::core::{Address, VoidResultChip8, Word};
use crate::memory::{ReadMemory, WriteMemory};
use crate::opcodes::ValueRegisterIndex;

pub struct Registers {
    pub values: [Word; 0x10],
    pub program_counter: Address,
    pub address: Address,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            values: [Word::ZERO; 0x10],
            program_counter: Address::new(0x0200u16),
            address: Address::ZERO,
        }
    }

    pub fn dump_values(
        &self,
        max_reg: ValueRegisterIndex,
        base_addr: Address,
        mem: &mut impl WriteMemory,
    ) -> VoidResultChip8 {
        for i in 0..=max_reg {
            mem.set(base_addr + i, self.values[i as usize])?;
        }
        Ok(())
    }

    pub fn load_values(
        &mut self,
        max_reg: ValueRegisterIndex,
        base_addr: Address,
        mem: &impl ReadMemory,
    ) -> VoidResultChip8 {
        for i in 0..=max_reg {
            self.values[i as usize] = mem.get(base_addr + i)?;
        }
        Ok(())
    }
}
