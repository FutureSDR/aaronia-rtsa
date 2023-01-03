#![allow(dead_code)]
use aaronia_rtsa_sys as sys;

pub struct Config {
    inner: sys::AARTSAAPI_Config,
}

pub struct ConfigInfo {
    inner: sys::AARTSAAPI_ConfigInfo,
}

pub struct Device {
    inner: sys::AARTSAAPI_Device,
}

pub struct DeviceInfo {
    inner: sys::AARTSAAPI_DeviceInfo,
}

pub struct Handle {
    inner: sys::AARTSAAPI_Handle,
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
            1 => Memory::Medium,
            2 => Memory::Large,
            3 => Memory::Ludicrous,
            _ => Memory::Small,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PacketFlags {
    v: u32,
}

impl PacketFlags {
    pub fn new(v: u32) -> Self {
        PacketFlags { v }
    }
    pub fn segment_start(&self) -> bool {
        self.v & sys::AARTSAAPI_PACKET_SEGMENT_START != 0
    }
    pub fn segment_end(&self) -> bool {
        self.v & sys::AARTSAAPI_PACKET_SEGMENT_END != 0
    }
    pub fn stream_start(&self) -> bool {
        self.v & sys::AARTSAAPI_PACKET_STREAM_START != 0
    }
    pub fn stream_end(&self) -> bool {
        self.v & sys::AARTSAAPI_PACKET_STREAM_END != 0
    }
}

pub fn version() -> String {
    let n = unsafe { sys::AARTSAAPI_Version() };
    format!("{}.{}", n >> 16, n & 0xffff)
}
