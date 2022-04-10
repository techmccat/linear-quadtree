use super::*;

const M: u8 = u8::MAX;

#[rustfmt::skip]
pub const BUF: [u8; 1024] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, M,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0f,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0f,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0f,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0f,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0b0000_0011,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0b0000_0011,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0b0000_0001,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// can't make this constant because stuff
fn expected_leaves() -> [Leaf; 5] {
    [
        Leaf::new(LeafData::Feature(true), heapless::Vec::from_slice(&[1, 1]).unwrap()),
        Leaf::new(LeafData::Feature(true), heapless::Vec::from_slice(&[1, 3, 1]).unwrap()),
        Leaf::new(LeafData::Feature(true), heapless::Vec::from_slice(&[1, 3, 3, 1]).unwrap()),
        Leaf::new(LeafData::Feature(true), heapless::Vec::from_slice(&[1, 3, 3, 3, 1]).unwrap()),
        Leaf::new(
            LeafData::Bitmap([0b0011_0011, 0b0001_0000]),
            heapless::Vec::from_slice(&[1, 3, 3, 3, 3]).unwrap(),
        ),
    ]
}

#[rustfmt::skip]
pub const EXPECTED_BYTES: &[u8] = &[
    1,
    0b1_010_01_01,
    0b1_011_01_11, 0b01_00_00_00,
    0b1_100_01_11, 0b11_01_00_00,
    0b1_101_01_11, 0b11_11_01_00,
    0b1_110_01_11, 0b11_11_11_00,
    0b0011_0011,   0b0001_0000
];

#[test]
fn full() {
    let buf = [u8::MAX; 128 * 64 / 8];
    let mut out = Vec::with_capacity(1);

    let mut tree = LinearQuadTree::new();
    tree.parse_12864(&buf);

    assert_eq!(
        tree.0,
        [Leaf::new(LeafData::Feature(true), heapless::Vec::from_slice(&[]).unwrap())]
    );

    tree.store_packed(&mut out).unwrap();
    assert_eq!(out, [0])
}

#[test]
fn empty() {
    let buf = [0; 128 * 64 / 8];
    let mut out = Vec::new();

    let mut tree = LinearQuadTree::new();
    tree.parse_12864(&buf);

    assert_eq!(
        tree.0,
        [Leaf::new(LeafData::Feature(false), heapless::Vec::from_slice(&[]).unwrap())]
    );

    tree.store_packed(&mut out).unwrap();
    assert_eq!(out, [1])
}

#[test]
fn stairs() {
    let mut out = Vec::with_capacity(EXPECTED_BYTES.len());

    let mut tree = LinearQuadTree::new();
    tree.parse_12864(&BUF);

    let active: Vec<_> = tree.0.clone().into_iter().filter(|l| l.feat_or_data(true)).collect();
    assert_eq!(active, expected_leaves());

    tree.store_packed(&mut out).unwrap();
    assert_eq!(out, EXPECTED_BYTES)
}
