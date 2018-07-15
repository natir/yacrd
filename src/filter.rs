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

/* project use */
use file;
use utils;
use overlap_format;

/* crates use */
use bio;
use csv;

/* standard use */
use std::io;
use std::collections::{HashSet};

pub fn run(reads: &Box<HashSet<String>>, filename: &str, filterd_suffix: &str) {
    let filterd_name = &generate_filterd_name(filename.to_owned(), filterd_suffix);
 
    let (raw_input, compression) = file::get_readable_file(filename);
    let input = Box::new(raw_input);
    let output = Box::new(file::get_output(filterd_name, compression));

    match utils::get_format(filename).unwrap() {
        utils::Format::Paf => filterd_paf(reads, input, output),
        utils::Format::Mhap => filterd_mhap(reads, input, output),
        utils::Format::Fasta => filterd_fasta(reads, input, output),
        utils::Format::Fastq => filterd_fastq(reads, input, output),
    }
}

fn generate_filterd_name(filename: String, filterd_suffix: &str) -> String {
    return filename.replacen(".", &format!("{}.", filterd_suffix), 1);
}

fn filterd_paf(reads: &Box<HashSet<String>>, input: Box<io::Read>, output: Box<io::Write>) {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(input);

    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_writer(output);

    for result in reader.deserialize::<overlap_format::PafRecord>() {
        let record = result.unwrap();
        if !(reads.contains(&record.read_a) || reads.contains(&record.read_b)) {
            writer.serialize(record).unwrap();
        }
    }
}

fn filterd_mhap(reads: &Box<HashSet<String>>, input: Box<io::Read>, output: Box<io::Write>) {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b' ')
        .has_headers(false)
        .from_reader(input);

    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_writer(output);

    for result in reader.deserialize::<overlap_format::MhapRecord>() {
        let record = result.unwrap();
        if !(reads.contains(&record.read_a) || reads.contains(&record.read_b)) {
            writer.serialize(record).unwrap();
        }
    }
}

fn filterd_fasta(reads: &Box<HashSet<String>>, input: Box<io::Read>, output: Box<io::Write>) {
    let reader = bio::io::fasta::Reader::new(input);
    let mut writer = bio::io::fasta::Writer::new(output);

    for r in reader.records() {
        let record = r.expect("Trouble in fasta parsing process");
        if !reads.contains(record.id()) {
            writer.write_record(&record).expect("Trouble durring fasta valid sequence writing");
        }
    }
}

fn filterd_fastq(reads: &Box<HashSet<String>>, input: Box<io::Read>, output: Box<io::Write>) {
    let reader = bio::io::fastq::Reader::new(input);
    let mut writer = bio::io::fastq::Writer::new(output);

    for r in reader.records() {
        let record = r.expect("Trouble in fastq parsing process");
        if !reads.contains(record.id()) {
            writer.write_record(&record).expect("Trouble durring fasta valid sequence writing");
        }
    }

}
