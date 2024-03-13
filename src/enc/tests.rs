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
        Leaf::new(
            LeafData::Feature(true),
            heapless::Vec::from_slice(&[1, 1]).unwrap(),
        ),
        Leaf::new(
            LeafData::Feature(true),
            heapless::Vec::from_slice(&[1, 3, 1]).unwrap(),
        ),
        Leaf::new(
            LeafData::Feature(true),
            heapless::Vec::from_slice(&[1, 3, 3, 1]).unwrap(),
        ),
        Leaf::new(
            LeafData::Feature(true),
            heapless::Vec::from_slice(&[1, 3, 3, 3, 1]).unwrap(),
        ),
        Leaf::new(
            LeafData::Bitmap([0b0011_0011, 0b0001_0000]),
            heapless::Vec::from_slice(&[1, 3, 3, 3, 3]).unwrap(),
        ),
    ]
}

#[rustfmt::skip]
pub const EXPECTED_BYTES_LINEAR: &[u8] = &[
    0b101,
    0b1_010_01_01,
    0b1_011_01_11, 0b01_00_00_00,
    0b1_100_01_11, 0b11_01_00_00,
    0b1_101_01_11, 0b11_11_01_00,
    0b1_110_01_11, 0b11_11_11_00,
    0b0011_0011,   0b0001_0000
];

pub const EXPECTED_BYTES_COMPACT: &[u8] = &[
    0b01_10_01_10,
    0b11_10_01_10,
    0b11_10_01_10,
    0b11_10_01_10,
    0b11_10_01_10,
    0b11_10_01_10,
    0b11_10_10_00,
    0b00 << 6,
];

#[test]
fn full() {
    let buf = [u8::MAX; 128 * 64 / 8];
    let mut out = Vec::with_capacity(1);

    let tree = QuadTree::from_128x64(&buf, true);

    assert_eq!(
        tree.leaves().collect::<Vec<_>>(),
        [Leaf::new(
            LeafData::Feature(true),
            heapless::Vec::from_slice(&[]).unwrap()
        )]
    );
    assert_eq!(
        tree.nodes().collect::<Vec<_>>(),
        [&Node::Leaf(LeafData::Feature(true))]
    );

    tree.store_packed(&mut out).unwrap();
    assert_eq!(out, [0b100]);

    let compact = tree.collect_compact().unwrap();
    assert_eq!(compact.as_raw_slice(), [0b11_00_00_00]);
}

#[test]
fn empty() {
    let buf = [0; 128 * 64 / 8];
    let mut out = Vec::new();

    let tree = QuadTree::from_128x64(&buf, true);

    assert_eq!(
        tree.leaves().collect::<Vec<_>>(),
        [Leaf::new(
            LeafData::Feature(false),
            heapless::Vec::from_slice(&[]).unwrap()
        )]
    );
    assert_eq!(
        tree.nodes().collect::<Vec<_>>(),
        [&Node::Leaf(LeafData::Feature(false))]
    );

    tree.store_packed(&mut out).unwrap();
    assert_eq!(out, [0b101]);

    let compact = tree.collect_compact().unwrap();
    assert_eq!(compact.as_raw_slice(), [0b10_00_00_00]);
}

#[test]
fn stairs_v1() {
    let mut out = Vec::with_capacity(EXPECTED_BYTES_LINEAR.len());

    let tree = QuadTree::from_128x64(&BUF, true);

    let active: Vec<_> = tree.leaves().filter(|l| l.feat_or_data(true)).collect();
    assert_eq!(active, expected_leaves());

    tree.store_packed(&mut out).unwrap();
    assert_eq!(out, EXPECTED_BYTES_LINEAR);
}

#[test]
fn stairs_v2() {
    let tree = QuadTree::from_128x64(&BUF, false);
    let compact = tree.collect_compact().unwrap();
    assert_eq!(compact.as_raw_slice(), EXPECTED_BYTES_COMPACT);
}

#[test]
fn traverse() {
    let tree = QuadTree {
        head: Node::Branch(Box::new([
            Node::Branch(Box::new([
                Node::Leaf(LeafData::Feature(true)),
                Node::Leaf(LeafData::Feature(false)),
                Node::Empty,
                Node::Empty,
            ])),
            Node::Empty,
            Node::Leaf(LeafData::Feature(true)),
            Node::Leaf(LeafData::Feature(false)),
        ])),
    };
    let nodes: Vec<_> = tree.nodes().collect();

    assert_eq!(
        nodes,
        [
            &tree.head,
            &tree.head.children().unwrap()[0],
            &tree.head.children().unwrap()[0].children().unwrap()[0],
            &tree.head.children().unwrap()[0].children().unwrap()[1],
            &tree.head.children().unwrap()[0].children().unwrap()[2],
            &tree.head.children().unwrap()[0].children().unwrap()[3],
            &tree.head.children().unwrap()[1],
            &tree.head.children().unwrap()[2],
            &tree.head.children().unwrap()[3],
        ]
    )
}

#[test]
fn diff_same() {
    let tree = QuadTree {
        head: Node::Branch(Box::new([
            Node::Branch(Box::new([
                Node::Leaf(LeafData::Feature(true)),
                Node::Leaf(LeafData::Feature(false)),
                Node::Empty,
                Node::Empty,
            ])),
            Node::Empty,
            Node::Leaf(LeafData::Feature(true)),
            Node::Leaf(LeafData::Feature(false)),
        ])),
    };
    let diff = tree.diff(&tree);

    assert_eq!(diff.head, Node::Empty);
}

#[test]
fn diff() {
    let old = QuadTree {
        head: Node::Branch(Box::new([
            Node::Empty,
            Node::Leaf(LeafData::Feature(false)),
            Node::Leaf(LeafData::Feature(true)),
            Node::Leaf(LeafData::Feature(true)),
        ])),
    };
    let new = QuadTree {
        head: Node::Branch(Box::new([
            Node::Branch(Box::new([
                Node::Leaf(LeafData::Feature(true)),
                Node::Leaf(LeafData::Feature(false)),
                Node::Empty,
                Node::Empty,
            ])),
            Node::Empty,
            Node::Leaf(LeafData::Feature(true)),
            Node::Leaf(LeafData::Feature(false)),
        ])),
    };

    let diff = new.diff(&old);

    let expected = Node::Branch(Box::new([
        Node::Branch(Box::new([
            Node::Leaf(LeafData::Feature(true)),
            Node::Leaf(LeafData::Feature(false)),
            Node::Empty,
            Node::Empty,
        ])),
        Node::Empty,
        Node::Empty,
        Node::Leaf(LeafData::Feature(false)),
    ]));

    assert_eq!(diff.head, expected);
}
