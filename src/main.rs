use std::{
    error::Error,
    fs::File,
    io::{self, stdin, stdout, BufReader, BufWriter, Read, Write},
};

use argh::FromArgs;
use monochrome_quadtree::enc::{video::{VideoEncoder, EncoderV1, EncoderV2}, QuadTree};

#[derive(FromArgs)]
/// Encode one or more frames using linear quadtrees
struct Encode {
    #[argh(option, short = 'v', default = "1")]
    /// tree wire format version
    version: u8,
    #[argh(subcommand)]
    subs: SubCommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum SubCommands {
    Frame(Frame),
    Sequence(Sequence),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "frame")]
/// Compress a single frame
struct Frame {
    #[argh(option, short = 'i', default = "String::from(\"-\")")]
    /// input file, defaults to standard input
    input: String,
    #[argh(option, short = 'o', default = "String::from(\"-\")")]
    /// output file, defaults to standard output
    output: String,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "sequence")]
/// Compress multiple contiguous frames
struct Sequence {
    #[argh(option, short = 'i', default = "String::from(\"-\")")]
    /// input file, defaults to standard input
    input: String,
    #[argh(option, short = 'o', default = "String::from(\"-\")")]
    /// output file, defaults to standard output
    output: String,
    #[argh(option, short = 'f')]
    /// number of frames to process
    frames: Option<u32>,
    #[argh(option, short = 'k', default = "60")]
    /// inclusive interval between I-frames
    i_frame_interval: u16,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Encode = argh::from_env();

    match args.subs {
        SubCommands::Frame(s) => frame(s, args.version),
        SubCommands::Sequence(s) => sequence(s, args.version),
    }
    .map_err(|e| e.into())
}

fn match_input(i: &str) -> Box<dyn Read> {
    match i {
        // TODO: check if stdio is a tty
        "-" => Box::new(BufReader::new(stdin())),
        _ => Box::new(BufReader::new(File::open(i).unwrap())),
    }
}

fn match_output(i: &str) -> Box<dyn Write> {
    match i {
        // TODO: check if stdio is a tty
        "-" => Box::new(BufWriter::new(stdout())),
        _ => Box::new(BufWriter::new(File::create(i).unwrap())),
    }
}

fn frame(args: Frame, version: u8) -> io::Result<()> {
    let mut input = match_input(&args.input);
    let mut output = match_output(&args.output);

    let mut buf = [0; 1024];
    input.read_exact(&mut buf)?;

    match version {
        1 => { QuadTree::from_128x64(&buf, true).store_packed(output)?; },
        2 => { output.write_all(QuadTree::from_128x64(&buf, false).collect_compact().unwrap().as_raw_slice())?; },
        _ => panic!("Invalid format version, valid versions are 1 and 2"),
    }
    Ok(())
}

fn sequence(args: Sequence, version: u8) -> io::Result<()> {
    let input = match_input(&args.input);
    let output = match_output(&args.output);

    match version {
        1 => seq_v1(output, input, args),
        2 => seq_v2(output, input, args),
        _ => panic!("Invalid format version, valid versions are 1 and 2"),
    }
}

fn seq_v1(output: Box<dyn Write>, mut input: Box<dyn Read>, args: Sequence) -> io::Result<()> {
    let mut enc = VideoEncoder::<_, EncoderV1>::new(output, args.i_frame_interval);
    Ok(if let Some(f) = args.frames {
        io::copy(&mut input.take(f as u64 * 1024), &mut enc)?;
    } else {
        io::copy(&mut input, &mut enc)?;
    })
}

fn seq_v2(output: Box<dyn Write>, mut input: Box<dyn Read>, args: Sequence) -> io::Result<()> {
    let mut enc = VideoEncoder::<_, EncoderV2>::new(output, args.i_frame_interval);
    Ok(if let Some(f) = args.frames {
        io::copy(&mut input.take(f as u64 * 1024), &mut enc)?;
    } else {
        io::copy(&mut input, &mut enc)?;
    })
}
