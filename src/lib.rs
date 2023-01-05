#![allow(dead_code)]
use aaronia_rtsa_sys as sys;
use std::mem::MaybeUninit;

pub struct Config {
    inner: sys::AARTSAAPI_Config,
}

pub struct ConfigInfo {
    inner: sys::AARTSAAPI_ConfigInfo,
}

pub struct Device {
    inner: sys::AARTSAAPI_Device,
}

#[derive(Debug)]
pub struct DeviceInfo {
    inner: sys::AARTSAAPI_DeviceInfo,
}

impl DeviceInfo {
    fn new() -> Self {
        unsafe {
        let d = MaybeUninit::<sys::AARTSAAPI_DeviceInfo>::zeroed().assume_init();
            Self {
                inner: d,
            }
        }
        // Self {
        //     inner: sys::AARTSAAPI_DeviceInfo {
        //         cbsize: 0,
        //         serialNumber: [0; 120],
        //         ready: false,
        //         boost: false,
        //         superspeed: false,
        //         active: false,
        //     },
        // }
    }
}

impl Drop for DeviceInfo {
    fn drop(&mut self) {
        println!("dropping dev info");
    }
}

#[derive(Debug)]
pub struct Handle {
    inner: sys::AARTSAAPI_Handle,
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            res(sys::AARTSAAPI_Close(&mut self.inner)).expect("error dropping API handle");
        }
    }
}

pub struct Packet {
    inner: sys::AARTSAAPI_Packet,
}

#[derive(Debug, Clone)]
pub enum ConfigType {
    Other,
    Group,
    Blob,
    Number,
    Bool,
    Enum,
    String,
}

impl From<std::os::raw::c_uint> for ConfigType {
    fn from(value: std::os::raw::c_uint) -> Self {
        match value {
            1 => ConfigType::Group,
            2 => ConfigType::Blob,
            3 => ConfigType::Number,
            4 => ConfigType::Bool,
            5 => ConfigType::Enum,
            6 => ConfigType::String,
            _ => ConfigType::Other,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Memory {
    Small,
    Medium,
    Large,
    Ludicrous,
}

impl From<u32> for Memory {
    fn from(value: u32) -> Self {
        match value {
            sys::AARTSAAPI_MEMORY_MEDIUM => Memory::Medium,
            sys::AARTSAAPI_MEMORY_LARGE => Memory::Large,
            sys::AARTSAAPI_MEMORY_LUDICROUS => Memory::Ludicrous,
            _ => Memory::Small,
        }
    }
}

impl From<Memory> for u32 {
    fn from(value: Memory) -> Self {
        match value {
            Memory::Small => sys::AARTSAAPI_MEMORY_SMALL,
            Memory::Medium => sys::AARTSAAPI_MEMORY_MEDIUM,
            Memory::Large => sys::AARTSAAPI_MEMORY_LARGE,
            Memory::Ludicrous => sys::AARTSAAPI_MEMORY_LUDICROUS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PacketFlags {
    v: u64,
}

impl PacketFlags {
    pub fn new(v: u64) -> Self {
        PacketFlags { v }
    }
    pub fn segment_start(&self) -> bool {
        self.v & sys::AARTSAAPI_PACKET_SEGMENT_START as u64 != 0
    }
    pub fn segment_end(&self) -> bool {
        self.v & sys::AARTSAAPI_PACKET_SEGMENT_END as u64 != 0
    }
    pub fn stream_start(&self) -> bool {
        self.v & sys::AARTSAAPI_PACKET_STREAM_START as u64 != 0
    }
    pub fn stream_end(&self) -> bool {
        self.v & sys::AARTSAAPI_PACKET_STREAM_END as u64 != 0
    }
}

pub type Result = std::result::Result<(), Error>;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Empty")]
    Empty,
    #[error("Retry")]
    Retry,

    #[error("Idle")]
    Idle,
    #[error("Connecting")]
    Connecting,
    #[error("Connected")]
    Connected,
    #[error("Starting")]
    Starting,
    #[error("Running")]
    Running,
    #[error("Stopping")]
    Stopping,
    #[error("Disconnecting")]
    Disconnecting,

    #[error("Warning")]
    Warning,
    #[error("Warning Value Adjusted")]
    WarningValueAdjusted,
    #[error("Warning Value Disabled")]
    WarningValueDisabled,

    #[error("Error")]
    Error,
    #[error("Error Not Initialized")]
    ErrorNotInitialized,
    #[error("Error Not Found")]
    ErrorNotFound,
    #[error("Error Busy")]
    ErrorBusy,
    #[error("Error Not Open")]
    ErrorNotOpen,
    #[error("Error Not Connected")]
    ErrorNotConnected,
    #[error("Error Invalid Config")]
    ErrorInvalidConfig,
    #[error("Error Buffer Size")]
    ErrorBufferSize,
    #[error("Error Invalid Channel")]
    ErrorInvalidChannel,
    #[error("Error Invalid Parameter")]
    ErrorInvalidParameter,
    #[error("Error Invalid Size")]
    ErrorInvalidSize,
    #[error("Error Missing Paths File")]
    ErrorMissingPathsFile,
    #[error("Error Value Invalid")]
    ErrorValueInvalid,
    #[error("Error Value Malformed")]
    ErrorValueMalformed,

    #[error("Undocumented")]
    Undocumented,
}

fn res(r: sys::AARTSAAPI_Result) -> Result {
    match r {
        0x00000000 => Ok(()),
        0x00000001 => Err(Error::Empty),
        0x00000002 => Err(Error::Retry),

        0x10000000 => Err(Error::Idle),
        0x10000001 => Err(Error::Connecting),
        0x10000002 => Err(Error::Connected),
        0x10000003 => Err(Error::Starting),
        0x10000004 => Err(Error::Running),
        0x10000005 => Err(Error::Stopping),
        0x10000006 => Err(Error::Disconnecting),

        0x40000000 => Err(Error::Warning),
        0x40000001 => Err(Error::WarningValueAdjusted),
        0x40000002 => Err(Error::WarningValueDisabled),

        0x80000000 => Err(Error::Error),
        0x80000001 => Err(Error::ErrorNotInitialized),
        0x80000002 => Err(Error::ErrorNotFound),
        0x80000003 => Err(Error::ErrorBusy),
        0x80000004 => Err(Error::ErrorNotOpen),
        0x80000005 => Err(Error::ErrorNotConnected),
        0x80000006 => Err(Error::ErrorInvalidConfig),
        0x80000007 => Err(Error::ErrorBufferSize),
        0x80000008 => Err(Error::ErrorInvalidChannel),
        0x80000009 => Err(Error::ErrorInvalidParameter),
        0x8000000a => Err(Error::ErrorInvalidSize),
        0x8000000b => Err(Error::ErrorMissingPathsFile),
        0x8000000c => Err(Error::ErrorValueInvalid),
        0x8000000d => Err(Error::ErrorValueMalformed),
        _ => Err(Error::Undocumented),
    }
}

pub fn version() -> String {
    let n = unsafe { sys::AARTSAAPI_Version() };
    format!("{}.{}", n >> 16, n & 0xffff)
}

pub fn init(memory: Memory) -> Result {
    unsafe { res(sys::AARTSAAPI_Init(memory.into())) }
}

pub fn shutdown() -> Result {
    unsafe { res(sys::AARTSAAPI_Shutdown()) }
}

pub fn handle() -> std::result::Result<Handle, Error> {
    let mut h = sys::AARTSAAPI_Handle {
        d: std::ptr::null_mut(),
    };
    unsafe { res(sys::AARTSAAPI_Open(&mut h)).map(|_| Handle { inner: h }) }
}

pub fn rescan_devices(h: &mut Handle) -> Result {
    loop {
        let r = unsafe { res(sys::AARTSAAPI_RescanDevices(&mut h.inner, 2000)) };
        match r {
            Ok(()) => break Ok(()),
            Err(Error::Retry) => continue,
            Err(e) => return Err(e),
        }
    }
}

pub fn reset_devices(h: &mut Handle) -> Result {
    unsafe { res(sys::AARTSAAPI_ResetDevices(&mut h.inner)) }
}

pub fn devices(h: &mut Handle) -> std::result::Result<Vec<DeviceInfo>, Error> {
    let mut devices = Vec::new();
    let device_type = wchar::wchz!("spectranv6");

    for i in 0.. {
        let mut di = DeviceInfo::new();
        match unsafe {
            res(sys::AARTSAAPI_EnumDevice(
                &mut h.inner,
                device_type.as_ptr() as _,
                i,
                &mut di.inner,
            ))
        } {
            Ok(()) => devices.push(di),
            Err(Error::Empty) => break,
            Err(e) => return Err(e),
        }
    }

    Ok(devices)
}

//// Enumerate the devices for a given type.  The list starts with index 0 and will
//// return AARTSAAPI_EMPTY when the end of the list is reached.
////
//AARONIARTSAAPI_EXPORT AARTSAAPI_Result AARTSAAPI_EnumDevice(AARTSAAPI_Handle * handle, const wchar_t * type, int32_t index, AARTSAAPI_DeviceInfo * dinfo);
//
//// Open a device for exclusive use.  This allocates the required data structures
//// and prepares the configuration settings, but will not access the hardware.
////
//AARONIARTSAAPI_EXPORT AARTSAAPI_Result AARTSAAPI_OpenDevice(AARTSAAPI_Handle * handle, AARTSAAPI_Device * dhandle, const wchar_t * type, const wchar_t * serialNumber);
//
//// Close a device
////
//AARONIARTSAAPI_EXPORT AARTSAAPI_Result AARTSAAPI_CloseDevice(AARTSAAPI_Handle * handle, AARTSAAPI_Device * dhandle);
//
//// Connect to the pysical device
////
//AARONIARTSAAPI_EXPORT AARTSAAPI_Result AARTSAAPI_ConnectDevice(AARTSAAPI_Device * dhandle);
//
//// Disconnect from the physical device
////
//AARONIARTSAAPI_EXPORT AARTSAAPI_Result AARTSAAPI_DisconnectDevice(AARTSAAPI_Device * dhandle);
