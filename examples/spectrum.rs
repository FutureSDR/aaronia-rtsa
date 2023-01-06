use aaronia_rtsa::version;
use aaronia_rtsa::ApiHandle;
use aaronia_rtsa::Device;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RTSA library version: {}", version());

    let mut api = ApiHandle::new()?;
    api.rescan_devices()?;
    let d = api.devices()?;
    println!("devices {d:?}");

    let mut dev = api.get_device()?;
    dev.open()?;
    dev.set("device/receiverchannel", "Rx1")?;
    dev.set("device/outputformat", "spectra")?;
    dev.set("device/receiverclock", "92MHz")?;
    dev.set("device/fft0/fftmergemode", "max")?;
    dev.set("device/fft0/fftaggregate", "100")?;
    dev.set("main/centerfreq", "810e6")?;
    dev.set("main/reflevel", "-20")?;
    dev.connect()?;
    dev.start()?;

    let s = rx(&mut dev)?;

    dev.stop()?;
    dev.disconnect()?;
    dev.close()?;

    plot(&s);

    Ok(())
}

fn rx(dev: &mut Device) -> Result<Vec<f32>, aaronia_rtsa::Error> {
    let p = dev.packet(2)?;
    let cur = Vec::from(p.spectrum());
    dev.consume(2)?;
    Ok(cur)
}

fn plot(s: &Vec<f32>) {
    use gnuplot::*;

    let mut fg = Figure::new();
    fg.axes2d().set_title("Spectrum", &[]).lines(
        0..s.len(),
        s.iter(),
        &[LineWidth(3.0), Color("blue"), LineStyle(DotDash)],
    );
    fg.show().unwrap();
}
