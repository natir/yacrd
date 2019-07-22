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

use clap::ArgMatches;

use itertools::Itertools;

use file;

use chimera;
use utils;

pub fn run<'a>(reads_info: &chimera::BadReadMap, matches: ArgMatches<'a>) {
    let (input, output, format) = parse(matches);

    match format {
        utils::Format::Fasta => fasta(input, output, reads_info),
        utils::Format::Fastq => fastq(input, output, reads_info),
        _ => (),
    }
}

fn parse<'a>(
    matches: ArgMatches,
) -> (
    std::io::BufReader<Box<dyn std::io::Read>>,
    std::io::BufWriter<Box<dyn std::io::Write>>,
    utils::Format,
) {
    let input_path = matches.value_of("input").unwrap();
    let (input, compression) = file::get_readable_file(input_path);

    let output = Box::new(file::get_output(
        matches.value_of("output").unwrap(),
        compression,
    ));

    return (
        std::io::BufReader::new(input),
        std::io::BufWriter::new(output),
        utils::get_format(input_path).unwrap(),
    );
}

fn fasta(
    input: std::io::BufReader<Box<dyn std::io::Read>>,
    output: std::io::BufWriter<Box<dyn std::io::Write>>,
    reads_info: &chimera::BadReadMap,
) -> () {
    let reader = bio::io::fasta::Reader::new(input);
    let mut writer = bio::io::fasta::Writer::new(output);

    for result in reader.records() {
        let record = result.unwrap();
        for out in record.scrubb(reads_info) {
            writer.write_record(&out).unwrap()
        }
    }
}

fn fastq(
    input: std::io::BufReader<Box<dyn std::io::Read>>,
    output: std::io::BufWriter<Box<dyn std::io::Write>>,
    reads_info: &chimera::BadReadMap,
) -> () {
    let reader = bio::io::fastq::Reader::new(input);
    let mut writer = bio::io::fastq::Writer::new(output);

    for result in reader.records() {
        let record = result.unwrap();
        for out in record.scrubb(reads_info) {
            writer.write_record(&out).unwrap()
        }
    }
}

fn get_good_pos(rm_int: &Vec<chimera::Interval>, max: u64) -> Vec<chimera::Interval> {
    let mut good_part = Vec::new();

    for (int1, int2) in vec![chimera::Interval::new(0, 0)]
        .iter()
        .chain(rm_int)
        .chain(vec![chimera::Interval::new(max, max)].iter())
        .tuple_windows()
    {
        let a = int1.end;
        let b = int2.begin;

        if a == b {
            continue;
        };

        good_part.push(chimera::Interval::new(a, b));
    }

    return good_part;
}

trait Scrubable<R> {
    fn scrubb(self: &Self, reads: &chimera::BadReadMap) -> Vec<R>;
}

impl Scrubable<bio::io::fasta::Record> for bio::io::fasta::Record {
    fn scrubb(self: &Self, reads: &chimera::BadReadMap) -> Vec<bio::io::fasta::Record> {
        let mut subrecord = vec![];

        let (_, _, gaps) = reads.get(self.id()).unwrap();

        for int_of_good in get_good_pos(gaps, self.seq().len() as u64) {
            let a = int_of_good.begin;
            let b = int_of_good.end;

            if !utils::in_read(a as usize, b as usize, self.seq().len()) {
                break; // interval not in record position next interval can't be good
            }

            subrecord.push(bio::io::fasta::Record::with_attrs(
                format!("{}_{}_{}", self.id(), a, b).as_str(),
                self.desc(),
                &self.seq()[(a as usize)..(b as usize)],
            ));
        }

        return subrecord;
    }
}

impl Scrubable<bio::io::fastq::Record> for bio::io::fastq::Record {
    fn scrubb(self: &Self, reads: &chimera::BadReadMap) -> Vec<bio::io::fastq::Record> {
        let mut subrecord = vec![];

        let (read_type, _, gaps) = reads.get(self.id()).unwrap();

        for int_of_good in get_good_pos(gaps, self.seq().len() as u64) {
            let a = int_of_good.begin;
            let b = int_of_good.end;

            if !utils::in_read(a as usize, b as usize, self.seq().len()) {
                break; // interval not in record position next interval can't be good
            }

            subrecord.push(bio::io::fastq::Record::with_attrs(
                format!("{}_{}_{}", self.id(), a, b).as_str(),
                self.desc(),
                &self.seq()[(a as usize)..(b as usize)],
                &self.qual()[(a as usize)..(b as usize)],
            ));
        }

        return subrecord;
    }
}
