use super::*;

#[test]
fn full() {
    let buf = [u8::MAX; 128 * 64];
    let mut out = Vec::with_capacity(1);

    let mut tree = LinearQuadTree::new(&mut out);
    tree.parse_slice_12864(&buf).unwrap();

    assert_eq!(out, [0b1_000_00_00])
}

#[test]
fn empty() {
    let buf = [0; 128 * 64];
    let mut out = Vec::new();

    let mut tree = LinearQuadTree::new(&mut out);
    tree.parse_slice_12864(&buf).unwrap();

    assert_eq!(out, [])
}

#[test]
fn stairs() {
    let m = u8::MAX;
    let buf = [
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, 0, 0, 0, 0, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, 0, 0, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, 0, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, m, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, m, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, m, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0, m, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0x0f, m, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0x0f, m, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0b0011_1111, m, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
        0b0111_1111, m, m, m, m, m, m, m, m, m, m, m, m, m, m, m, 
    ];

    let expected: [u8; 12] = [
        0b00_00_10_10, 0b10_10_10_11,
        0b1_110_00_10, 0b10_10_10_11,
        0b1_101_00_10, 0b10_10_11_00,
        0b1_100_00_10, 0b10_11_00_00,
        0b1_011_00_10, 0b11_00_00_00,
        0b1_010_00_11,
        0b1_001_01_00,
    ];

    let mut out = Vec::with_capacity(12);

    let mut tree = LinearQuadTree::new(&mut out);
    tree.parse_slice_12864(&buf).unwrap();

    assert_eq!(expected, out.as_slice())
}
