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

use chimera;
use io;
use file;
use postdetection;

pub trait PostDetectionOperationPaf {
    fn work(self: &Self, reads: &chimera::BadReadMap, filename_in: &str, suffix: &str) {
        let filename_out = postdetection::generate_out_name(filename_in.to_string(), suffix);

        let (raw_input, compression) = file::get_readable_file(filename_in);
        let input = Box::new(raw_input);
        let output = Box::new(file::get_output(&filename_out, compression));

        let mut reader = io::paf::Reader::new(input);
        let mut writer = io::paf::Writer::new(output);

        for result in reader.records() {
            let record = result.unwrap();
            for out in self.check(reads, &record) {
                writer.write(&out).unwrap()
            }
        }
    }

    fn check(
        self: &Self,
        reads: &chimera::BadReadMap,
        record: &io::paf::Record,
    ) -> Vec<io::paf::Record>;
}

pub struct Filter;

impl Filter {
    pub fn run(reads: &chimera::BadReadMap, filename_in: &str, suffix: &str) {
        let a = Filter {};
        a.work(reads, filename_in, suffix);
    }
}

impl PostDetectionOperationPaf for Filter {
    fn check(
        self: &Self,
        reads: &chimera::BadReadMap,
        record: &io::paf::Record,
    ) -> Vec<io::paf::Record> {
        if !(reads.contains_key(&record.read_a) || reads.contains_key(&record.read_b)) {
            return vec![record.clone()];
        }
        return Vec::new();
    }
}

pub struct Extract;

impl Extract {
    pub fn run(reads: &chimera::BadReadMap, filename_in: &str, suffix: &str) {
        let a = Extract {};
        a.work(reads, filename_in, suffix);
    }
}

impl PostDetectionOperationPaf for Extract {
    fn check(
        self: &Self,
        reads: &chimera::BadReadMap,
        record: &io::paf::Record,
    ) -> Vec<io::paf::Record> {
        if reads.contains_key(&record.read_a) || reads.contains_key(&record.read_b) {
            return vec![record.clone()];
        }
        return Vec::new();
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use std::collections::HashMap;

    lazy_static! {
        static ref REMOVE_READS: Box<chimera::BadReadMap> = {
            let mut m = Box::new(HashMap::new());
            m.insert(
                "1".to_string(),
                (
                    chimera::BadReadType::Chimeric,
                    vec![chimera::Interval {
                        begin: 4500,
                        end: 5500,
                    }],
                ),
            );
            m
        };
    }

    const PAF_FILE: &'static [u8] = b"1\t12000\t20\t4500\t-\t2\t10000\t5500\t10000\t4500\t4500\t255
1\t12000\t5500\t10000\t-\t3\t10000\t0\t4500\t4500\t4500\t255
";

    const PAF_FILE_FILTRED: &'static [u8] = b"";

    #[test]
    fn filtred() {
        let mut out: Vec<u8> = Vec::new();

        let f = Filter {};
        {
            let mut reader = io::paf::Reader::new(PAF_FILE);
            let mut writer = io::paf::Writer::new(&mut out);

            for result in reader.records() {
                let record = result.unwrap();
                for out in f.check(&REMOVE_READS, &record) {
                    writer.write(&out).unwrap()
                }
            }
        }
        assert_eq!(out, PAF_FILE_FILTRED);
    }

    const PAF_FILE_EXTRACTED: &'static [u8] = b"1\t12000\t20\t4500\t-\t2\t10000\t5500\t10000\t4500\t4500\t255
1\t12000\t5500\t10000\t-\t3\t10000\t0\t4500\t4500\t4500\t255
";

    #[test]
    fn extracted() {
        let mut out: Vec<u8> = Vec::new();

        let f = Extract {};
        {
            let mut reader = io::paf::Reader::new(PAF_FILE);
            let mut writer = io::paf::Writer::new(&mut out);

            for result in reader.records() {
                let record = result.unwrap();
                for out in f.check(&REMOVE_READS, &record) {
                    writer.write(&out).unwrap()
                }
            }
        }
        assert_eq!(out, PAF_FILE_EXTRACTED);
    }
}
