use embedded_graphics::{
    image::Image,
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use monochrome_quadtree::dec::{video::VideoSlice, LeafParserV1, LeafParserV2, Decoder};
use std::{
    env::args,
    error::Error,
    fs::File,
    io::{BufReader, Read},
    thread::sleep,
    time::{Duration, Instant},
};

//const W: u32 = 128;
//const H: u32 = 64;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = args().skip(1);
    let mode = args.next().unwrap();
    let format = args.next().unwrap();
    let file = args.next().unwrap();

    let display = SimulatorDisplay::<BinaryColor>::new(Size::new(128, 64));

    let settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledWhite)
        .build();
    let window = Window::new("", &settings);

    let mut source = BufReader::new(File::open(file)?);

    let version = match format.as_str() {
        "-1" => 1,
        "-2" => 2,
        _ => panic!("Invalid format version")
    };

    let mut data = Vec::new();
    source.read_to_end(&mut data)?;

    match mode.as_str() {
        "-v" | "--video" => if version == 1 {
            video::<LeafParserV1>(display, window, &data)
        } else {
            video::<LeafParserV2>(display, window, &data)
        },
        "-i" | "--image" => if version == 1 { 
            img::<LeafParserV1>(display, window, &data)
        } else {
            img::<LeafParserV2>(display, window, &data)
        },
        _ => panic!("Wrong argument"),
    }
}

fn img<'a, D: Decoder<'a>>(
    mut display: SimulatorDisplay<BinaryColor>,
    mut window: Window,
    data: &'a[u8]
) -> Result<(), Box<dyn Error>> {
    let raw = D::from_buf(&data).unwrap().drawable();
    let img = Image::new(&raw, Point::zero());

    img.draw(&mut display)?;
    window.show_static(&display);

    Ok(())
}

fn video<'a, D: Decoder<'a>>(
    mut display: SimulatorDisplay<BinaryColor>,
    mut window: Window,
    data: &'a [u8]
) -> Result<(), Box<dyn Error>> {
    window.update(&display);

    let mut frame = 0u64;
    let started = Instant::now();

    let iter = VideoSlice::<D>::new(&data);

    for i in iter {
        for event in window.events() {
            if let SimulatorEvent::Quit = event {
                return Ok(());
            }
        }

        let raw = i.drawable();
        let img = Image::new(&raw, Point::zero());

        img.draw(&mut display)?;
        window.update(&display);

        frame += 1;
        let elapsed = started.elapsed().as_millis() as u64;
        let expected = 1000 / 30 * frame;
        let sleep_ms = expected.checked_sub(elapsed).unwrap_or(0);

        sleep(Duration::from_millis(sleep_ms));
    }

    Ok(())
}
