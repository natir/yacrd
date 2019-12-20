/*
Copyright (c) 2019 Pierre Marijon <pmarijon@mpi-inf.mpg.de>

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

/* crate use */
use anyhow::{anyhow, Context, Result};

/* local use */
use error;

#[derive(Debug, PartialEq)]
pub enum FileType {
    Fasta,
    Fastq,
    Yacrd,
    Paf,
    M4,
    YacrdOverlap,
}

pub fn get_file_type(filename: &str) -> Option<FileType> {
    if filename.contains(".m4") || filename.contains("mhap") {
        Some(FileType::M4)
    } else if filename.contains(".paf") {
        Some(FileType::Paf)
    } else if filename.contains(".yacrd") {
        Some(FileType::Yacrd)
    } else if filename.contains(".fastq") || filename.contains(".fq") {
        Some(FileType::Fastq)
    } else if filename.contains(".fasta") || filename.contains(".fa") {
        Some(FileType::Fasta)
    } else {
        None
    }
}

pub fn read_file(filename: &str) -> Result<(Box<dyn std::io::Read>, niffler::compression::Format)> {
    let raw_in = Box::new(std::io::BufReader::new(
        std::fs::File::open(filename).with_context(|| error::Error::CantReadFile {
            filename: filename.to_string(),
        })?,
    ));

    niffler::get_reader(raw_in)
        .with_context(|| anyhow!("Error in compression detection of file {}", filename))
}

pub fn write_file(
    filename: &str,
    compression: niffler::compression::Format,
) -> Result<Box<dyn std::io::Write>> {
    let raw_out = Box::new(std::io::BufWriter::new(
        std::fs::File::create(filename).with_context(|| error::Error::CantWriteFile {
            filename: filename.to_string(),
        })?,
    ));

    let output = niffler::get_writer(raw_out, compression, niffler::compression::Level::One)?;

    Ok(output)
}

pub fn str2usize(val: &str) -> Result<usize> {
    val.parse::<usize>().with_context(|| {
        anyhow!(
            "Error durring parsing of number from string {:?} in usize",
            val
        )
    })
}

pub fn str2u32(val: &str) -> Result<u32> {
    val.parse::<u32>().with_context(|| {
        anyhow!(
            "Error durring parsing of number from string {:?} in u32",
            val
        )
    })
}
