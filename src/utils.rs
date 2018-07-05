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
use clap;

/* standard use */

#[derive(Debug)]
pub enum Format {
    Paf,
    Mhap,
    Fasta,
    Fastq,
}

pub fn get_mapping_format(m: &clap::ArgMatches) -> Option<Format> {
    if m.is_present("format") {
        return match m.value_of("format").unwrap() {
            "paf" => Some(Format::Paf),
            "mhap" => Some(Format::Mhap),
            _ => None,
        };
    }

    return get_format(m.value_of("input").unwrap());
}

pub fn get_format(filename: &str) -> Option<Format> {
    return if filename.contains(".paf") {
        Some(Format::Paf)
    }
    else if filename.contains(".mhap") {
        Some(Format::Mhap)
    }
    else if filename.contains(".fasta") {
        Some(Format::Fasta)
    }
    else if filename.contains(".fastq") {
        Some(Format::Fastq)
    }
    else {
        None
    };
}
