use scroll::Pread;
use std::ops::Range;

use crate::errors::{Error, Result};

const RAM_BUNDLE_MAGIC: u32 = 0xFB0BD1E5;

#[derive(Debug, Pread)]
#[repr(C, packed)]
struct RamBundleHeader {
    magic: u32,
    module_count: u32,
    startup_code_size: u32,
}

#[derive(Debug, Pread)]
#[repr(C, packed)]
struct ModuleEntry {
    offset: u32,
    length: u32,
}

pub struct RamBundleModule<'a> {
    id: usize,
    data: &'a [u8],
}

pub struct RamBundleModuleIter<'a, 'b> {
    range: Range<usize>,
    ram_bundle: &'b RamBundle<'a>,
}

impl<'a> Iterator for RamBundleModuleIter<'a, '_> {
    type Item = Option<RamBundleModule<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.range
            .next()
            .map(|id| self.ram_bundle.get_module(id).unwrap())
    }
}

pub struct RamBundle<'a> {
    bytes: &'a [u8],
    module_count: usize,
    startup_code_size: usize,
    startup_code_offset: usize,
}

impl ModuleEntry {
    pub fn is_empty(&self) -> bool {
        self.offset == 0 && self.length == 0
    }
}

impl<'a> RamBundle<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self> {
        let header = bytes.pread_with::<RamBundleHeader>(0, scroll::LE)?;

        if header.magic != RAM_BUNDLE_MAGIC {
            return Err(Error::InvalidRamBundleMagic);
        }

        let module_count = header.module_count as usize;
        let startup_code_offset = std::mem::size_of::<RamBundleHeader>()
            + module_count * std::mem::size_of::<ModuleEntry>();
        Ok(RamBundle {
            bytes,
            module_count,
            startup_code_size: header.startup_code_size as usize,
            startup_code_offset,
        })
    }

    pub fn module_count(&self) -> usize {
        self.module_count
    }

    pub fn startup_code_size(&self) -> usize {
        self.startup_code_size
    }

    pub fn startup_code_offset(&self) -> usize {
        self.startup_code_offset
    }

    pub fn startup_code(&self) -> Result<&'a [u8]> {
        self.bytes
            .pread_with(self.startup_code_offset, self.startup_code_size)
            .map_err(Error::Scroll)
    }

    pub fn get_module(&self, id: usize) -> Result<Option<RamBundleModule<'a>>> {
        if id >= self.module_count {
            return Err(Error::InvalidRamBundleIndex);
        }

        let entry_offset =
            std::mem::size_of::<RamBundleHeader>() + id * std::mem::size_of::<ModuleEntry>();

        let module_entry = self
            .bytes
            .pread_with::<ModuleEntry>(entry_offset, scroll::LE)?;

        if module_entry.is_empty() {
            return Ok(None);
        }

        let module_global_offset = self.startup_code_offset + module_entry.offset as usize;
        let data = self
            .bytes
            .pread_with(module_global_offset, module_entry.length as usize)?;

        Ok(Some(RamBundleModule { id, data }))
    }

    pub fn iter_modules(&self) -> RamBundleModuleIter {
        RamBundleModuleIter {
            range: 0..self.module_count,
            ram_bundle: self,
        }
    }
}

impl<'a> RamBundleModule<'a> {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}
