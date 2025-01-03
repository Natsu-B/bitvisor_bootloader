// Copyright (c) 2022 RIKEN
// Copyright (c) 2022 National Institute of Advanced Industrial Science and Technology (AIST)
// All rights reserved.
//
// This software is released under the MIT License.
// http://opensource.org/licenses/mit-license.php

//!
//! UEFI Boot Services
//!

mod memory_service;

pub use memory_service::*;

use super::{EfiHandle, EfiStatus, EfiTableHeader, Guid};

#[repr(C)]
pub struct EfiBootServices {
    efi_table_header: EfiTableHeader,
    raise_tpl: usize,
    restore_tpl: usize,
    allocate_pages: extern "efiapi" fn(
        allocate_type: EfiAllocateType,
        memory_type: EfiMemoryType,
        pages: usize,
        memory: *mut usize,
    ) -> EfiStatus,
    free_pages: extern "efiapi" fn(
        memory: usize,
        pages: usize,
    ) -> EfiStatus,
    get_memory_map: extern "efiapi" fn(
        memory_map_size: *mut usize,
        memory_map: *mut EfiMemoryDescriptor,
        map_key: *mut usize,
        descriptor_size: *mut usize,
        descriptor_version: *mut u32,
    ) -> EfiStatus,
    _allocate_pool:
        extern "efiapi" fn(pool_type: EfiMemoryType, size: usize, memory: *mut usize) -> EfiStatus,
    _free_pool: extern "efiapi" fn(memory: usize) -> EfiStatus,
    create_event: usize,
    set_timer: usize,
    wait_for_event: usize,
    signal_event: usize,
    close_event: usize,
    check_event: usize,
    install_protocol_interface: usize,
    reinstall_protocol_interface: usize,
    uninstall_protocol_interface: usize,
    handle_protocol: usize,
    reserved: usize,
    register_protocol_notify: usize,
    pub locate_handle: extern "efiapi" fn(
            efi_locate_search_type: i32,
            protocol: *const Guid,
            search_key: *const usize,
            buffer_size: *mut usize,
            buffer: *mut EfiHandle
    ) -> EfiStatus,
    locate_device_path: usize,
    install_configuration_table: usize,
    pub load_image: extern "efiapi" fn(
        boot_policy: u8,
        parent_image_handle: EfiHandle,
        device_path: *const DevicePathProtocol,
        out_handle: *mut EfiHandle,
    ) -> EfiStatus,
    pub start_image:
        extern "efiapi" fn(handle: EfiHandle, data_size: *mut usize, data: usize) -> EfiStatus,
    pub exit: extern "efiapi" fn(
        image_handler: EfiHandle,
        exit_status: EfiStatus,
        exit_data_size: usize,
        exit_data: *const u16,
    ) -> EfiStatus,
    unload_image: usize,
    pub exit_boot_services:
        extern "efiapi" fn(image_handler: EfiHandle, map_key: usize) -> EfiStatus,
    get_next_monotonic_count: usize,
    stall: usize,
    set_watchdog_timer: usize,
    connect_controller: usize,
    pub disconnect_controller: extern "efiapi" fn(
        handle: EfiHandle,
        driver_image: EfiHandle,
        child: EfiHandle,
    ) -> EfiStatus,
    pub open_protocol: extern "efiapi" fn(
        handle: EfiHandle,
        protocol: *const Guid,
        interface: *mut *const usize,
        agent_handle: EfiHandle,
        controller_handle: EfiHandle,
        attributes: u32,
    ) -> EfiStatus,
    pub close_protocol: extern "efiapi" fn(
        handle: EfiHandle,
        protocol: *const Guid,
        agent_handle: EfiHandle,
        controller_handle: EfiHandle,
    ) -> EfiStatus,
    open_protocol_information: usize,
    protocols_per_handle: usize,
    locate_handle_buffer: usize,
    pub locate_protocol: extern "efiapi" fn(
        protocol: *const Guid,
        registration: *const usize,
        interface: *mut *const usize,
    ) -> EfiStatus,
    install_multiple_protocol_interfaces: usize,
    /// Warning: It will work correctly only for '*-unknown-uefi'
    pub uninstall_multiple_protocol_interfaces: unsafe extern "C" fn(handle: EfiHandle, ...)-> EfiStatus,
    calculate_crc32: usize,
    copy_mem: extern "efiapi" fn(destination: usize, source: usize, length: usize),
    set_mem: extern "efiapi" fn(buffer: usize, size: usize, value: u8),
    create_event_ex: usize,
}

pub const EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL: u32 = 0x00000001;
pub const EFI_OPEN_PROTOCOL_GET_PROTOCOL: u32 = 0x00000002;
#[allow(dead_code)]
pub const EFI_OPEN_PROTOCOL_TEST_PROTOCOL: u32 = 0x00000004;
#[allow(dead_code)]
pub const EFI_OPEN_PROTOCOL_BY_CHILD_CONTROLLER: u32 = 0x00000008;
#[allow(dead_code)]
pub const EFI_OPEN_PROTOCOL_BY_DRIVER: u32 = 0x00000010;
#[allow(dead_code)]
pub const EFI_OPEN_PROTOCOL_EXCLUSIVE: u32 = 0x00000020;

#[repr(C)]
pub struct DevicePathProtocol {
    pub major_type: u8,
    pub sub_type: u8,
    pub length: [u8; 2],
}