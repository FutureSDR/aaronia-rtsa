use aaronia_rtsa_sys as sys;
use std::collections::HashMap;
use std::sync::Mutex;
use widestring::WideCString;

/// Version String (`<major>.<minor>`)
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

/// Handle to interface the library
///
/// Internally, all [`ApiHandle`]s use one global library handle, which is dropped when there are
/// no [`ApiHandle`]s left. Only the first handle, i.e., the one that creates the global library
/// handle can configure the [`Memory`] size. Later created [`ApiHandle`]s ignore the memory
/// parameter.
#[derive(Debug)]
pub struct ApiHandle {
    inner: sys::AARTSAAPI_Handle,
}

impl ApiHandle {
    /// Create [`ApiHandle`] with default [`Memory`] size medium.
    pub fn new() -> std::result::Result<Self, Error> {
        Self::with_mem(Memory::Medium)
    }

    /// Create [`ApiHandle`] with given [`Memory`] size.
    ///
    /// The memory size is only considered, if this is the first [`ApiHandle`], i.e. the
    /// one that initializes the underlying RTSA library.
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
                    Ok(ApiHandle { inner: h })
                }
                Err(e) => {
                    if api.as_mut().unwrap().handles() == 0 {
                        *api = None;
                    }
                    Err(e)
                }
            }
        }
    }

    /// Rescan for devices.
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

    /// Reset all devices.
    pub fn reset_devices(&mut self) -> Result {
        unsafe { res(sys::AARTSAAPI_ResetDevices(&mut self.inner)) }
    }

    /// Get a list with information about all detected devices.
    pub fn devices(&mut self) -> std::result::Result<Vec<DeviceInfo>, Error> {
        let mut devices = Vec::new();
        let device_type = WideCString::from_str_truncate("spectranv6");

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

    /// Get the first detected [`Device`].
    pub fn get_device(&mut self) -> std::result::Result<Device, Error> {
        let devs = self.devices()?;
        if let Some(d) = devs.get(0) {
            self.get_this_device(d)
        } else {
            Err(Error::Empty)
        }
    }

    /// Get a specific [`Device`], identified by its [`DeviceInfo`].
    ///
    /// The [DeviceInfo] can be get from the [devices()](Self::devices) function.
    pub fn get_this_device(&mut self, info: &DeviceInfo) -> std::result::Result<Device, Error> {
        Device::new(info)
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

struct Config {
    inner: sys::AARTSAAPI_Config,
}

impl Config {
    fn new() -> Self {
        Self {
            inner: sys::AARTSAAPI_Config {
                d: std::ptr::null_mut(),
            },
        }
    }
}

struct ConfigInfo {
    inner: sys::AARTSAAPI_ConfigInfo,
}

impl ConfigInfo {
    fn new() -> Self {
        Self {
            inner: sys::AARTSAAPI_ConfigInfo {
                cbsize: std::mem::size_of::<sys::AARTSAAPI_ConfigInfo>() as _,
                name: [0; 80],
                title: [0; 120],
                type_: 0,
                minValue: 0.0,
                maxValue: 0.0,
                stepValue: 0.0,
                unit: [0; 10],
                options: [0; 1000],
                disabledOptions: 0,
            },
        }
    }
}

#[derive(Debug, PartialEq)]
enum DeviceStatus {
    Uninit,
    Opened,
    Connected,
    Started,
}

/// Device state can be queried with [`Device::state()`]
#[derive(Debug, PartialEq)]
pub enum DeviceState {
    Idle,
    Connecting,
    Connected,
    Starting,
    Running,
    Stopping,
    Disconnecting,
}

impl TryInto<DeviceState> for Error {
    type Error = Error;

    fn try_into(self) -> std::result::Result<DeviceState, <Error as TryInto<DeviceState>>::Error> {
        match self {
            Error::Idle => Ok(DeviceState::Idle),
            Error::Connecting => Ok(DeviceState::Connecting),
            Error::Connected => Ok(DeviceState::Connected),
            Error::Starting => Ok(DeviceState::Starting),
            Error::Running => Ok(DeviceState::Running),
            Error::Stopping => Ok(DeviceState::Stopping),
            Error::Disconnecting => Ok(DeviceState::Disconnecting),
            x => Err(x),
        }
    }
}

/// A device, created through the [ApiHandle].
///
/// The typical life-cycle of a device is:
/// - Create with [`ApiHandle`]
/// - [`Device::open()`]
/// - Configure with [`Device::set()`], [`Device::set_int()`], and [`Device::set_float()`]
/// - [`Device::connect()`]
/// - [`Device::start()`]
/// - [`Device::stop()`]
/// - [`Device::disconnect()`]
/// - [`Device::close()`]
pub struct Device {
    inner: sys::AARTSAAPI_Device,
    api: ApiHandle,
    status: DeviceStatus,
    serial: WideCString,
}

impl Device {
    fn new(info: &DeviceInfo) -> std::result::Result<Self, Error> {
        Ok(Device {
            inner: sys::AARTSAAPI_Device {
                d: std::ptr::null_mut(),
            },
            api: ApiHandle::new()?,
            status: DeviceStatus::Uninit,
            serial: WideCString::from_vec_truncate(info.inner.serialNumber),
        })
    }

    /// Open the [`Device`] for exclusive use.
    ///
    /// This allocates the required data structures and prepares the configuration settings, but
    /// will not access the hardware.
    pub fn open(&mut self) -> Result {
        assert_eq!(self.status, DeviceStatus::Uninit);
        let device_type = WideCString::from_str_truncate("spectranv6/raw");

        unsafe {
            res(sys::AARTSAAPI_OpenDevice(
                &mut self.api.inner,
                &mut self.inner,
                device_type.as_ptr(),
                self.serial.as_ptr(),
            ))?;
        }

        self.status = DeviceStatus::Opened;

        Ok(())
    }

    /// Close the [`Device`] for exclusive use.
    pub fn close(&mut self) -> Result {
        assert_eq!(self.status, DeviceStatus::Opened);
        unsafe {
            res(sys::AARTSAAPI_CloseDevice(
                &mut self.api.inner,
                &mut self.inner,
            ))?
        }
        self.status = DeviceStatus::Uninit;
        Ok(())
    }

    /// Connect to the [`Device`].
    pub fn connect(&mut self) -> Result {
        assert_eq!(self.status, DeviceStatus::Opened);
        unsafe { res(sys::AARTSAAPI_ConnectDevice(&mut self.inner))? }
        self.status = DeviceStatus::Connected;
        Ok(())
    }

    /// Disconnect from the [`Device`].
    pub fn disconnect(&mut self) -> Result {
        assert_eq!(self.status, DeviceStatus::Connected);
        unsafe { res(sys::AARTSAAPI_ConnectDevice(&mut self.inner))? }
        self.status = DeviceStatus::Opened;
        Ok(())
    }

    /// Start data acqusition from the [`Device] / data transmission to the [`Device`].
    pub fn start(&mut self) -> Result {
        assert_eq!(self.status, DeviceStatus::Connected);
        unsafe { res(sys::AARTSAAPI_StartDevice(&mut self.inner))? }
        self.status = DeviceStatus::Started;
        Ok(())
    }

    /// Stop data acqusition from the [`Device`] / data transmission to the [`Device`].
    pub fn stop(&mut self) -> Result {
        assert_eq!(self.status, DeviceStatus::Started);
        unsafe { res(sys::AARTSAAPI_StopDevice(&mut self.inner))? }
        self.status = DeviceStatus::Connected;
        Ok(())
    }

    /// Get [`DeviceState`] from the [`Device`].
    pub fn state(&mut self) -> std::result::Result<DeviceState, Error> {
        let res = unsafe { res(sys::AARTSAAPI_GetDeviceState(&mut self.inner)) };
        match res {
            Ok(()) => Err(Error::Error),
            Err(e) => e.try_into(),
        }
    }

    /// Get [`Device`] configuration parameter.
    pub fn get<S: AsRef<str>>(&mut self, path: S) -> std::result::Result<ConfigItem, Error> {
        let mut root = Config::new();
        let mut node = Config::new();
        let path = WideCString::from_str_truncate(path.as_ref());

        unsafe { res(sys::AARTSAAPI_ConfigRoot(&mut self.inner, &mut root.inner))? };
        unsafe {
            res(sys::AARTSAAPI_ConfigFind(
                &mut self.inner,
                &mut root.inner,
                &mut node.inner,
                path.as_ptr(),
            ))?
        };

        let (_, item) = self.parse_item(&mut node)?;

        Ok(item)
    }

    /// Set [`Device`] configuration parameter as string.
    pub fn set<S1: AsRef<str>, S2: AsRef<str>>(&mut self, path: S1, value: S2) -> Result {
        let path = WideCString::from_str_truncate(path.as_ref());
        let value = WideCString::from_str_truncate(value.as_ref());

        let mut root = Config::new();
        let mut node = Config::new();

        unsafe { res(sys::AARTSAAPI_ConfigRoot(&mut self.inner, &mut root.inner))? };
        unsafe {
            res(sys::AARTSAAPI_ConfigFind(
                &mut self.inner,
                &mut root.inner,
                &mut node.inner,
                path.as_ptr(),
            ))?
        };
        unsafe {
            res(sys::AARTSAAPI_ConfigSetString(
                &mut self.inner,
                &mut node.inner,
                value.as_ptr(),
            ))?
        };

        Ok(())
    }

    /// Set [`Device`] configuration parameter as float.
    pub fn set_float<S1: AsRef<str>, F: Into<f64>>(&mut self, path: S1, value: F) -> Result {
        let path = WideCString::from_str_truncate(path.as_ref());

        let mut root = Config::new();
        let mut node = Config::new();

        unsafe { res(sys::AARTSAAPI_ConfigRoot(&mut self.inner, &mut root.inner))? };
        unsafe {
            res(sys::AARTSAAPI_ConfigFind(
                &mut self.inner,
                &mut root.inner,
                &mut node.inner,
                path.as_ptr(),
            ))?
        };
        unsafe {
            res(sys::AARTSAAPI_ConfigSetFloat(
                &mut self.inner,
                &mut node.inner,
                value.into(),
            ))?
        };

        Ok(())
    }

    /// Set [`Device`] configuration parameter as integer.
    pub fn set_int<S1: AsRef<str>, F: Into<i64>>(&mut self, path: S1, value: F) -> Result {
        let path = WideCString::from_str_truncate(path.as_ref());

        let mut root = Config::new();
        let mut node = Config::new();

        unsafe { res(sys::AARTSAAPI_ConfigRoot(&mut self.inner, &mut root.inner))? };
        unsafe {
            res(sys::AARTSAAPI_ConfigFind(
                &mut self.inner,
                &mut root.inner,
                &mut node.inner,
                path.as_ptr(),
            ))?
        };
        unsafe {
            res(sys::AARTSAAPI_ConfigSetInteger(
                &mut self.inner,
                &mut node.inner,
                value.into(),
            ))?
        };

        Ok(())
    }

    /// Query [`Packet`] queue of [`Device`] data channel.
    pub fn packets_avail(&mut self, chan: i32) -> std::result::Result<usize, Error> {
        let mut n = 0i32;
        unsafe { res(sys::AARTSAAPI_AvailPackets(&mut self.inner, chan, &mut n))? };
        Ok(n as usize)
    }

    /// Get [`Packet`] from the [`Device`].
    ///
    /// This call is blocking, polling the queue every 5ms, in case it is empty.
    pub fn packet(&mut self, chan: i32) -> std::result::Result<Packet, Error> {
        let mut packet = Packet::new();

        loop {
            let ret = unsafe {
                res(sys::AARTSAAPI_GetPacket(
                    &mut self.inner,
                    chan,
                    0,
                    &mut packet.inner,
                ))
            };
            match ret {
                Ok(_) => return Ok(packet),
                Err(Error::Empty) => {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Try to get a [`Packet`] from the [`Device`] data channel.
    ///
    /// This call is non-blocking.
    pub fn try_packet(&mut self, chan: i32) -> std::result::Result<Packet, Error> {
        let mut packet = Packet::new();

        unsafe {
            res(sys::AARTSAAPI_GetPacket(
                &mut self.inner,
                chan,
                0,
                &mut packet.inner,
            ))
        }
        .map(|_| packet)
    }

    /// Send a [`Packet`] to the [`Device`] data channel.
    pub fn send_packet(&mut self, chan: i32, packet: &Packet) -> Result {
        unsafe {
            res(sys::AARTSAAPI_SendPacket(
                &mut self.inner,
                chan,
                &packet.inner,
            ))
        }
    }

    /// Consume a [`Packet`] from a [`Device`] data channel.
    pub fn consume(&mut self, chan: i32) -> Result {
        unsafe { res(sys::AARTSAAPI_ConsumePackets(&mut self.inner, chan, 1)) }
    }

    /// Get [`Device`] clock time.
    pub fn clock(&mut self) -> std::result::Result<f64, Error> {
        let mut val = 0.0f64;
        unsafe {
            res(sys::AARTSAAPI_GetMasterStreamTime(
                &mut self.inner,
                &mut val,
            ))?
        };
        Ok(val)
    }

    /// Print the [`Device`] configuration parameter tree.
    pub fn print_config(&mut self) -> Result {
        let mut conf = HashMap::<String, ConfigItem>::new();
        let mut root = Config::new();

        unsafe { res(sys::AARTSAAPI_ConfigRoot(&mut self.inner, &mut root.inner))? };

        let (name, item) = self.parse_item(&mut root)?;
        conf.insert(name, item);

        println!("config: {conf:#?}");

        Ok(())
    }

    /// Print the [`Device`] health parameter tree.
    pub fn print_health(&mut self) -> Result {
        let mut conf = HashMap::<String, ConfigItem>::new();

        let mut root = Config::new();

        unsafe {
            res(sys::AARTSAAPI_ConfigHealth(
                &mut self.inner,
                &mut root.inner,
            ))?
        };

        let (name, item) = self.parse_item(&mut root)?;
        conf.insert(name, item);

        println!("health: {conf:#?}");

        Ok(())
    }

    fn parse_item(
        &mut self,
        node: &mut Config,
    ) -> std::result::Result<(String, ConfigItem), Error> {
        let mut info = ConfigInfo::new();

        unsafe {
            res(sys::AARTSAAPI_ConfigGetInfo(
                &mut self.inner,
                &mut node.inner,
                &mut info.inner,
            ))?
        };

        let item = match info.inner.type_ {
            sys::AARTSAAPI_ConfigType_AARTSAAPI_CONFIG_TYPE_BLOB => ConfigItem::Blob,
            sys::AARTSAAPI_ConfigType_AARTSAAPI_CONFIG_TYPE_BOOL => {
                let mut val = 0i64;
                match unsafe {
                    res(sys::AARTSAAPI_ConfigGetInteger(
                        &mut self.inner,
                        &mut node.inner,
                        &mut val,
                    ))
                } {
                    Ok(_) => ConfigItem::Bool(val > 0),
                    Err(Error::ErrorInvalidConfig) => ConfigItem::Button,
                    Err(e) => return Err(e),
                }
            }
            sys::AARTSAAPI_ConfigType_AARTSAAPI_CONFIG_TYPE_ENUM => {
                let s = WideCString::from_vec_truncate(info.inner.options)
                    .to_string_lossy()
                    .split(';')
                    .map(|s| s.into())
                    .collect();

                let mut val = 0i64;
                unsafe {
                    res(sys::AARTSAAPI_ConfigGetInteger(
                        &mut self.inner,
                        &mut node.inner,
                        &mut val,
                    ))?
                }
                ConfigItem::Enum(val, s)
            }
            sys::AARTSAAPI_ConfigType_AARTSAAPI_CONFIG_TYPE_NUMBER => {
                let mut num = 0.0f64;
                unsafe {
                    res(sys::AARTSAAPI_ConfigGetFloat(
                        &mut self.inner,
                        &mut node.inner,
                        &mut num,
                    ))?
                };
                ConfigItem::Number(num)
            }
            sys::AARTSAAPI_ConfigType_AARTSAAPI_CONFIG_TYPE_STRING => ConfigItem::String(
                WideCString::from_vec_truncate(info.inner.options).to_string_lossy(),
            ),
            sys::AARTSAAPI_ConfigType_AARTSAAPI_CONFIG_TYPE_GROUP => {
                let mut items = HashMap::new();
                let mut n = Config::new();

                unsafe {
                    res(sys::AARTSAAPI_ConfigFirst(
                        &mut self.inner,
                        &mut node.inner,
                        &mut n.inner,
                    ))?
                };

                let (name, item) = self.parse_item(&mut n)?;
                items.insert(name, item);

                loop {
                    match unsafe {
                        res(sys::AARTSAAPI_ConfigNext(
                            &mut self.inner,
                            &mut node.inner,
                            &mut n.inner,
                        ))
                    } {
                        Ok(_) => {
                            let (name, item) = self.parse_item(&mut n)?;
                            items.insert(name, item);
                        }
                        Err(Error::Empty) => break,
                        Err(e) => return Err(e),
                    }
                }

                ConfigItem::Group(items)
            }
            _ => ConfigItem::Other,
        };

        Ok((
            WideCString::from_vec_truncate(info.inner.name).to_string_lossy(),
            item,
        ))
    }
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("api", &self.api)
            .field("status", &self.status)
            .field("serial", &self.serial)
            .finish()
    }
}

/// [`Device`] configuration parameter.
#[derive(Debug)]
pub enum ConfigItem {
    Blob,
    Bool(bool),
    Button,
    Enum(i64, Vec<String>),
    Group(HashMap<String, ConfigItem>),
    Number(f64),
    Other,
    String(String),
}

impl Drop for Device {
    fn drop(&mut self) {
        match self.status {
            DeviceStatus::Uninit => {}
            DeviceStatus::Opened => {
                let _ = self.close();
            }
            DeviceStatus::Connected => {
                let _ = self.disconnect().and_then(|_| self.close());
            }
            DeviceStatus::Started => {
                let _ = self
                    .stop()
                    .and_then(|_| self.disconnect())
                    .and_then(|_| self.close());
            }
        }
    }
}

/// Information about a [`Device`].
///
/// Can be used to identify the device for opening.
#[derive(Clone)]
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
    pub fn serial(&self) -> String {
        WideCString::from_vec_truncate(self.inner.serialNumber).to_string_lossy()
    }
    pub fn ready(&self) -> bool {
        self.inner.ready
    }
    pub fn boost(&self) -> bool {
        self.inner.boost
    }
    pub fn superspeed(&self) -> bool {
        self.inner.superspeed
    }
    pub fn active(&self) -> bool {
        self.inner.active
    }
}

impl std::fmt::Debug for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceInfo")
            .field(
                "serial",
                &WideCString::from_vec_truncate(self.inner.serialNumber).to_string_lossy(),
            )
            .field("ready", &self.inner.ready)
            .field("boost", &self.inner.boost)
            .field("superspeed", &self.inner.superspeed)
            .field("active", &self.inner.active)
            .finish()
    }
}

/// Packet that holds IQ or spectrum data.
///
/// Packets are used for RX and TX.
#[derive(Debug)]
pub struct Packet {
    inner: sys::AARTSAAPI_Packet,
}

impl Packet {
    fn new() -> Self {
        Self {
            inner: sys::AARTSAAPI_Packet {
                cbsize: std::mem::size_of::<sys::AARTSAAPI_Packet>() as _,
                streamID: 0,
                flags: 0,
                startTime: 0.0,
                endTime: 0.0,
                startFrequency: 0.0,
                stepFrequency: 0.0,
                spanFrequency: 0.0,
                rbwFrequency: 0.0,
                num: 0,
                total: 0,
                size: 0,
                stride: 0,
                fp32: std::ptr::null_mut(),
            },
        }
    }

    /// Get stream ID.
    pub fn stream_id(&self) -> u64 {
        self.inner.streamID
    }
    /// Get packet flags.
    pub fn flags(&self) -> PacketFlags {
        PacketFlags::from(self.inner.flags)
    }
    /// Get packet start time.
    pub fn start_time(&self) -> f64 {
        self.inner.startTime
    }
    /// Get packet end time.
    pub fn end_time(&self) -> f64 {
        self.inner.endTime
    }
    /// Get packet start frequency.
    pub fn start_frequency(&self) -> f64 {
        self.inner.startFrequency
    }
    /// Get packet step frequency.
    pub fn step_frequency(&self) -> f64 {
        self.inner.stepFrequency
    }
    /// Get packet span frequency.
    pub fn span_frequency(&self) -> f64 {
        self.inner.spanFrequency
    }
    /// Get packet real-time bandwidth.
    pub fn rbw_frequency(&self) -> f64 {
        self.inner.rbwFrequency
    }
    /// Get number of items in stride.
    pub fn num(&self) -> i64 {
        self.inner.num
    }
    /// Get total number of items in packet.
    pub fn total(&self) -> i64 {
        self.inner.total
    }
    /// Get total size of packet.
    pub fn size(&self) -> i64 {
        self.inner.size
    }
    /// Get sample stride.
    pub fn stride(&self) -> i64 {
        self.inner.stride
    }

    /// Get IQ samples from packet.
    pub fn samples(&self) -> &'static [num_complex::Complex32] {
        unsafe { std::slice::from_raw_parts(self.inner.fp32 as _, self.inner.num as _) }
    }

    /// Get spectrum data from packet.
    pub fn spectrum(&self) -> &'static [f32] {
        unsafe { std::slice::from_raw_parts(self.inner.fp32 as _, self.inner.size as _) }
    }
}

#[derive(Debug, Clone)]
enum ConfigType {
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

/// Options for memory sizes, used by the RTSA library.
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

/// Packet Flags
#[derive(Debug, Clone)]
pub struct PacketFlags {
    v: u64,
}

impl PacketFlags {
    /// Create struct, setting all flags to false.
    pub fn new() -> Self {
        PacketFlags { v: 0 }
    }
    /// Is packet start of a segment?
    pub fn segment_start(&self) -> bool {
        self.v & sys::AARTSAAPI_PACKET_SEGMENT_START as u64 != 0
    }
    /// Is packet end of a segment?
    pub fn segment_end(&self) -> bool {
        self.v & sys::AARTSAAPI_PACKET_SEGMENT_END as u64 != 0
    }
    /// Is packet start of a stream?
    pub fn stream_start(&self) -> bool {
        self.v & sys::AARTSAAPI_PACKET_STREAM_START as u64 != 0
    }
    /// Is packet end of a stream?
    pub fn stream_end(&self) -> bool {
        self.v & sys::AARTSAAPI_PACKET_STREAM_END as u64 != 0
    }
    /// Set flag to indicate start of a segment.
    pub fn set_segment_start(&mut self) -> &mut Self {
        self.v |= sys::AARTSAAPI_PACKET_SEGMENT_START as u64;
        self
    }
    /// Set flag to indicate end of a segment.
    pub fn set_segment_end(&mut self) -> &mut Self {
        self.v |= sys::AARTSAAPI_PACKET_SEGMENT_END as u64;
        self
    }
    /// Set flag to indicate start of a stream.
    pub fn set_stream_start(&mut self) -> &mut Self {
        self.v |= sys::AARTSAAPI_PACKET_STREAM_START as u64;
        self
    }
    /// Set flag to indicate end of a stream.
    pub fn set_stream_end(&mut self) -> &mut Self {
        self.v |= sys::AARTSAAPI_PACKET_STREAM_END as u64;
        self
    }
}

impl From<PacketFlags> for u64 {
    fn from(value: PacketFlags) -> Self {
        value.v
    }
}

impl From<u64> for PacketFlags {
    fn from(value: u64) -> Self {
        Self { v: value }
    }
}

impl Default for PacketFlags {
    fn default() -> Self {
        Self::new()
    }
}

pub type Result = std::result::Result<(), Error>;

/// RTSA library error
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
