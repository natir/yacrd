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
use io;
use utils;

/* crates use */
use bio;

/* standard use */
use std;
use std::collections::HashSet;

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

fn filterd_paf(
    reads: &Box<HashSet<String>>,
    input: Box<std::io::Read>,
    output: Box<std::io::Write>,
) {
    let mut reader = io::paf::Reader::new(input);
    let mut writer = io::paf::Writer::new(output);

    for result in reader.records() {
        let record = result.unwrap();
        if !(reads.contains(&record.read_a) || reads.contains(&record.read_b)) {
            writer.write(&record).unwrap();
        }
    }
}

fn filterd_mhap(
    reads: &Box<HashSet<String>>,
    input: Box<std::io::Read>,
    output: Box<std::io::Write>,
) {
    let mut reader = io::mhap::Reader::new(input);
    let mut writer = io::mhap::Writer::new(output);

    for result in reader.records() {
        let record = result.unwrap();
        if !(reads.contains(&record.read_a) || reads.contains(&record.read_b)) {
            writer.write(&record).unwrap();
        }
    }
}

fn filterd_fasta(
    reads: &Box<HashSet<String>>,
    input: Box<std::io::Read>,
    output: Box<std::io::Write>,
) {
    let reader = bio::io::fasta::Reader::new(input);
    let mut writer = bio::io::fasta::Writer::new(output);

    for r in reader.records() {
        let record = r.expect("Trouble in fasta parsing process");
        if !reads.contains(record.id()) {
            writer
                .write_record(&record)
                .expect("Trouble durring fasta valid sequence writing");
        }
    }
}

fn filterd_fastq(
    reads: &Box<HashSet<String>>,
    input: Box<std::io::Read>,
    output: Box<std::io::Write>,
) {
    let reader = bio::io::fastq::Reader::new(input);
    let mut writer = bio::io::fastq::Writer::new(output);

    for r in reader.records() {
        let record = r.expect("Trouble in fastq parsing process");
        if !reads.contains(record.id()) {
            writer
                .write_record(&record)
                .expect("Trouble durring fasta valid sequence writing");
        }
    }
}
