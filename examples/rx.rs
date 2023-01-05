use aaronia_rtsa::version;
use aaronia_rtsa::ApiHandle;
use aaronia_rtsa::Device;
use num_complex::Complex32;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RTSA library version: {}", version());

    let mut api = ApiHandle::new()?;
    api.rescan_devices()?;
    let d = api.devices()?;
    println!("devices {:?}", d);

    let mut dev = api.get_device()?;
    dev.open()?;
    dev.config("device/receiverchannel", "Rx1")?;
    dev.config("device/outputformat", "iq")?;
    dev.config("device/receiverclock", "92MHz")?;
    dev.config("main/decimation", "1 / 64")?;
    dev.connect()?;
    dev.start()?;

    rx(&mut dev)?;

    dev.stop()?;
    dev.disconnect()?;
    dev.close()?;

    Ok(())
}

fn rx(dev: &mut Device) -> Result<(), aaronia_rtsa::Error> {
    const N: usize = 8192;
    let mut samples = [Complex32::new(0.0, 0.0); N];
    let mut i = 0;
    while i < N {
        let p = dev.packet()?;
        let cur = p.samples();
        let n = std::cmp::min(N - i, cur.len());
        samples[i..i + n].copy_from_slice(&cur[0..n]);
        i += n;
        dev.consume()?;
    }

    plot(&mut samples);
    Ok(())
}

fn plot(s: &mut [num_complex::Complex32]) {
    use gnuplot::*;
    let re = s.iter().map(|s| s.re);
    let im = s.iter().map(|s| s.im);

    let mut fg = Figure::new();

    fg.axes2d()
        .set_title("Samples", &[])
        .lines(
            0..s.len(),
            re,
            &[LineWidth(3.0), Color("brown"), LineStyle(DotDash)],
        )
        .lines(
            0..s.len(),
            im,
            &[LineWidth(3.0), Color("blue"), LineStyle(DotDash)],
        );

    fg.show().unwrap();

    let mut planner = rustfft::FftPlanner::new();
    planner.plan_fft_forward(s.len()).process(s);

    let abs = s.iter().map(|s| s.norm_sqr().log10());

    let mut fg = Figure::new();

    fg.axes2d()
        .set_title("Spectrum", &[])
        .lines(
            0..s.len(),
            abs,
            &[LineWidth(3.0), Color("blue"), LineStyle(DotDash)],
        );

    fg.show().unwrap();

}
