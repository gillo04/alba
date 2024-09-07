#![allow(unused)]

use core::ffi::c_void;

// Calls exit_boot_services
pub fn exit_boot_services(
    system_table: *const SystemTable,
    image_handle: *const c_void,
    memory_map_key: usize,
) -> Result<(), Status> {
    let status = unsafe {
        ((*(*system_table).boot_services).exit_boot_services)(image_handle, memory_map_key)
    };

    match status {
        Status::SUCCESS => Ok(()),
        _ => Err(status),
    }
}

#[repr(C)]
pub struct SystemTable {
    pub hdr: TableHeader,
    pub firmware_vendor: *const u16,
    pub firmware_revision: u32,
    pub console_inhandle: *const c_void,
    pub con_in: *const c_void,
    pub console_out_handle: *const c_void,
    pub con_out: *mut SimpleTextOutputProtocol,
    pub standard_error_handle: *const c_void,
    pub std_err: *const c_void,
    pub runtime_services: *const RuntimeServices,
    pub boot_services: *const BootServices,
    pub number_of_table_entries: u64,
    pub configuration_table: *const ConfigurationTable,
}

#[repr(C)]
pub struct ConfigurationTable {
    pub vendor_guid: Guid,
    pub vendor_table: *const c_void,
}

#[repr(C)]
pub struct TableHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub crc32: u32,
    pub reserved: u32,
}

#[repr(C)]
pub struct SimpleTextOutputProtocol {
    pub reset: *const c_void,
    pub output_string: extern "efiapi" fn(*mut SimpleTextOutputProtocol, *const u16),
    pub test_string: *const c_void,
    pub query_mode: *const c_void,
    pub set_mode: *const c_void,
    pub set_attribute: *const c_void,
    pub clear_screen: extern "efiapi" fn(*mut SimpleTextOutputProtocol) -> Status,
    pub set_cursorposition: *const c_void,
    pub enable_cursor: *const c_void,
    pub mode: *const c_void,
}

#[repr(C)]
pub struct BootServices {
    pub hdr: TableHeader,

    pub raise_tpl: *const c_void,
    pub restore_tpl: *const c_void,

    pub allocate_pages: *const c_void,
    pub free_pages: *const c_void,
    pub get_memory_map: extern "efiapi" fn(
        *mut usize,
        *mut MemoryDescriptor,
        *mut usize,
        *mut usize,
        *mut u32,
    ) -> Status,
    pub allocate_pool: *const c_void,
    pub free_pool: *const c_void,

    pub create_event: *const c_void,
    pub set_timer: *const c_void,
    pub wait_for_event: *const c_void,
    pub signal_event: *const c_void,
    pub close_event: *const c_void,
    pub check_event: *const c_void,

    pub install_protocol_interface: *const c_void,
    pub reinstall_protocol_interface: *const c_void,
    pub uninstall_protocol_interface: *const c_void,
    pub handle_protocol: *const c_void,
    pub reserved: *const c_void,
    pub register_protocol_notify: *const c_void,
    pub locate_handle: *const c_void,
    pub locate_device_path: *const c_void,
    pub install_configuration_table: *const c_void,

    pub load_image: *const c_void,
    pub start_image: *const c_void,
    pub exit: *const c_void,
    pub unload_image: *const c_void,
    pub exit_boot_services: extern "efiapi" fn(*const c_void, usize) -> Status,

    pub get_next_monotonic_count: *const c_void,
    pub stall: *const c_void,
    pub set_watchdog_timer: *const c_void,

    pub connect_controller: *const c_void,
    pub disconnect_controller: *const c_void,

    pub open_protocol: *const c_void,
    pub close_protocol: *const c_void,
    pub open_protocol_information: *const c_void,

    pub protocols_per_handle: *const c_void,
    pub locate_handle_buffer: *const c_void,
    pub locate_protocol: extern "efiapi" fn(
        protocol: *const Guid,
        registration: *const c_void,
        interface: *mut *const c_void,
    ) -> Status,
    pub install_multiple_protocol_interfaces: *const c_void,
    pub uninstall_multiple_protocol_interfaces: *const c_void,

    pub calculate_crc32: *const c_void,

    pub copy_mem: *const c_void,
    pub set_mem: *const c_void,
    pub create_event_ex: *const c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemoryDescriptor {
    pub t: MemoryType,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}

impl MemoryDescriptor {
    pub const fn new() -> MemoryDescriptor {
        MemoryDescriptor {
            t: MemoryType::ReservedMemoryType,
            physical_start: 0,
            virtual_start: 0,
            number_of_pages: 0,
            attribute: 0,
        }
    }
}

#[repr(C)]
pub struct LoadFileProtocol {
    load_file: extern "efiapi" fn(
        this: *const LoadFileProtocol,
        file_path: *const c_void,
        boot_policy: bool,
        buffer_size: usize,
        buffer: *const c_void,
    ) -> Status,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MemoryType {
    ReservedMemoryType = 0,
    LoaderCode,
    LoaderData,
    BootServicesCode,
    BootServicesData,
    RuntimeServicesCode,
    RuntimeServicesData,
    ConventionalMemory,
    UnusableMemory,
    ACPIReclaimMemory,
    ACPIMemoryNVS,
    MemoryMappedIO,
    MemoryMappedIOPortSpace,
    PalCode,
    PersistentMemory,
    UnacceptedMemoryType,
    MaxMemoryType,
}

#[repr(u64)]
pub enum MemoryAttribute {
    UC = 0x0000000000000001,
    WC = 0x0000000000000002,
    WT = 0x0000000000000004,
    WB = 0x0000000000000008,
    UCE = 0x0000000000000010,
    WP = 0x0000000000001000,
    RP = 0x0000000000002000,
    XP = 0x0000000000004000,
    NV = 0x0000000000008000,
    MoreReliable = 0x0000000000010000,
    RO = 0x0000000000020000,
    SP = 0x0000000000040000,
    CpuCrypto = 0x0000000000080000,
    Runtime = 0x8000000000000000,
    IsaValid = 0x4000000000000000,
    IsaMask = 0x0FFFF00000000000,
}

#[derive(PartialEq)]
#[repr(C)]
pub struct Guid {
    time_low: u32,
    time_mid: u16,
    time_high_and_version: u16,
    clock_seq_high_and_reverse: u8,
    clock_seq_low: u8,
    node: [u8; 6],
}

impl Guid {
    pub const fn new(
        time_low: u32,
        time_mid: u16,
        time_high_and_version: u16,
        clock_seq_high_and_reverse: u8,
        clock_seq_low: u8,
        node1: u8,
        node2: u8,
        node3: u8,
        node4: u8,
        node5: u8,
        node6: u8,
    ) -> Guid {
        Guid {
            time_low,
            time_mid,
            time_high_and_version,
            clock_seq_high_and_reverse,
            clock_seq_low,
            node: [node1, node2, node3, node4, node5, node6],
        }
    }
}

pub const GOP_GUID: Guid = Guid::new(
    0x9042a9de, 0x23dc, 0x4a38, 0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a,
);

#[repr(C)]
pub struct GraphicsOutputProtocol {
    pub query_mode: *const c_void,
    pub set_mode: *const c_void,
    pub blt: *const c_void,
    pub mode: *const GraphicsOutputProtocolMode,
}

#[repr(C)]
pub struct GraphicsOutputProtocolMode {
    pub max_mode: u32,
    pub mode: u32,
    pub info: *const GraphicsOutputProtocolInformation,
    pub size_of_info: usize,
    pub frame_buffer_base: *const c_void,
    pub frame_buffer_size: usize,
}

#[repr(C)]
pub struct GraphicsOutputProtocolInformation {
    pub version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixel_format: u8,
    pub pixel_information: [u32; 4],
    pub pixels_per_scan_line: u32,
}

#[repr(C)]
pub struct RuntimeServices {
    pub hdr: TableHeader,
    pub get_time: *const c_void,
    pub set_time: *const c_void,
    pub get_wakeup_time: *const c_void,
    pub set_wakeup_time: *const c_void,
    pub set_virtual_address_map: extern "efiapi" fn(
        memory_map_size: usize,
        descriptor_size: usize,
        descriptor_version: u32,
        virtual_map: *const MemoryDescriptor,
    ) -> Status,
    pub convert_pointer: *const c_void,
    pub get_variable: *const c_void,
    pub get_next_variable_name: *const c_void,
    pub set_variable: *const c_void,
    pub get_next_high_monotonic_count: *const c_void,
    pub reset_system: extern "efiapi" fn(
        reset_type: u32,
        reset_status: Status,
        data_size: usize,
        reset_data: *const c_void,
    ) -> !,
    pub update_capsule: *const c_void,
    pub query_capsule_capabilities: *const c_void,
    pub query_variable_info: *const c_void,
}

// Taken from uefi_raw
const ERROR_BIT: usize = 1 << (core::mem::size_of::<usize>() * 8 - 1);

#[allow(warnings)]
#[repr(usize)]
#[derive(PartialEq, Debug)]
pub enum Status {
    /// The operation completed successfully.
    SUCCESS = 0,

    /// The string contained characters that could not be rendered and were skipped.
    WARN_UNKNOWN_GLYPH = 1,
    /// The handle was closed, but the file was not deleted.
    WARN_DELETE_FAILURE = 2,
    /// The handle was closed, but the data to the file was not flushed properly.
    WARN_WRITE_FAILURE = 3,
    /// The resulting buffer was too small, and the data was truncated.
    WARN_BUFFER_TOO_SMALL = 4,
    /// The data has not been updated within the timeframe set by local policy.
    WARN_STALE_DATA = 5,
    /// The resulting buffer contains UEFI-compliant file system.
    WARN_FILE_SYSTEM = 6,
    /// The operation will be processed across a system reset.
    WARN_RESET_REQUIRED = 7,

    /// The image failed to load.
    LOAD_ERROR = ERROR_BIT | 1,
    /// A parameter was incorrect.
    INVALID_PARAMETER = ERROR_BIT | 2,
    /// The operation is not supported.
    UNSUPPORTED = ERROR_BIT | 3,
    /// The buffer was not the proper size for the request.
    BAD_BUFFER_SIZE = ERROR_BIT | 4,
    /// The buffer is not large enough to hold the requested data.
    /// The required buffer size is returned in the appropriate parameter.
    BUFFER_TOO_SMALL = ERROR_BIT | 5,
    /// There is no data pending upon return.
    NOT_READY = ERROR_BIT | 6,
    /// The physical device reported an error while attempting the operation.
    DEVICE_ERROR = ERROR_BIT | 7,
    /// The device cannot be written to.
    WRITE_PROTECTED = ERROR_BIT | 8,
    /// A resource has run out.
    OUT_OF_RESOURCES = ERROR_BIT | 9,
    /// An inconstency was detected on the file system.
    VOLUME_CORRUPTED = ERROR_BIT | 10,
    /// There is no more space on the file system.
    VOLUME_FULL = ERROR_BIT | 11,
    /// The device does not contain any medium to perform the operation.
    NO_MEDIA = ERROR_BIT | 12,
    /// The medium in the device has changed since the last access.
    MEDIA_CHANGED = ERROR_BIT | 13,
    /// The item was not found.
    NOT_FOUND = ERROR_BIT | 14,
    /// Access was denied.
    ACCESS_DENIED = ERROR_BIT | 15,
    /// The server was not found or did not respond to the request.
    NO_RESPONSE = ERROR_BIT | 16,
    /// A mapping to a device does not exist.
    NO_MAPPING = ERROR_BIT | 17,
    /// The timeout time expired.
    TIMEOUT = ERROR_BIT | 18,
    /// The protocol has not been started.
    NOT_STARTED = ERROR_BIT | 19,
    /// The protocol has already been started.
    ALREADY_STARTED = ERROR_BIT | 20,
    /// The operation was aborted.
    ABORTED = ERROR_BIT | 21,
    /// An ICMP error occurred during the network operation.
    ICMP_ERROR = ERROR_BIT | 22,
    /// A TFTP error occurred during the network operation.
    TFTP_ERROR = ERROR_BIT | 23,
    /// A protocol error occurred during the network operation.
    PROTOCOL_ERROR = ERROR_BIT | 24,
    /// The function encountered an internal version that was
    /// incompatible with a version requested by the caller.
    INCOMPATIBLE_VERSION = ERROR_BIT | 25,
    /// The function was not performed due to a security violation.
    SECURITY_VIOLATION = ERROR_BIT | 26,
    /// A CRC error was detected.
    CRC_ERROR = ERROR_BIT | 27,
    /// Beginning or end of media was reached
    END_OF_MEDIA = ERROR_BIT | 28,
    /// The end of the file was reached.
    END_OF_FILE = ERROR_BIT | 31,
    /// The language specified was invalid.
    INVALID_LANGUAGE = ERROR_BIT | 32,
    /// The security status of the data is unknown or compromised and
    /// the data must be updated or replaced to restore a valid security status.
    COMPROMISED_DATA = ERROR_BIT | 33,
    /// There is an address conflict address allocation
    IP_ADDRESS_CONFLICT = ERROR_BIT | 34,
    /// A HTTP error occurred during the network operation.
    HTTP_ERROR = ERROR_BIT | 35,
}
