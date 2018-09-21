/*
Copyright (c) 2018 Pierre Marijon <pierre.marijon@inria.fr>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

/* crates use */
use bzip2;
use flate2;
use xz2;
use enum_primitive::FromPrimitive;

/* standard use */
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter};

enum_from_primitive! {
    #[repr(u64)]
    #[derive(Debug, PartialEq)]
    pub enum CompressionFormat {
        Gzip = 0x1F8B,
        Bzip = 0x425A,
        Lzma = 0xFD377A585A,
        No,
    }
}

pub fn get_input(input_name: &str) -> (Box<io::Read>, CompressionFormat) {
    // choose std::io::stdin or open file
    if input_name == "-" {
        return (Box::new(get_readable(input_name)), CompressionFormat::No);
    }

    return get_readable_file(input_name);
}

pub fn get_readable_file(input_name: &str) -> (Box<io::Read>, CompressionFormat) {
    let raw_input = get_readable(input_name);

    // check compression
    let compression = get_compression(raw_input);

    // return readable and compression status
    match compression {
        CompressionFormat::Gzip => (
            Box::new(flate2::read::GzDecoder::new(get_readable(input_name))),
            CompressionFormat::Gzip,
        ),
        CompressionFormat::Bzip => (
            Box::new(bzip2::read::BzDecoder::new(get_readable(input_name))),
            CompressionFormat::Bzip,
        ),
        CompressionFormat::Lzma => (
            Box::new(xz2::read::XzDecoder::new(get_readable(input_name))),
            CompressionFormat::Lzma,
        ),
        CompressionFormat::No => (Box::new(get_readable(input_name)), CompressionFormat::No),
    }
}

pub fn get_readable(input_name: &str) -> Box<io::Read> {
    match input_name {
        "-" => Box::new(BufReader::new(io::stdin())),
        _ => Box::new(BufReader::new(File::open(input_name).expect(&format!(
            "Can't open input file {}",
            input_name
        )))),
    }
}

fn get_compression(mut in_stream: Box<io::Read>) -> CompressionFormat {
    let mut buf = vec![0u8; 5];

    in_stream.read_exact(&mut buf).expect(
        "Error durring reading first bit of file",
    );


    let mut five_bit_val: u64 = 0;
    for i in 0..5 {
        five_bit_val |= (buf[i] as u64) << 8 * (4 - i);
    }

    if CompressionFormat::from_u64(five_bit_val) == Some(CompressionFormat::Lzma) {
        return CompressionFormat::Lzma;
    }

    let mut two_bit_val: u64 = 0;
    for i in 0..2 {
        two_bit_val |= (buf[i] as u64) << 8 * (1 - i);
    }

    match CompressionFormat::from_u64(two_bit_val) {
        e @ Some(CompressionFormat::Gzip) |
        e @ Some(CompressionFormat::Bzip) => e.unwrap(),
        _ => CompressionFormat::No,
    }
}

pub fn get_output(output_name: &str, format: CompressionFormat) -> Box<io::Write> {
    match format {
        CompressionFormat::Gzip => Box::new(flate2::write::GzEncoder::new(
            get_writable(output_name),
            flate2::Compression::best(),
        )),
        CompressionFormat::Bzip => Box::new(bzip2::write::BzEncoder::new(
            get_writable(output_name),
            bzip2::Compression::Best,
        )),
        CompressionFormat::Lzma => {
            Box::new(xz2::write::XzEncoder::new(get_writable(output_name), 9))
        }
        CompressionFormat::No => Box::new(get_writable(output_name)),
    }
}

pub fn choose_compression(
    input_compression: CompressionFormat,
    compression_set: bool,
    compression_value: &str,
) -> CompressionFormat {
    if !compression_set {
        return input_compression;
    }

    match compression_value {
        "gzip" => CompressionFormat::Gzip,
        "bzip2" => CompressionFormat::Bzip,
        "lzma" => CompressionFormat::Lzma,
        _ => CompressionFormat::No,
    }
}

fn get_writable(output_name: &str) -> Box<io::Write> {
    match output_name {
        "-" => Box::new(BufWriter::new(io::stdout())),
        _ => Box::new(BufWriter::new(File::create(output_name).expect(&format!(
            "Can't open output file {}",
            output_name
        )))),
    }
}

#[cfg(test)]
mod test {

    use super::*;

    const GZIP_FILE: &'static [u8] = &[0o037, 0o213, 0o0, 0o0, 0o0];
    const BZIP_FILE: &'static [u8] = &[0o102, 0o132, 0o0, 0o0, 0o0];
    const LZMA_FILE: &'static [u8] = &[0o375, 0o067, 0o172, 0o130, 0o132];

    #[test]
    fn compression_from_file() {
        assert_eq!(
            get_compression(Box::new(GZIP_FILE)),
            CompressionFormat::Gzip
        );
        assert_eq!(
            get_compression(Box::new(BZIP_FILE)),
            CompressionFormat::Bzip
        );
        assert_eq!(
            get_compression(Box::new(LZMA_FILE)),
            CompressionFormat::Lzma
        );
    }

    #[test]
    fn compression_from_input_or_cli() {
        assert_eq!(
            choose_compression(CompressionFormat::Gzip, false, "_"),
            CompressionFormat::Gzip
        );
        assert_eq!(
            choose_compression(CompressionFormat::Bzip, false, "_"),
            CompressionFormat::Bzip
        );
        assert_eq!(
            choose_compression(CompressionFormat::Lzma, false, "_"),
            CompressionFormat::Lzma
        );
        assert_eq!(
            choose_compression(CompressionFormat::No, true, "gzip"),
            CompressionFormat::Gzip
        );
        assert_eq!(
            choose_compression(CompressionFormat::No, true, "bzip2"),
            CompressionFormat::Bzip
        );
        assert_eq!(
            choose_compression(CompressionFormat::No, true, "lzma"),
            CompressionFormat::Lzma
        );
    }
}
