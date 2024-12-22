// Copyright (c) 2022 RIKEN
// All rights reserved.
//
// This software is released under the MIT License.
// http://opensource.org/licenses/mit-license.php

//!
//! Executable and Linkable Format for
//! Aarch64 ELF64
//! X86_64 ELF64
//! Intel 80386 ELF32
//!
//! Supported Version: 1

#[allow(dead_code)]

use crate::println;

const EI_NIDENT: usize = 16;
pub const ELF32_IDENTIFIER: [u8; EI_NIDENT] = [
    0x7f, 0x45, 0x4c, 0x46, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
pub const ELF64_IDENTIFIER: [u8; EI_NIDENT] = [
    0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const EM_AARCH64: Elf64Half = 183;
const EM_X86_64:Elf64Half = 62;
const EM_386: Elf32Half = 3; // Intel 80386
const PT_LOAD: Elf64Word = 1;
const ELF_VERSION: Elf64Word = 0x1;

type Elf32Addr = u32;
type Elf32Half = u16;
type Elf32Off = u32;
//type Elf32Sword = i32;
//type Elf32Sxword = i64;
type Elf32Word = u32;
type Elf32Xword = u64;
//type Elf32Section = u16;

type Elf64Addr = u64;
type Elf64Half = u16;
type Elf64Off = u64;
//type Elf64Sword = i32;
//type Elf64Sxword = i64;
type Elf64Word = u32;
type Elf64Xword = u64;
//type Elf32Section = u16;

#[repr(C)]
pub struct Elf32Header {
    e_ident: [u8; EI_NIDENT], /* Magic number and other info */
    e_type: Elf32Half,  /* Object file type */
    e_machine: Elf32Half,   /* Architecture */
    e_version: Elf32Word,   /* Object file version */
    e_entry: Elf32Addr, /* Entry point virtual address */
    e_phoff: Elf32Off,  /* Program header table file offset */
    e_shoff: Elf32Off,  /* Section header table file offset */
    e_flags: Elf32Word, /* Processor-specific flags */
    e_ehsize: Elf32Half,    /* ELF header size in bytes */
    e_phentsize: Elf32Half, /* Program header table entry size */
    e_phnum: Elf32Half, /* Program header table entry count */
    e_shentsize: Elf32Half, /* Section header table entry size */
    e_shnum: Elf32Half, /* Section header table entry count */
    e_shstrndx: Elf32Half,  /* Section header string table index */
}

#[repr(C)]
struct Elf32ProgramHeader {
    p_type: Elf32Word,  /* Segment type */
    p_offset: Elf32Off, /* Segment file offset */
    p_vaddr: Elf32Addr, /* Segment virtual address */
    p_paddr: Elf32Addr, /* Segment physical address */
    p_filesz: Elf32Word,    /* Segment size in file */
    p_memsz: Elf32Word, /* Segment size in memory */
    p_flags: Elf32Word, /* Segment flags */
    p_align: Elf32Word, /* Segment alignment */
}

#[repr(C)]
pub struct Elf64Header {
    e_ident: [u8; EI_NIDENT],
    e_type: Elf64Half,
    e_machine: Elf64Half,
    e_version: Elf64Word,
    e_entry: Elf64Addr,
    e_phoff: Elf64Off,
    e_shoff: Elf64Off,
    e_flags: Elf64Word,
    e_ehsize: Elf64Half,
    e_phentsize: Elf64Half,
    e_phnum: Elf64Half,
    e_shentsize: Elf64Half,
    e_shnum: Elf64Half,
    e_shstrndx: Elf64Half,
}

#[repr(C)]
struct Elf64ProgramHeader {
    p_type: Elf64Word,
    p_flags: Elf64Word,
    p_offset: Elf64Off,
    p_vaddr: Elf64Addr,
    p_paddr: Elf64Addr,
    p_filesz: Elf64Xword,
    p_memsz: Elf64Xword,
    p_align: Elf64Xword,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SegmentInfo {
    pub virtual_base_address: usize,
    pub physical_base_address: usize,
    pub file_offset: usize,
    pub memory_size: usize,
    pub file_size: usize,
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

impl Elf32Header {
    pub fn check_elf_header(&self) -> bool {
        if self.e_ident != ELF32_IDENTIFIER {
            println!("Invalid elf identifier: {:?}", self.e_ident);
            return false;
        }
        if self.e_machine != EM_386 {
            println!("Target machine is not matched: {}", self.e_machine);
            return false;
        }
        if self.e_version < ELF_VERSION {
            println!("Unsupported ELF version: {}", self.e_version);
            return false;
        }
        true
    }

    pub fn get_entry_point(&self) -> usize {
        self.e_entry as usize
    }

    pub fn get_num_of_program_header_entries(&self) -> usize {
        self.e_phnum as usize
    }

    pub fn get_program_header_offset(&self) -> usize {
        self.e_phoff as usize
    }

    pub fn get_program_header_entry_size(&self) -> usize {
        self.e_phentsize as usize
    }

    pub fn get_segment_info(
        &self,
        index: usize,
        program_header_base: usize,
    ) -> Option<SegmentInfo> {
        if self.get_num_of_program_header_entries() <= index {
            return None;
        }
        let program_header = unsafe {
            &*((program_header_base + index * (self.e_phentsize as usize))
                as *const Elf64ProgramHeader)
        };
        if program_header.p_type == PT_LOAD {
            Some(SegmentInfo {
                file_offset: program_header.p_offset as usize,
                virtual_base_address: program_header.p_vaddr as usize,
                physical_base_address: program_header.p_paddr as usize,
                memory_size: program_header.p_memsz as usize,
                file_size: program_header.p_filesz as usize,
                readable: (program_header.p_flags & 0x4) != 0,
                writable: (program_header.p_flags & 0x2) != 0,
                executable: (program_header.p_flags & 0x1) != 0,
            })
        } else {
            None
        }
    }
}

impl Elf64Header {
    pub fn check_elf_header(&self) -> bool {
        if self.e_ident != ELF64_IDENTIFIER {
            println!("Invalid elf identifier: {:?}", self.e_ident);
            return false;
        }
        if self.e_machine != EM_X86_64 && self.e_machine != EM_AARCH64 {
            println!("Target machine is not matched: {}", self.e_machine);
            return false;
        }
        if self.e_version < ELF_VERSION {
            println!("Unsupported ELF version: {}", self.e_version);
            return false;
        }
        true
    }

    pub fn get_entry_point(&self) -> usize {
        self.e_entry as usize
    }

    pub fn get_num_of_program_header_entries(&self) -> usize {
        self.e_phnum as usize
    }

    pub fn get_program_header_offset(&self) -> usize {
        self.e_phoff as usize
    }

    pub fn get_program_header_entry_size(&self) -> usize {
        self.e_phentsize as usize
    }

    pub fn get_segment_info(
        &self,
        index: usize,
        program_header_base: usize,
    ) -> Option<SegmentInfo> {
        if self.get_num_of_program_header_entries() <= index {
            return None;
        }
        let program_header = unsafe {
            &*((program_header_base + index * (self.e_phentsize as usize))
                as *const Elf64ProgramHeader)
        };
        if program_header.p_type == PT_LOAD {
            Some(SegmentInfo {
                file_offset: program_header.p_offset as usize,
                virtual_base_address: program_header.p_vaddr as usize,
                physical_base_address: program_header.p_paddr as usize,
                memory_size: program_header.p_memsz as usize,
                file_size: program_header.p_filesz as usize,
                readable: (program_header.p_flags & 0x4) != 0,
                writable: (program_header.p_flags & 0x2) != 0,
                executable: (program_header.p_flags & 0x1) != 0,
            })
        } else {
            None
        }
    }
}
