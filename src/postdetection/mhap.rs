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

pub trait PostDetectionOperationMhap {
    fn work(self: &Self, reads: &chimera::BadReadMap, filename_in: &str, suffix: &str) {
        let filename_out = postdetection::generate_out_name(filename_in.to_string(), suffix);

        let (raw_input, compression) = file::get_readable_file(filename_in);
        let input = Box::new(raw_input);
        let output = Box::new(file::get_output(&filename_out, compression));

        let mut reader = io::mhap::Reader::new(input);
        let mut writer = io::mhap::Writer::new(output);

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
        record: &io::mhap::Record,
    ) -> Vec<io::mhap::Record>;
}

pub struct Filter;

impl Filter {
    pub fn run(reads: &chimera::BadReadMap, filename_in: &str, suffix: &str) {
        let a = Filter {};
        a.work(reads, filename_in, suffix);
    }
}

impl PostDetectionOperationMhap for Filter {
    fn check(
        self: &Self,
        reads: &chimera::BadReadMap,
        record: &io::mhap::Record,
    ) -> Vec<io::mhap::Record> {
        if !(reads.contains_key(&record.read_a) || reads.contains_key(&record.read_b)) {
            return vec![record.clone()];
        }
        return Vec::new();
    }
}

pub struct Extract;

impl Extract {
    pub fn run(reads: &chimera::BadReadMap, filename_in: &str, suffix: &str) {
        let a = Filter {};
        a.work(reads, filename_in, suffix);
    }
}

impl PostDetectionOperationMhap for Extract {
    fn check(
        self: &Self,
        reads: &chimera::BadReadMap,
        record: &io::mhap::Record,
    ) -> Vec<io::mhap::Record> {
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

    const MHAP_FILE: &'static [u8] = b"1 2 0.1 2 0 100 450 1000 0 550 900 1000
1 3 0.1 2 0 550 900 1000 0 100 450 1000
";

    const MHAP_FILE_FILTRED: &'static [u8] = b"";

    #[test]
    fn filtred() {
        let mut out: Vec<u8> = Vec::new();

        let f = Filter {};
        {
            let mut reader = io::mhap::Reader::new(MHAP_FILE);
            let mut writer = io::mhap::Writer::new(&mut out);

            for result in reader.records() {
                let record = result.unwrap();
                for out in f.check(&REMOVE_READS, &record) {
                    writer.write(&out).unwrap()
                }
            }
        }
        assert_eq!(out, MHAP_FILE_FILTRED);
    }

    const MHAP_FILE_EXTRACTED: &'static [u8] = b"1 2 0.1 2 0 100 450 1000 0 550 900 1000
1 3 0.1 2 0 550 900 1000 0 100 450 1000
";

    #[test]
    fn extracted() {
        let mut out: Vec<u8> = Vec::new();

        let f = Extract {};
        {
            let mut reader = io::mhap::Reader::new(MHAP_FILE);
            let mut writer = io::mhap::Writer::new(&mut out);

            for result in reader.records() {
                let record = result.unwrap();
                for out in f.check(&REMOVE_READS, &record) {
                    writer.write(&out).unwrap()
                }
            }
        }
        assert_eq!(out, MHAP_FILE_EXTRACTED);
    }
}
