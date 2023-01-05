use aaronia_rtsa::handle;
use aaronia_rtsa::init;
use aaronia_rtsa::shutdown;
use aaronia_rtsa::version;
use aaronia_rtsa::rescan_devices;
use aaronia_rtsa::reset_devices;
use aaronia_rtsa::devices;
use aaronia_rtsa::Memory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RTSA library version: {}", version());

    init(Memory::Medium)?;
    {
        let mut h = handle()?;
        println!("rescan");
        rescan_devices(&mut h)?;
        println!("devices");
        let d = devices(&mut h)?;
        println!("devices {:?}", d);
    }
    shutdown()?;

    Ok(())
}
