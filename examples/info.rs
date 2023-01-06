use aaronia_rtsa::version;
use aaronia_rtsa::ApiHandle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RTSA library version: {}", version());

    let mut api = ApiHandle::new()?;
    api.rescan_devices()?;
    let d = api.devices()?;
    println!("devices {:?}", d);

    let mut dev = api.get_device()?;
    dev.open()?;
    dev.print_config()?;
    dev.print_health()?;
    dev.close()?;

    Ok(())
}
