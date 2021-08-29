use std::{error::Error, fs::File, io::{self, BufReader, BufWriter, Read, Write, stdin, stdout}};

use argh::FromArgs;
use linear_quadtree::enc::{LinearQuadTree, video::VideoEncoder};

#[derive(FromArgs)]
/// Encode one or more frames using linear quadtrees
struct Encode {
    #[argh(subcommand)]
    subs: SubCommands
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
}

fn main() -> Result<(), Box<dyn Error>>{
    let args: Encode = argh::from_env();

    match args.subs {
        SubCommands::Frame(s) => frame(s),
        SubCommands::Sequence(s) => sequence(s),
    }.map_err(|e| e.into())
}

fn match_input(i: String) -> Box<dyn Read> {
    match i.as_str() {
        // TODO: check if stdio is a tty
        "-" => Box::new(BufReader::new(stdin())),
        _ => Box::new(BufReader::new(File::open(i).unwrap()))
    }
}

fn match_output(i: String) -> Box<dyn Write> {
    match i.as_str() {
        // TODO: check if stdio is a tty
        "-" => Box::new(BufWriter::new(stdout())),
        _ => Box::new(BufWriter::new(File::create(i).unwrap()))
    }
}

fn frame(args: Frame) -> io::Result<()> {
    let mut input = match_input(args.input);
    let output = match_output(args.output);

    let mut buf = [0; 1024];
    input.read_exact(&mut buf)?;

    let mut enc = LinearQuadTree::new();
    enc.parse_12864(&buf);
    enc.store_packed(output)?;

    Ok(())
}

fn sequence(args: Sequence)  -> io::Result<()> {
    let mut input = match_input(args.input);
    let output = match_output(args.output);

    let mut enc = VideoEncoder::new(output);
    if let Some(f) = args.frames {
        io::copy(&mut input.take(f as u64 * 1024), &mut enc)?;
    } else {
        io::copy(&mut input, &mut enc)?;
    }

    Ok(())
}
