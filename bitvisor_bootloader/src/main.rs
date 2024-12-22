#![no_std]
#![no_main]
#![feature(c_variadic)]
///https://doc.rust-lang.org/beta/unstable-book/language-features/c-variadic.html

// Copyright (c) 2022 RIKEN
// Copyright (c) 2022 National Institute of Advanced Industrial Science and Technology (AIST)
// All rights reserved.
//
// This software is released under the MIT License.
// http://opensource.org/licenses/mit-license.php

/// x86_64 BitVisor bootloader written by rust.
/// This program call BitVisor kernel
/// - EfiHandle
/// - EfiSystemTable
/// - SystemInformation
///     - BitVisorBoot
///     - BitVisorDisconnectController
///     - AcpiTable
///     - DtbTable
pub mod uefi;
#[macro_use]
pub mod console;
mod bsdriver;
mod cpu;
mod elf;
mod info;

use bsdriver::load_bsdriver;
use core::{
    mem::MaybeUninit,
    num::NonZeroUsize,
    ptr::{null, null_mut},
    result,
};
use cpu::halt_loop;
use info::{
    AcpiTable, BitVisorBoot, BitVisorDisconnectController, DtbTable, UEFI_BITVISOR_BOOT_UUID,
    UEFI_BITVISOR_DEV_TREE_UUID, UEFI_BITVISOR_DISCONNECT_CONTROLLER_UUID,
};
use uefi::{
    boot_service::{self, EfiBootServices},
    file::{self, EfiFileProtocol},
    EfiConfigurationTable, EfiHandle, EfiStatus, EfiSystemTable, EFI_ACPI_20_TABLE_GUID,
    EFI_DTB_TABLE_GUID,
};

type KernelEntryPointFn = unsafe extern "efiapi" fn(EfiHandle, *mut EfiSystemTable, usize) -> i32;

static UPPER_LOAD_ADDR: usize = 0x4000_0000;
static ENTRY_BOOTSTRAP_CODE_SIZE: usize = 0x10000;
static ENTRY_MASK: usize = 0xFFFF;

static mut SYSTEM_TABLE_REF: *const EfiSystemTable = core::ptr::null();
static mut IMAGE_HANDLE_REF: EfiHandle = 0;
static mut BOOT_SERVICES: *mut EfiBootServices = core::ptr::null_mut();
static mut BITVISOR_PROTOCOL_REF: *const EfiFileProtocol = core::ptr::null_mut();

#[no_mangle]
extern "C" fn efi_main(image_handle: EfiHandle, system_table: *mut EfiSystemTable) -> EfiStatus {
    unsafe {
        unsafe { SYSTEM_TABLE_REF = system_table };
        unsafe { IMAGE_HANDLE_REF = image_handle };
        console::DEFAULT_CONSOLE.init((*system_table).console_output_protocol);
    }
    let b_s = unsafe { &mut *((*system_table).efi_boot_services) };
    unsafe {
        BOOT_SERVICES = b_s as *mut EfiBootServices;
    }

    const PATH: &str = "EFI\\BOOT\\bitvisor.elf";

    let root_protocol =
        file::EfiFileProtocol::open_root_dir(image_handle, b_s).expect("Failed to open root file.");
    let mut bitvisor_path_utf16: [u16; PATH.len() + 1] = [0; PATH.len() + 1];
    for (i, m) in PATH.encode_utf16().enumerate() {
        bitvisor_path_utf16[i] = m;
    }
    let bitvisor_protocol = file::EfiFileProtocol::open_file(root_protocol, &bitvisor_path_utf16)
        .expect("Failed to open bitvisor file");

    let mut bitvisor_protocol_ref: *const EfiFileProtocol = bitvisor_protocol;
    unsafe { BITVISOR_PROTOCOL_REF = bitvisor_protocol_ref };

    /* Read ElfHeader */
    let mut elf_header: MaybeUninit<elf::Elf32Header> = MaybeUninit::uninit();
    const ELF32_HEADER_SIZE: usize = core::mem::size_of::<elf::Elf32Header>();
    let read_size = bitvisor_protocol
        .read(elf_header.as_mut_ptr() as *mut usize, ELF32_HEADER_SIZE)
        .expect("Failed to read Elf header");
    if read_size != core::mem::size_of_val(&elf_header) {
        panic!(
            "Expected {} bytes, but read {} bytes",
            ELF32_HEADER_SIZE, read_size
        );
    }
    println!("read {:#X}", read_size);

    bitvisor_protocol
        .seek(0x0000)
        .expect("Failed to seek for the program header");
    let elf_header = unsafe { elf_header.assume_init() };
    if !elf_header.check_elf_header() {
        panic!("Failed to load the bitvisor");
    }

    // ENTRY_BOOTSTRAP_CODE_SIZE までにある第2段階ブートローダーを読み込む
    let program_header_entries_size = ENTRY_BOOTSTRAP_CODE_SIZE;

    let physical_address = b_s
        .alloc_highest_memory(ENTRY_BOOTSTRAP_CODE_SIZE / 4096, UPPER_LOAD_ADDR)
        .expect("Failed to allocate memory");
    println!(
        "Allocate memory at {:#X} ~ {:#X}",
        physical_address,
        physical_address + program_header_entries_size * 4096
    );
    /*println!(
        "program_header_offset is {:#X}",
        elf_header.get_program_header_offset()
    );*/
    //0x34
    let mut position: u64 = 0;
    (bitvisor_protocol.get_position)(bitvisor_protocol, &mut position);
    println!("{:#X}", position);
    let read_size = bitvisor_protocol
        .read(physical_address as *mut usize, program_header_entries_size)
        .expect("Failed to read hypervisor");
    if read_size != program_header_entries_size {
        panic!(
            "Expected {} bytes, but read {} bytes",
            program_header_entries_size, read_size
        );
    }

    let entry_point = elf_header.get_entry_point();

    println!("Load hypervisor at {:#X}", entry_point);
    let boot_info = BitVisorBoot {
        bitvisor_boot_uuid: UEFI_BITVISOR_BOOT_UUID,
        bitvisor_memory_address: physical_address,
        bitvisor_size: ENTRY_BOOTSTRAP_CODE_SIZE,
        bitvisor_protocol: unsafe { BITVISOR_PROTOCOL_REF },
    };
    let bitvisor_disconnect_info = BitVisorDisconnectController {
        bitvisor_disconnect_controller_uuid: UEFI_BITVISOR_DISCONNECT_CONTROLLER_UUID,
        disconnect_controller: (b_s.disconnect_controller) as *const usize,
    };
    let acpi_table = AcpiTable {
        bitvisor_acpi_uuid: EFI_ACPI_20_TABLE_GUID,
        acpi_table_mod: acpi_table_mod as *const usize,
    };
    let dtb_address = detect_dtb(unsafe { &*system_table });
    let dtb_table = DtbTable {
        bitvisor_dtb_uuid: UEFI_BITVISOR_DEV_TREE_UUID,
        dtb_table_address: dtb_address,
    };

    let mut system_info_pointers: [*const usize; 5] = [core::ptr::null(); 5];
    system_info_pointers[0] = &boot_info as *const BitVisorBoot as *const usize;
    system_info_pointers[1] =
        &bitvisor_disconnect_info as *const BitVisorDisconnectController as *const usize;
    system_info_pointers[2] = &acpi_table as *const AcpiTable as *const usize;
    system_info_pointers[3] = match dtb_address {
        Some(_) => null(), //&dtb_table as *const DtbTable as *const usize,
        None => core::ptr::null(),
    };
    system_info_pointers[4] = core::ptr::null();

    let system_info_ptr =
        &system_info_pointers as *const [*const usize; 5] as *const usize as usize;

    let entry = (entry_point & ENTRY_MASK) + physical_address;
    println!("programmer_header_pool:{:#X}", physical_address);
    println!("bitvisor entry point:{:#X}", entry);

    /*let result: i32;
    unsafe {
        core::arch::asm!(
            "call {}",
            in(reg) entry,
            in("rcx") image_handle,
            in("rdx") system_table,
            in("r8") system_info_ptr,
            out("eax") result,
            clobber_abi("efiapi"),
        )
    }*/

    let entry_fn: KernelEntryPointFn =
        unsafe { core::mem::transmute::<usize, KernelEntryPointFn>(entry) };

    let result = unsafe { entry_fn(image_handle, system_table, system_info_ptr) };

    if result == 0 {
        println!("BootFailed!");
        return EfiStatus::EfiLoadError;
    }

    b_s.free_memory(physical_address, 0x10)
        .expect("failed to free memory");

    if let Err(e) = file::EfiFileProtocol::close_file(bitvisor_protocol) {
        println!("Failed to close BitVisor Protocol: {:?}", e);
    }
    if let Err(e) = file::EfiFileProtocol::close_file(root_protocol) {
        println!("Failed to close RootProtocol: {:?}", e);
    }
    EfiStatus::EfiSuccess
}

fn detect_dtb(system_table: &EfiSystemTable) -> Option<NonZeroUsize> {
    for i in 0..system_table.num_table_entries {
        let table = unsafe {
            &*((system_table.configuration_table
                + i * core::mem::size_of::<EfiConfigurationTable>())
                as *const EfiConfigurationTable)
        };
        pr_debug!("GUID: {:#X?}", table.vendor_guid);
        if table.vendor_guid == EFI_DTB_TABLE_GUID {
            pr_debug!("Detect DTB");
            return NonZeroUsize::new(table.vendor_table);
        }
    }
    None
}

extern "efiapi" fn acpi_table_mod(signature: u32, tableaddr: u64) -> EfiStatus {
    println!("fn acpi_table_mod is called by bitvisor.elf");
    let image_handle = unsafe { IMAGE_HANDLE_REF };
    let system_table = unsafe { SYSTEM_TABLE_REF } as *mut EfiSystemTable;
    let boot_service = unsafe { &*BOOT_SERVICES };
    if let Some(bsdriver) = load_bsdriver(image_handle, boot_service) {
        return (bsdriver.acpi_table_mod)(system_table, signature, tableaddr);
    } else {
        EfiStatus::EfiLoadError
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\n\nBoot Loader Panic: {}", info);
    cpu::halt_loop();
}
