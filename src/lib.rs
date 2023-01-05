#![allow(dead_code)]
use aaronia_rtsa_sys as sys;
use std::sync::Mutex;
use widestring::WideString;

pub fn version() -> String {
    let n = unsafe { sys::AARTSAAPI_Version() };
    format!("{}.{}", n >> 16, n & 0xffff)
}

static API: Mutex<Option<Api>> = Mutex::new(None);

struct Api {
    handles: usize,
}

impl Api {
    fn new(mem: Memory) -> Self {
        unsafe { res(sys::AARTSAAPI_Init(mem.into())).expect("RTSA library initialization failed") }
        Self { handles: 0 }
    }

    fn add_handle(&mut self) {
        self.handles += 1;
    }

    fn remove_handle(&mut self) {
        self.handles -= 1;
    }

    fn handles(&self) -> usize {
        self.handles
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        unsafe { res(sys::AARTSAAPI_Shutdown()).expect("RTSA library shutdown failed") }
    }
}

#[derive(Debug)]
pub struct ApiHandle {
    inner: sys::AARTSAAPI_Handle,
}

impl ApiHandle {
    pub fn new() -> std::result::Result<Self, Error> {
        Self::with_mem(Memory::Medium)
    }

    pub fn with_mem(mem: Memory) -> std::result::Result<Self, Error> {
        let mut api = API.lock().unwrap();

        if api.is_none() {
            *api = Some(Api::new(mem));
        }

        let mut h = sys::AARTSAAPI_Handle {
            d: std::ptr::null_mut(),
        };
        unsafe {
            match res(sys::AARTSAAPI_Open(&mut h)) {
                Ok(()) => {
                    api.as_mut().unwrap().add_handle();
                    return Ok(ApiHandle { inner: h });
                }
                Err(e) => {
                    if api.as_mut().unwrap().handles() == 0 {
                        *api = None;
                    }
                    return Err(e);
                }
            }
        }
    }

    pub fn rescan_devices(&mut self) -> Result {
        loop {
            let r = unsafe { res(sys::AARTSAAPI_RescanDevices(&mut self.inner, 10000)) };
            match r {
                Ok(()) => break Ok(()),
                Err(Error::Retry) => continue,
                Err(e) => return Err(e),
            }
        }
    }

    pub fn reset_devices(&mut self) -> Result {
        unsafe { res(sys::AARTSAAPI_ResetDevices(&mut self.inner)) }
    }

    pub fn devices(&mut self) -> std::result::Result<Vec<DeviceInfo>, Error> {
        let mut devices = Vec::new();
        let device_type = WideString::from("spectranv6");

        for i in 0.. {
            let mut di = DeviceInfo::new();
            match unsafe {
                res(sys::AARTSAAPI_EnumDevice(
                    &mut self.inner,
                    device_type.as_ptr(),
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

    pub fn open_device(&mut self) -> std::result::Result<Device, Error> {
        let devs = self.devices()?;
        if let Some(d) = devs.get(0) {
            self.open_this_device(&d)
        } else {
            Err(Error::Empty)
        }
    }

    pub fn open_this_device(&mut self, info: &DeviceInfo) -> std::result::Result<Device, Error> {
        let mut dev = Device::new();
        let device_type = WideString::from("spectranv6/raw");
        unsafe {
            res(sys::AARTSAAPI_OpenDevice(
                &mut self.inner,
                &mut dev.inner,
                device_type.as_ptr(),
                info.inner.serialNumber.as_ptr(),
            ))?;
        }
        Ok(dev)
    }
}

impl Drop for ApiHandle {
    fn drop(&mut self) {
        unsafe {
            res(sys::AARTSAAPI_Close(&mut self.inner)).expect("error dropping API handle");
        }

        let mut api = API.lock().unwrap();

        api.as_mut().unwrap().remove_handle();
        if api.as_mut().unwrap().handles() == 0 {
            *api = None;
        }
    }
}

pub struct Config {
    inner: sys::AARTSAAPI_Config,
}

pub struct ConfigInfo {
    inner: sys::AARTSAAPI_ConfigInfo,
}

pub struct Device {
    inner: sys::AARTSAAPI_Device,
}

impl Device {
    fn new() -> Self {
        Device {
            inner: sys::AARTSAAPI_Device {
                d: std::ptr::null_mut(),
            },
        }
    }
}

#[derive(Debug)]
pub struct DeviceInfo {
    inner: sys::AARTSAAPI_DeviceInfo,
}

impl DeviceInfo {
    fn new() -> Self {
        Self {
            inner: sys::AARTSAAPI_DeviceInfo {
                cbsize: std::mem::size_of::<sys::AARTSAAPI_DeviceInfo>() as _,
                serialNumber: [0; 120],
                ready: false,
                boost: false,
                superspeed: false,
                active: false,
            },
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
