use crate::uefi::{
    boot_service::EfiBootServices, file::EfiFileProtocol, EfiHandle, EfiStatus, Guid,
};
use core::{num::NonZeroUsize, ptr::NonNull};

#[allow(dead_code)]

pub const UEFI_BITVISOR_BOOT_UUID: Guid = Guid {
    d1: 0x4CF80319,
    d2: 0xA870,
    d3: 0x44C5,
    d4: [0x9A, 0x87, 0x60, 0xE5, 0x86, 0xE7, 0x9D, 0x0F],
};

pub const UEFI_BITVISOR_PASS_AUTH_UUID: Guid = Guid {
    d1: 0xE0970CB4,
    d2: 0xDF2E,
    d3: 0x44D1,
    d4: [0xB1, 0xA9, 0x63, 0x3C, 0xCD, 0xE3, 0xA2, 0xC6],
};

pub const UEFI_BITVISOR_CPU_TYPE_UUID: Guid = Guid {
    d1: 0x0992D209,
    d2: 0x72B0,
    d3: 0x491F,
    d4: [0xA2, 0xBC, 0xC2, 0x3D, 0x39, 0xF9, 0x78, 0x76],
};

pub const UEFI_BITVISOR_DISCONNECT_CONTROLLER_UUID: Guid = Guid {
    d1: 0x7B50E9DA,
    d2: 0xE7AD,
    d3: 0x4E89,
    d4: [0x9D, 0x93, 0xC3, 0x54, 0x69, 0x33, 0x46, 0x84],
};

pub const UEFI_BITVISOR_ACPI_TABLE_MOD_UUID: Guid = Guid {
    d1: 0x79FF5F54,
    d2: 0x5392,
    d3: 0x42E0,
    d4: [0xA8, 0x92, 0x3B, 0x13, 0xB4, 0x71, 0xD2, 0x88],
};

pub const UEFI_BITVISOR_DEV_TREE_UUID: Guid = Guid {
    d1: 0xD18652BC,
    d2: 0xD274,
    d3: 0x4E45,
    d4: [0xBF, 0xCC, 0x02, 0xDB, 0x91, 0xAE, 0xD8, 0x10],
};

pub const EFI_BLOCK_IO_CRYPTO_PROTOCOL_GUID: Guid = Guid {
    d1: 0xa00490ba,
    d2: 0x3f1a,
    d3: 0x4b4c,
    d4: [0xab, 0x90, 0x4f, 0xa9, 0x97, 0x26, 0xa1, 0xe8],
};

#[repr(C)]
pub struct BitVisorBoot {
    pub bitvisor_boot_uuid: Guid,
    pub bitvisor_memory_address: usize,
    pub bitvisor_size: usize,
    pub bitvisor_protocol: *const EfiFileProtocol,
}

// https://uefi.org/specs/UEFI/2.10/07_Services_Boot_Services.html#efi-boot-services-disconnectcontroller
#[repr(C)]
pub struct BitVisorDisconnectController {
    pub bitvisor_disconnect_controller_uuid: Guid,
    pub disconnect_controller: *const usize,
}

#[repr(C)]
pub struct AcpiTable {
    pub bitvisor_acpi_uuid: Guid,
    pub acpi_table_mod: *const usize,
}

#[repr(C)]
pub struct DtbTable {
    pub bitvisor_dtb_uuid: Guid,
    pub dtb_table_address: Option<NonZeroUsize>,
}
