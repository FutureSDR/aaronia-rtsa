use aaronia_rtsa::handle;
use aaronia_rtsa::init;
use aaronia_rtsa::shutdown;
use aaronia_rtsa::version;
use aaronia_rtsa::Memory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RTSA library version: {}", version());

    init(Memory::Medium)?;
    {
        let _h = handle()?;
    }
    shutdown()?;

    Ok(())
}
