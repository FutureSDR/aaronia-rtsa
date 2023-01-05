use aaronia_rtsa::ApiHandle;
use aaronia_rtsa::version;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RTSA library version: {}", version());

    let mut api = ApiHandle::new()?;
    api.rescan_devices()?;
    let d = api.devices()?;
    println!("devices {:?}", d);

    Ok(())
}
