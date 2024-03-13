use crate::{
    dec::{Decoder, LeafParserV2},
    enc::{
        tests::EXPECTED_BYTES_COMPACT,
        video::{Encode, EncoderV2},
    },
};
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
    dec::{video::VideoSlice, LeafParserV1},
    enc::{
        tests::{BUF, EXPECTED_BYTES_LINEAR},
        video::{EncoderV1, VideoEncoder},
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
                let left = FrameMeta::new(a, b, c);
                let int: u8 = left.into();
                let right = FrameMeta::try_from(int).unwrap();
                assert_eq!(left, right)
            }
        }
    }
}

#[test]
fn enc_then_draw_v1() {
    let mut out = Vec::with_capacity(12);

    let tree = QuadTree::from_128x64(&BUF, true);
    tree.store_packed(&mut out).unwrap();

    // really just a sanity check
    assert_eq!(EXPECTED_BYTES_LINEAR, out.as_slice());

    let mut display = DumpableDisplay::default();

    let dec = LeafParserV1::new(&out).unwrap();
    dec.drawable().draw(&mut display).unwrap();

    assert_eq!(BUF, display.buf.as_raw_slice())
}

#[test]
fn enc_then_draw_v2() {
    let tree = QuadTree::from_128x64(&BUF, false);
    let compact = tree.collect_compact().unwrap();

    assert_eq!(EXPECTED_BYTES_COMPACT, compact.as_raw_slice());

    let mut display = DumpableDisplay::default();
    let dec = LeafParserV2::from_buf(compact.as_raw_slice()).unwrap();
    dec.drawable().draw(&mut display).unwrap();

    assert_eq!(BUF, display.buf.as_raw_slice())
}

fn read_test_frames() -> Vec<u8> {
    let mut buf = Vec::new();
    File::open("test_data/frames.bin")
        .unwrap()
        .read_to_end(&mut buf)
        .unwrap();
    buf
}
fn encode_test_frames<E: Encode>(iframe_int: u16) -> (Vec<u8>, Vec<u8>) {
    let input = read_test_frames();
    let mut output = Vec::with_capacity(input.len() / 2);
    let mut enc = VideoEncoder::<_, E>::new(&mut output, iframe_int);
    enc.write_all(&input).unwrap();
    (input, output)
}

#[test]
fn bad_apple_v2() {
    let (input, output) = encode_test_frames::<EncoderV2>(60);
    compare_original_and_encoded::<LeafParserV2>(&input, &output);
}

#[test]
fn bad_apple_v1() {
    let (input, output) = encode_test_frames::<EncoderV1>(1);
    compare_original_and_encoded::<LeafParserV1>(&input, &output);
}

#[test]
fn bad_apple_pframes_v1() {
    let (input, output) = encode_test_frames::<EncoderV1>(60);
    compare_original_and_encoded::<LeafParserV1>(&input, &output);
}

fn compare_original_and_encoded<'a, D: Decoder<'a> + Clone + core::fmt::Debug>(
    original: &[u8],
    encoded: &'a [u8],
) {
    let mut display = DumpableDisplay::default();
    let bitmaps = original.chunks(1024).take_while(|c| c.len() == 1024);
    let mut enc_iter: VideoSlice<D> = VideoSlice::new(&encoded);

    for (i, frame) in bitmaps.enumerate() {
        let mut last_leaves = [None, None];

        for leaves in enc_iter.by_ref() {
            leaves.clone().drawable().draw(&mut display).unwrap();

            if leaves.flush_after() {
                last_leaves[1] = Some(leaves);
                break;
            } else {
                last_leaves[0] = Some(leaves)
            }
        }

        if frame != display.buf.as_raw_slice() {
            dump_to_image(&frame, "test_data/orig.png");
            dump_to_image(&display.buf.as_raw_slice(), "test_data/decoded.png");

            let mut leaf_dump = File::create("test_data/leaves.txt").unwrap();
            let leaves: Vec<_> = last_leaves
                .iter()
                .flatten()
                .map(Decoder::iter)
                .flatten()
                .collect();
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
