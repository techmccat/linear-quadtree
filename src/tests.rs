use std::{
    convert::{Infallible, TryFrom},
    fs::File,
    io::{Read, Write},
};

use bitvec::prelude::*;
use embedded_graphics::{
    image::ImageRaw, pixelcolor::BinaryColor, prelude::*, primitives::Rectangle,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};

use crate::{
    dec::{video::VideoSlice, LeafParser},
    enc::{
        tests::{BUF, EXPECTED_BYTES},
        video::VideoEncoder,
        QuadTree,
    },
    FrameMeta,
};

const WIDTH: u32 = 128;
const HEIGHT: u32 = 64;

#[derive(Default)]
struct DumpableDisplay {
    buf: BitArr!(for 8192, in u8, Msb0),
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
        let bb = self.bounding_box();
        for Pixel(Point { x, y }, col) in pixels.into_iter().filter(|p| bb.contains(p.0)) {
            let index = (y as u32 * WIDTH + x as u32) as usize;
            let col = if col.is_on() { 1u8 } else { 0 };
            self.buf[index..=index].store(col);
        }
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let col = if color.is_on() { u128::MAX } else { 0 };
        let Rectangle {
            top_left: Point { x, y },
            size: Size { width, height },
        } = area.intersection(&self.bounding_box());
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
fn framemeta_pack_then_parse() {
    let foo = [true, false];
    for a in foo {
        for b in foo {
            for c in foo {
                dbg!(a, b, c);
                let left = FrameMeta::new(a, b, c);
                let int: u8 = left.into();
                let right = FrameMeta::try_from(int).unwrap();
                assert_eq!(left, right)
            }
        }
    }
}

#[test]
fn enc_then_draw() {
    let mut out = Vec::with_capacity(12);

    let tree = QuadTree::from_128x64(&BUF);
    tree.store_packed(&mut out).unwrap();

    // really just a sanity check
    assert_eq!(EXPECTED_BYTES, out.as_slice());

    let mut display = DumpableDisplay::default();

    let dec = LeafParser::new(&out).unwrap();
    dec.draw(&mut display).unwrap();

    assert_eq!(BUF, display.buf.as_raw_slice())
}

#[test]
fn bad_apple() -> std::io::Result<()> {
    let input = {
        let mut buf = Vec::new();
        File::open("test_data/frames.bin")?.read_to_end(&mut buf)?;
        buf
    };

    let mut output = Vec::with_capacity(input.len() / 2);
    let mut enc = VideoEncoder::new(&mut output, 1);
    enc.write_all(&input)?;

    compare_original_and_encoded(&input, &output);
    Ok(())
}

#[test]
fn bad_apple_pframes() -> std::io::Result<()> {
    let input = {
        let mut buf = Vec::new();
        File::open("test_data/frames.bin")?.read_to_end(&mut buf)?;
        buf
    };

    let mut output = Vec::with_capacity(input.len() / 2);
    let mut enc = VideoEncoder::new(&mut output, 60);
    enc.write_all(&input)?;

    compare_original_and_encoded(&input, &output);
    Ok(())
}

fn compare_original_and_encoded(original: &[u8], encoded: &[u8]) {
    let mut display = DumpableDisplay::default();
    let bitmaps = original.chunks(1024).take_while(|c| c.len() == 1024);
    let mut enc_iter = VideoSlice::new(&encoded);
    // let mut old_tree = None;

    for (i, frame) in bitmaps.enumerate() {
        // dbg!(i);
        let mut last_leaves = [None, None];

        // if i == 363 {
        //     old_tree = Some(QuadTree::from_128x64(frame.try_into().unwrap()));
        // }
        // if i == 364 {
        //     dump_to_image(&frame, "test_data/364.png");
        //     let tree = QuadTree::from_128x64(frame.try_into().unwrap());
        //     let old_tree = old_tree.take().unwrap();
        //     let diff = tree.diff(&old_tree);
        //     dbg!(old_tree, tree, diff);
        // }

        for leaves in enc_iter.by_ref() {
            leaves.draw(&mut display).unwrap();

            let stop = leaves.flush_after();
            if stop {
                last_leaves[1] = Some(leaves);
                break;
            } else {
                last_leaves[0] = Some(leaves)
            }
        }

        if frame != display.buf.as_raw_slice() {
            dump_to_image(&frame, "test_data/left.png");
            dump_to_image(&display.buf.as_raw_slice(), "test_data/right.png");

            let mut leaf_dump = File::create("test_data/leaves.txt").unwrap();
            let leaves: Vec<_> = last_leaves.iter().flatten().flatten().collect();
            write!(&mut leaf_dump, "Leaf dump:\n{leaves:#?}").unwrap();

            panic!("Decoded image did not match the source frame {}", i)
        }
    }
}

fn dump_to_image(buf: &[u8], path: &str) {
    let settings = OutputSettingsBuilder::new()
        .theme(embedded_graphics_simulator::BinaryColorTheme::LcdBlue)
        .build();
    let mut dump_display =
        SimulatorDisplay::with_default_color(Size::new(WIDTH, HEIGHT), BinaryColor::Off);

    ImageRaw::new_binary(buf, WIDTH)
        .draw(&mut dump_display)
        .unwrap();

    dump_display
        .to_rgb_output_image(&settings)
        .save_png(path)
        .unwrap();
}
