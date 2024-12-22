use crate::uefi::{
    boot_service::{DevicePathProtocol, EfiBootServices},
    loaded_image::{EfiLoadedImageProtocol, EFI_LOADED_IMAGE_PROTOCOL_GUID},
    EfiHandle, EfiStatus, EfiSystemTable,
};
use core::{
    include_bytes,
    ptr::null_mut,
};

static mut ALREADY_LOADED: bool = false;
static mut DRIVER_DATA: Option<&BootServiceDriver> = None;
static BSDRIVER_BIN: &[u8] = include_bytes!("../../build/bsdriver.efi"); //TODO

/// load boot service driver
/// https://github.com/matsu/bitvisor/commit/d62ffe23fe23a4beed69314784b35927bccab847
#[repr(C)]
pub struct BootServiceDriver {
    pub acpi_table_mod: extern "C" fn(
        system_table: *mut EfiSystemTable,
        signature: u32,
        table_addr: u64,
    ) -> EfiStatus,
}

pub fn load_bsdriver(
    image_handle: EfiHandle,
    boot_service: &EfiBootServices,
) -> Option<&'static BootServiceDriver> {
    if unsafe { ALREADY_LOADED } {
        return unsafe { DRIVER_DATA };
    }
    let boot_service_driver: *mut BootServiceDriver = null_mut();
    let mut handle = 0;
    let interface = null_mut();

    let device_path = DevicePathProtocol {
        major_type: 0,
        sub_type: unsafe { *BSDRIVER_BIN.as_ptr() },
        length: (BSDRIVER_BIN.len() as u16).to_le_bytes(),
    };
    if EfiStatus::EfiSuccess
        != (boot_service.load_image)(false as u8, image_handle, &device_path, &mut handle)
    {
        unsafe { ALREADY_LOADED = true };
        return None;
    }
    if EfiStatus::EfiSuccess
        != (boot_service.open_protocol)(
            handle,
            &EFI_LOADED_IMAGE_PROTOCOL_GUID,
            interface,
            image_handle,
            0,
            0x00000002, //EFI_OPEN_PROTOCOL_GET_PROTOCOL
        )
    {
        unsafe { ALREADY_LOADED = true };
        return None;
    }

    let loaded_image: &mut EfiLoadedImageProtocol =
        unsafe { &mut *(interface as *mut EfiLoadedImageProtocol) };
    loaded_image.load_option_size = size_of::<BootServiceDriver>() as u32; // usize to u32
    loaded_image.load_options = boot_service_driver as usize;

    if EfiStatus::EfiSuccess
        != (boot_service.close_protocol)(handle, &EFI_LOADED_IMAGE_PROTOCOL_GUID, image_handle, 0)
    {
        unsafe { ALREADY_LOADED = true };
        return None;
    }

    if EfiStatus::EfiSuccess != (boot_service.start_image)(handle, null_mut() as *mut usize, 0) {
        unsafe { ALREADY_LOADED = true };
        return None;
    }

    if boot_service_driver == null_mut() {
        unsafe { ALREADY_LOADED = true };
        return None;
    }
    unsafe { ALREADY_LOADED = true };
    unsafe { DRIVER_DATA = Some(&*boot_service_driver) }
    return unsafe { DRIVER_DATA };
}
