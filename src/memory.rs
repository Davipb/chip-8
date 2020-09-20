use crate::core::{Address, Error, ResultChip8, VoidResultChip8, Word};
use std::fmt::{self, Display, Formatter, Write};
use std::fs::File;
use std::io::Read;

pub trait ReadMemory {
    fn get(&self, addr: Address) -> ResultChip8<Word>;
}

pub trait WriteMemory {
    fn set(&mut self, addr: Address, value: Word) -> VoidResultChip8;
}

pub trait ReadWriteMemory: ReadMemory + WriteMemory {}
impl<T> ReadWriteMemory for T where T: ReadMemory + WriteMemory {}

#[derive(Copy, Clone)]
pub struct MemoryRange {
    pub min: Address,
    pub max: Address,
}

impl MemoryRange {
    pub fn new(min: u16, max: u16) -> MemoryRange {
        MemoryRange {
            min: min.into(),
            max: max.into(),
        }
    }

    pub fn contains(&self, addr: Address) -> bool {
        self.min <= addr && self.max >= addr
    }

    pub fn overlaps(&self, other: &MemoryRange) -> bool {
        self.min <= other.max && other.min <= self.max
    }
}

impl Display for MemoryRange {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.min, self.max)
    }
}

struct ReadMemoryWrapper<T: ReadMemory>(T);

impl<T: ReadMemory> ReadMemory for ReadMemoryWrapper<T> {
    fn get(&self, addr: Address) -> ResultChip8<Word> {
        self.0.get(addr)
    }
}

impl<T: ReadMemory> WriteMemory for ReadMemoryWrapper<T> {
    fn set(&mut self, _: Address, _: Word) -> VoidResultChip8 {
        Err(Error::new("Write not supported for this memory".to_owned()))
    }
}

struct WriteMemoryWrapper<T: WriteMemory>(T);

impl<T: WriteMemory> ReadMemory for WriteMemoryWrapper<T> {
    fn get(&self, _: Address) -> ResultChip8<Word> {
        Err(Error::new("Read not supported for this memory".to_owned()))
    }
}

impl<T: WriteMemory> WriteMemory for WriteMemoryWrapper<T> {
    fn set(&mut self, addr: Address, value: Word) -> VoidResultChip8 {
        self.0.set(addr, value)
    }
}

struct MemoryMapperBank {
    name: String,
    range: MemoryRange,
    delegate: Box<dyn ReadWriteMemory>,
}

impl MemoryMapperBank {
    fn offset(&self, addr: Address) -> Address {
        addr - self.range.min
    }
}

impl Display for MemoryMapperBank {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.range)
    }
}

pub struct MemoryMapper {
    banks: Vec<MemoryMapperBank>,
}

impl MemoryMapper {
    pub fn new<'a>() -> MemoryMapper {
        MemoryMapper { banks: Vec::new() }
    }

    pub fn add(
        &mut self,
        bank: impl ReadWriteMemory + 'static,
        range: MemoryRange,
        name: String,
    ) -> VoidResultChip8 {
        let overlapping: Vec<&MemoryMapperBank> = self
            .banks
            .iter()
            .filter(|x| x.range.overlaps(&range))
            .collect();

        if !overlapping.is_empty() {
            let mut error = "Bank would overlap with ".to_owned();
            for overlapping_bank in overlapping {
                write!(error, "{}", overlapping_bank)?;
            }
            return Err(Error::new(error));
        }

        self.banks.push(MemoryMapperBank {
            name,
            range,
            delegate: Box::new(bank),
        });
        return Ok(());
    }

    pub fn add_read(
        &mut self,
        bank: impl ReadMemory + 'static,
        range: MemoryRange,
        name: String,
    ) -> VoidResultChip8 {
        self.add(ReadMemoryWrapper(bank), range, name)
    }

    pub fn add_write(
        &mut self,
        bank: impl WriteMemory + 'static,
        range: MemoryRange,
        name: String,
    ) -> VoidResultChip8 {
        self.add(WriteMemoryWrapper(bank), range, name)
    }
}

impl ReadMemory for MemoryMapper {
    fn get(&self, addr: Address) -> ResultChip8<Word> {
        let bank = self
            .banks
            .iter()
            .find(|x| x.range.contains(addr))
            .ok_or_else(|| Error::new(format!("No bank mapped to address {}", addr)))?;

        let addr_offset = bank.offset(addr);
        bank.delegate.get(addr_offset).map_err(|x| {
            x.chain(format!(
                "Unable to read address {} from bank {}",
                addr, bank
            ))
        })
    }
}

impl WriteMemory for MemoryMapper {
    fn set(&mut self, addr: Address, value: Word) -> VoidResultChip8 {
        let bank = self
            .banks
            .iter_mut()
            .find(|x| x.range.contains(addr))
            .ok_or_else(|| Error::new(format!("No bank mapped to address {}", addr)))?;
        let addr_offset = bank.offset(addr);
        bank.delegate.set(addr_offset, value).map_err(|x| {
            x.chain(format!(
                "Unable to write to address {} in bank {}",
                addr, bank
            ))
        })
    }
}

pub struct ByteArrayMemory(Vec<Word>);

impl ByteArrayMemory {
    pub fn new(data: Vec<Word>) -> ByteArrayMemory {
        ByteArrayMemory(data)
    }

    pub fn zero(size: usize) -> ByteArrayMemory {
        ByteArrayMemory(vec![Word::ZERO; size])
    }

    pub fn from_file(size: usize, path: &str) -> ResultChip8<ByteArrayMemory> {
        let mut file = File::open(path)?;

        let mut bytes = Vec::with_capacity(size);
        file.read_to_end(&mut bytes)?;
        bytes.resize_with(size, Default::default);

        let words = bytes.into_iter().map(Word::new).collect();
        Ok(ByteArrayMemory(words))
    }

    fn make_bounds_error(addr: Address) -> Error {
        Error::new(format!(
            "Address {} is outside the range of the byte array with length",
            addr
        ))
    }
}

impl ReadMemory for ByteArrayMemory {
    fn get(&self, addr: Address) -> ResultChip8<Word> {
        self.0
            .get(usize::from(addr))
            .map(Clone::clone)
            .ok_or_else(|| ByteArrayMemory::make_bounds_error(addr))
    }
}

impl WriteMemory for ByteArrayMemory {
    fn set(&mut self, addr: Address, value: Word) -> VoidResultChip8 {
        let x = self
            .0
            .get_mut(usize::from(addr))
            .ok_or_else(|| ByteArrayMemory::make_bounds_error(addr))?;
        *x = value;
        Ok(())
    }
}
