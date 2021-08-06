use std::{
    convert::Infallible,
    fs::{read_dir, File},
    io::{Read, Write},
};

use bitvec::prelude::*;
use embedded_graphics::{
    image::ImageRaw, pixelcolor::BinaryColor, prelude::*, primitives::Rectangle,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};

use crate::dec::LeafParser;
use crate::enc::LinearQuadTree;

const WIDTH: u32 = 128;
const HEIGHT: u32 = 64;
const M: u8 = u8::MAX;

#[rustfmt::skip]
const STAIR_BUF: &[u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, 0, 0, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, 0, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, M, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, M, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, M, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0, M, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0x0f, M, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0x0f, M, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0b0011_1111, M, M, M, M, M, M, M, M, M, M, M, M, M, M, M, 
    0b0111_1111, M, M, M, M, M, M, M, M, M, M, M, M, M, M, M
];

#[rustfmt::skip]
const STAIRS_BYTES: &[u8] = &[
    0b00_00_10_10, 0b10_10_10_11,
    0b1_110_00_10, 0b10_10_10_11,
    0b1_101_00_10, 0b10_10_11_00,
    0b1_100_00_10, 0b10_11_00_00,
    0b1_011_00_10, 0b11_00_00_00,
    0b1_010_00_11,
    0b1_001_01_00,
];

#[derive(Default)]
struct DumpableDisplay {
    buf: BitArr!(for 8192, in Msb0, u8),
}

impl OriginDimensions for DumpableDisplay {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl DrawTarget for DumpableDisplay {
    type Color = BinaryColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x, y }, col) in pixels {
            let index = (y as u32 * WIDTH + x as u32) as usize;
            let col = if col.is_on() { 1u8 } else { 0 };
            self.buf[index..=index].store(col);
        }
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let col = if color.is_on() { u128::MAX } else { 0 };
        let &Rectangle {
            top_left: Point { x, y },
            size: Size { width, height },
        } = area;
        let y = y as usize;
        let x = x as usize;
        let height = height as usize;
        let width = width as usize;

        for row_height in y..y + height {
            let base = WIDTH as usize * row_height + x;
            self.buf[base..base + width].store(col);
        }
        Ok(())
    }
}

#[test]
fn enc_then_draw() {
    let mut out = Vec::with_capacity(12);

    let mut tree = LinearQuadTree::new(&mut out);
    tree.parse_slice_12864(STAIR_BUF).unwrap();

    // really just a sanity check
    assert_eq!(STAIRS_BYTES, out.as_slice());

    let mut display = DumpableDisplay::default();

    let dec = LeafParser::new(&out);
    dec.draw(&mut display).unwrap();

    assert_eq!(STAIR_BUF, display.buf.as_raw_slice())
}

#[test]
fn bad_apple() -> std::io::Result<()> {
    let mut read_buf = [0u8; 1024];
    let mut leaf_buf = Vec::with_capacity(500);
    let mut display = DumpableDisplay::default();

    for path in read_dir("test_data/frames")?
        .filter_map(|r| r.ok())
        .filter(|e| e.metadata().unwrap().is_file())
        .map(|e| e.path())
    {
        let mut file = File::open(&path)?;
        file.read_exact(&mut read_buf)?;

        let mut tree = LinearQuadTree::new(&mut leaf_buf);
        tree.parse_slice_12864(&read_buf)?;

        let dec = LeafParser::new(&leaf_buf);
        dec.draw(&mut display).unwrap();

        if read_buf != display.buf.as_raw_slice() {
            let settings = OutputSettingsBuilder::new()
                .theme(embedded_graphics_simulator::BinaryColorTheme::LcdBlue)
                .build();
            let mut dump_display =
                SimulatorDisplay::with_default_color(Size::new(WIDTH, HEIGHT), BinaryColor::Off);

            ImageRaw::new_binary(&read_buf, WIDTH)
                .draw(&mut dump_display)
                .unwrap();

            dump_display
                .to_rgb_output_image(&settings)
                .save_png("test_data/left.png")
                .unwrap();

            dump_display.clear(BinaryColor::Off).unwrap();

            ImageRaw::new_binary(&display.buf.as_raw_slice(), WIDTH)
                .draw(&mut dump_display)
                .unwrap();

            dump_display
                .to_rgb_output_image(&settings)
                .save_png("test_data/right.png")
                .unwrap();

            let mut leaf_dump = File::create("test_data/leaves.txt")?;
            let leaves: Vec<_> = LeafParser::new(&leaf_buf).into_iter().collect();

            write!(&mut leaf_dump, "Leaf dump:\n{:#?}", leaves)?;

            panic!(
                "Decoded image did not match the source file {}",
                path.display()
            )
        }

        display.clear(BinaryColor::Off).unwrap();
        leaf_buf.clear();
    }

    Ok(())
}
