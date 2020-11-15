use crate::core::{Address, Error, ResultChip8, Word};
use std::cmp::PartialEq;
use std::fmt::{self, Display, Formatter};

pub type ValueRegisterIndex = u8;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Opcode {
    // Value Registers
    Assign {
        left_reg: ValueRegisterIndex,
        right: OpcodeParam,
        op: Operation,
    },
    Shift {
        reg: ValueRegisterIndex,
        right: bool,
    },
    Random {
        reg: ValueRegisterIndex,
        mask: Word,
    },

    // Address Register
    AssignAddress(Address),
    AddAddress(ValueRegisterIndex),
    GetCharacterAddress(ValueRegisterIndex),

    // Flow Control
    Return,
    Jump(Address),
    OffsetJump(Address),
    Call(Address),
    CallNative(Address),
    CondJump {
        left: OpcodeParam,
        right: OpcodeParam,
        cond: Condition,
    },

    // Graphics
    ClearScreen,
    Draw {
        x: ValueRegisterIndex,
        y: ValueRegisterIndex,
        height: u8,
    },

    // IO
    BlockOnKey(ValueRegisterIndex),
    CondKeyJump {
        reg: ValueRegisterIndex,
        cond: Condition,
    },

    // Timers
    GetDelayTimer(ValueRegisterIndex),
    SetTimer {
        reg: ValueRegisterIndex,
        timer: Timer,
    },

    // Misc
    Nop,
    WriteBCD(ValueRegisterIndex),
    DumpValueRegisters(ValueRegisterIndex),
    LoadValueRegisters(ValueRegisterIndex),
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum OpcodeParam {
    Immediate(Word),
    Register(ValueRegisterIndex),
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Condition {
    Equal,
    NotEqual,
}

impl Condition {
    pub fn evaluate<T: PartialEq>(&self, a: T, b: T) -> bool {
        match self {
            Condition::Equal => a == b,
            Condition::NotEqual => a != b,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Operation {
    None,
    Add,
    Sub,
    ReverseSub,
    Or,
    And,
    Xor,
}

impl Operation {
    pub fn evaluate(&self, lhs: Word, rhs: Word) -> (Word, Option<bool>) {
        match self {
            Operation::None => (rhs, None),
            Operation::Or => (rhs | lhs, None),
            Operation::And => (rhs & lhs, None),
            Operation::Xor => (rhs ^ lhs, None),
            Operation::Add => {
                let lhs: u8 = lhs.into();
                let (result, carry) = lhs.overflowing_add(rhs.into());
                (Word::new(result), Some(carry))
            }
            Operation::Sub => {
                let lhs: u8 = lhs.into();
                let (result, carry) = lhs.overflowing_sub(rhs.into());
                (Word::new(result), Some(!carry))
            }
            Operation::ReverseSub => {
                let rhs: u8 = rhs.into();
                let (result, carry) = rhs.overflowing_sub(lhs.into());
                (Word::new(result), Some(!carry))
            }
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Timer {
    Delay,
    Sound,
}

impl Opcode {
    pub fn decode_bytes(bytes: &[impl Into<u8> + Clone; 2]) -> ResultChip8<Opcode> {
        let value = u16::from_be_bytes([bytes[0].clone().into(), bytes[1].clone().into()]);
        Opcode::decode(value)
    }

    pub fn decode(value: u16) -> ResultChip8<Opcode> {
        if value == 0x0000 {
            return Ok(Opcode::Nop);
        }

        if value == 0x00E0 {
            return Ok(Opcode::ClearScreen);
        }

        if value == 0x00EE {
            return Ok(Opcode::Return);
        }

        let first_nibble = ((value & 0xF000) >> 12) as u8;
        if [0x0, 0x1, 0x2, 0xA, 0xB].contains(&first_nibble) {
            let addr = Address::new(value & 0x0FFF);
            return Ok(match first_nibble {
                0x0 => Opcode::CallNative(addr),
                0x1 => Opcode::Jump(addr),
                0x2 => Opcode::Call(addr),
                0xA => Opcode::AssignAddress(addr),
                0xB => Opcode::OffsetJump(addr),
                _ => panic!(),
            });
        }

        if [0x3, 0x4, 0x5, 0x9].contains(&first_nibble) {
            let reg = ((value & 0x0F00) >> 8) as u8;
            return Ok(Opcode::CondJump {
                left: OpcodeParam::Register(reg),
                right: match first_nibble {
                    3 | 4 => OpcodeParam::Immediate(((value & 0x00FF) as u8).into()),
                    _ => OpcodeParam::Register(((value & 0x00F0) >> 8) as u8),
                },
                cond: match first_nibble {
                    3 | 5 => Condition::Equal,
                    _ => Condition::NotEqual,
                },
            });
        }

        if [0x6, 0x7].contains(&first_nibble) {
            let reg = ((value & 0x0F00) >> 8) as u8;
            let immediate = (value & 0x00FF) as u8;
            return Ok(Opcode::Assign {
                left_reg: reg,
                right: OpcodeParam::Immediate(Word::new(immediate)),
                op: match first_nibble {
                    6 => Operation::None,
                    7 => Operation::Add,
                    _ => panic!(),
                },
            });
        }

        if first_nibble == 0x8 {
            let last_nibble = (value & 0x000F) as u8;

            let reg1 = ((value & 0x0F00) >> 8) as u8;
            let reg2 = ((value & 0x00F0) >> 4) as u8;

            if last_nibble == 0x6 || last_nibble == 0xE {
                return Ok(Opcode::Shift {
                    reg: reg1,
                    right: last_nibble == 0x6,
                });
            }

            if last_nibble > 7 {
                return Err(Error::new(format!(
                    "Last nibble invalid in opcode {:04X}",
                    value
                )));
            }

            return Ok(Opcode::Assign {
                left_reg: reg1,
                right: OpcodeParam::Register(reg2),
                op: match last_nibble {
                    0 => Operation::None,
                    1 => Operation::Or,
                    2 => Operation::And,
                    3 => Operation::Xor,
                    4 => Operation::Add,
                    5 => Operation::Sub,
                    7 => Operation::ReverseSub,
                    _ => panic!(),
                },
            });
        }

        if first_nibble == 0xC {
            return Ok(Opcode::Random {
                reg: ((value & 0x0F00) >> 8) as u8,
                mask: Word::new((value & 0x00FF) as u8),
            });
        }

        if first_nibble == 0xD {
            return Ok(Opcode::Draw {
                x: ((value & 0x0F00) >> 8) as u8,
                y: ((value & 0x00F0) >> 4) as u8,
                height: (value & 0x000F) as u8,
            });
        }

        if first_nibble == 0xE {
            let reg = ((value & 0x0F00) >> 8) as u8;
            let last_byte = (value & 0x00FF) as u8;

            if last_byte != 0x9E && last_byte != 0xA1 {
                return Err(Error::new(format!(
                    "Last byte invalid in opcode {:04X}",
                    value
                )));
            }

            return Ok(Opcode::CondKeyJump {
                reg,
                cond: match last_byte {
                    0x9E => Condition::Equal,
                    0xA1 => Condition::NotEqual,
                    _ => panic!(),
                },
            });
        }

        if first_nibble == 0xF {
            let reg = ((value & 0x0F00) >> 8) as u8;
            let last_byte = (value & 0x00FF) as u8;

            return match last_byte {
                0x07 => Ok(Opcode::GetDelayTimer(reg)),
                0x0A => Ok(Opcode::BlockOnKey(reg)),
                0x15 => Ok(Opcode::SetTimer {
                    reg,
                    timer: Timer::Delay,
                }),
                0x18 => Ok(Opcode::SetTimer {
                    reg,
                    timer: Timer::Sound,
                }),
                0x1E => Ok(Opcode::AddAddress(reg)),
                0x29 => Ok(Opcode::GetCharacterAddress(reg)),
                0x33 => Ok(Opcode::WriteBCD(reg)),
                0x55 => Ok(Opcode::DumpValueRegisters(reg)),
                0x65 => Ok(Opcode::LoadValueRegisters(reg)),

                _ => Err(Error::new(format!(
                    "Last byte invalid in opcode {:04X}",
                    value
                ))),
            };
        }

        return Err(Error::new(format!("Invalid opcode {:04X}", value)));
    }
}

impl Display for Opcode {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            // Value Registers
            Opcode::Assign {
                left_reg,
                right,
                op,
            } => match op {
                Operation::ReverseSub => write!(
                    fmt,
                    "{0} = {1} - {0}",
                    OpcodeParam::Register(*left_reg),
                    right
                ),
                _ => write!(
                    fmt,
                    "{} {}= {}",
                    OpcodeParam::Register(*left_reg),
                    op,
                    right
                ),
            },
            Opcode::Shift { reg, right } => write!(
                fmt,
                "{} {}= 1",
                OpcodeParam::Register(*reg),
                if *right { ">>" } else { "<<" }
            ),
            Opcode::Random { reg, mask } => {
                write!(fmt, "{} = rand() & {}", OpcodeParam::Register(*reg), mask)
            }

            // Address Register
            Opcode::AssignAddress(x) => write!(fmt, "I = {}", x),
            Opcode::AddAddress(x) => write!(fmt, "I += {}", OpcodeParam::Register(*x)),
            Opcode::GetCharacterAddress(x) => {
                write!(fmt, "I = char[{}]", OpcodeParam::Register(*x))
            }

            // Flow Control
            Opcode::Return => write!(fmt, "return"),
            Opcode::Jump(x) => write!(fmt, "goto {}", x),
            Opcode::OffsetJump(x) => write!(fmt, "goto {} + {}", x, OpcodeParam::Register(0)),
            Opcode::Call(x) => write!(fmt, "{}()", x),
            Opcode::CallNative(x) => write!(fmt, "Native {}()", x),
            Opcode::CondJump { left, right, cond } => {
                write!(fmt, "if {} {} {} {{ skip }}", left, cond, right)
            }

            // Graphics
            Opcode::ClearScreen => write!(fmt, "clear()"),
            Opcode::Draw { x, y, height } => write!(
                fmt,
                "draw *I at ({}; {}) size 8x{}",
                OpcodeParam::Register(*x),
                OpcodeParam::Register(*y),
                height + 1
            ),

            // IO
            Opcode::BlockOnKey(x) => write!(fmt, "{} = wait_for_key()", OpcodeParam::Register(*x)),
            Opcode::CondKeyJump { reg, cond } => write!(
                fmt,
                "if {} {} key {{ skip }}",
                OpcodeParam::Register(*reg),
                cond
            ),

            // Timers
            Opcode::GetDelayTimer(x) => {
                write!(fmt, "{} = {}", OpcodeParam::Register(*x), Timer::Delay)
            }
            Opcode::SetTimer { reg, timer } => {
                write!(fmt, "{} = {}", timer, OpcodeParam::Register(*reg))
            }

            // Misc
            Opcode::Nop => write!(fmt, "nop"),
            Opcode::WriteBCD(x) => write!(fmt, "*I = BCD({})", OpcodeParam::Register(*x)),
            Opcode::DumpValueRegisters(x) => write!(
                fmt,
                "*I = [{}..={}]",
                OpcodeParam::Register(0),
                OpcodeParam::Register(*x)
            ),
            Opcode::LoadValueRegisters(x) => write!(
                fmt,
                "[{}..={}] = *I",
                OpcodeParam::Register(0),
                OpcodeParam::Register(*x)
            ),
        }
    }
}

impl Display for OpcodeParam {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            OpcodeParam::Register(x) => write!(fmt, "V{:X}", x),
            OpcodeParam::Immediate(x) => Display::fmt(&x, fmt),
        }
    }
}

impl Display for Condition {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Condition::Equal => write!(fmt, "=="),
            Condition::NotEqual => write!(fmt, "!="),
        }
    }
}

impl Display for Operation {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Operation::None => Ok(()),
            Operation::Add => write!(fmt, "+"),
            Operation::Sub => write!(fmt, "-"),
            Operation::ReverseSub => write!(fmt, "(Reverse Sub)"),
            Operation::Or => write!(fmt, "|"),
            Operation::And => write!(fmt, "&"),
            Operation::Xor => write!(fmt, "^"),
        }
    }
}

impl Display for Timer {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Timer::Delay => write!(fmt, "delay_timer"),
            Timer::Sound => write!(fmt, "sound_timer"),
        }
    }
}
