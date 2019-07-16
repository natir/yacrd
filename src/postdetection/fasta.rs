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
use file;
use postdetection;

use bio;

pub trait PostDetectionOperationFasta {
    fn work(self: &Self, reads: &chimera::BadReadMap, filename_in: &str, suffix: &str) {
        let filename_out = postdetection::generate_out_name(filename_in, suffix);

        let (raw_input, compression) = file::get_readable_file(filename_in);
        let input = Box::new(raw_input);
        let output = Box::new(file::get_output(&filename_out, compression));

        let reader = bio::io::fasta::Reader::new(input);
        let mut writer = bio::io::fasta::Writer::new(output);

        for result in reader.records() {
            let record = result.unwrap();
            for out in self.check(reads, &record) {
                writer.write_record(&out).unwrap()
            }
        }
    }

    fn check(
        self: &Self,
        reads: &chimera::BadReadMap,
        record: &bio::io::fasta::Record,
    ) -> Vec<bio::io::fasta::Record>;
}

pub struct Filter;

impl Filter {
    pub fn run(reads: &chimera::BadReadMap, filename_in: &str, suffix: &str) {
        let a = Filter {};
        a.work(reads, filename_in, suffix);
    }
}

impl PostDetectionOperationFasta for Filter {
    fn check(
        self: &Self,
        reads: &chimera::BadReadMap,
        record: &bio::io::fasta::Record,
    ) -> Vec<bio::io::fasta::Record> {
        if !(reads.contains_key(record.id())) {
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

impl PostDetectionOperationFasta for Extract {
    fn check(
        self: &Self,
        reads: &chimera::BadReadMap,
        record: &bio::io::fasta::Record,
    ) -> Vec<bio::io::fasta::Record> {
        if reads.contains_key(record.id()) {
            return vec![record.clone()];
        }
        return Vec::new();
    }
}

pub struct Split;

impl Split {
    pub fn run(reads: &chimera::BadReadMap, filename_in: &str, suffix: &str) {
        let a = Split {};
        a.work(reads, filename_in, suffix);
    }
}

impl PostDetectionOperationFasta for Split {
    fn check(
        self: &Self,
        reads: &chimera::BadReadMap,
        record: &bio::io::fasta::Record,
    ) -> Vec<bio::io::fasta::Record> {
        let mut subrecord = vec![];

        // check if we need work on this read
        if !reads.contains_key(record.id()) {
            return vec![record.clone()];
        }

        let (read_type, _, gaps) = reads.get(record.id()).unwrap();

        if *read_type == chimera::BadReadType::NotCovered {
            return vec![]; // if read is not covered we discard him
        }

        let mut position = vec![0];
        let mut passed_pos = std::collections::HashSet::new();
        for inter in gaps.iter() {
            if !passed_pos.contains(&inter.begin) {
                position.push(inter.begin);
                passed_pos.insert(inter.begin);
            }
            if !passed_pos.contains(&inter.end) {
                position.push(inter.end);
                passed_pos.insert(inter.end);
            }
        }

        if position.len() % 2 == 1 {
            position.push(record.seq().len() as u64);
        }

        if position.len() == 2 && position[0] == 0 && position[1] as usize == record.seq().len() {
            return vec![record.clone()];
        }

        for (a, b) in position.chunks(2).map(|x| (x[0], x[1])) {
            if a == b {
                continue; // empty interval
            }

            if !postdetection::in_read(a as usize, b as usize, record.seq().len()) {
                continue; // interval not in record position
            }

            subrecord.push(bio::io::fasta::Record::with_attrs(
                format!("{}_{}_{}", record.id(), a, b).as_str(),
                record.desc(),
                &record.seq()[(a as usize)..(b as usize)],
            ));
        }

        return subrecord;
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
                    6000,
                    vec![
                        chimera::Interval {
                            begin: 0,
                            end: 9,
                            int_type: chimera::IntervalType::Sure,
                        },
                        chimera::Interval {
                            begin: 13,
                            end: 18,
                            int_type: chimera::IntervalType::Sure,
                        },
                    ],
                ),
            );
            m
        };
    }

    lazy_static! {
        static ref REMOVE_READS_ALL: Box<chimera::BadReadMap> = {
            let mut m = Box::new(HashMap::new());
            m.insert(
                "1".to_string(),
                (
                    chimera::BadReadType::Chimeric,
                    6000,
                    vec![
                        chimera::Interval {
                            begin: 0,
                            end: 9,
                            int_type: chimera::IntervalType::Sure,
                        },
                        chimera::Interval {
                            begin: 13,
                            end: 18,
                            int_type: chimera::IntervalType::Sure,
                        },
                    ],
                ),
            );
            m.insert(
                "2".to_string(),
                (
                    chimera::BadReadType::NotBad,
                    6000,
                    vec![
                        chimera::Interval {
                            begin: 0,
                            end: 1,
                            int_type: chimera::IntervalType::Sure,
                        },
                        chimera::Interval {
                            begin: 3,
                            end: 4,
                            int_type: chimera::IntervalType::Sure,
                        },
                    ],
                ),
            );
            m.insert(
                "3".to_string(),
                (chimera::BadReadType::NotBad, 6000, vec![]),
            );
            m
        };
    }

    const FASTA_FILE: &'static [u8] = b">1
ACTG
>2
ACTG
>3
ACTG
";

    const FASTA_FILE_FILTRED: &'static [u8] = b">2
ACTG
>3
ACTG
";

    #[test]
    fn filtred() {
        let mut out: Vec<u8> = Vec::new();

        let f = Filter {};
        {
            let reader = bio::io::fasta::Reader::new(FASTA_FILE);
            let mut writer = bio::io::fasta::Writer::new(&mut out);

            for result in reader.records() {
                let record = result.unwrap();
                for out in f.check(&REMOVE_READS, &record) {
                    writer.write_record(&out).unwrap()
                }
            }
        }
        assert_eq!(out, FASTA_FILE_FILTRED);
    }

    const FASTA_FILE_EXTRACTED: &'static [u8] = b">1
ACTG
";

    #[test]
    fn extracted() {
        let mut out: Vec<u8> = Vec::new();

        let f = Extract {};
        {
            let reader = bio::io::fasta::Reader::new(FASTA_FILE);
            let mut writer = bio::io::fasta::Writer::new(&mut out);

            for result in reader.records() {
                let record = result.unwrap();
                for out in f.check(&REMOVE_READS, &record) {
                    writer.write_record(&out).unwrap()
                }
            }
        }
        assert_eq!(out, FASTA_FILE_EXTRACTED);
    }

    const FASTA_FILE_SPLITABLE: &'static [u8] = b">1
ACTGGGGGGACTGGGGGGACTG
>2
ACTG
>3
ACTG
";

    const FASTA_FILE_SPLITED: &'static [u8] = b">1_9_13
ACTG
>1_18_22
ACTG
>2
ACTG
>3
ACTG
";

    #[test]
    fn split() {
        let mut out: Vec<u8> = Vec::new();

        let f = Split {};
        {
            let reader = bio::io::fasta::Reader::new(FASTA_FILE_SPLITABLE);
            let mut writer = bio::io::fasta::Writer::new(&mut out);

            for result in reader.records() {
                let record = result.unwrap();
                for out in f.check(&REMOVE_READS, &record) {
                    writer.write_record(&out).unwrap()
                }
            }
        }

        assert_eq!(out, FASTA_FILE_SPLITED);
    }

    const FASTA_FILE_SPLITED_ALL: &'static [u8] = b">1_9_13
ACTG
>1_18_22
ACTG
>2_1_3
CT
>3
ACTG
";

    #[test]
    fn split_all() {
        let mut out: Vec<u8> = Vec::new();

        let f = Split {};
        {
            let reader = bio::io::fasta::Reader::new(FASTA_FILE_SPLITABLE);
            let mut writer = bio::io::fasta::Writer::new(&mut out);

            for result in reader.records() {
                let record = result.unwrap();
                for out in f.check(&REMOVE_READS_ALL, &record) {
                    writer.write_record(&out).unwrap()
                }
            }
        }

        assert_eq!(out, FASTA_FILE_SPLITED_ALL);
    }

    const SHORT_FASTA_FILE: &'static [u8] = b">1
ACTGGGGGGACTG
>2
ACTG
>3
ACTG
";

    const SHORT_FASTA_FILE_SPLIT: &'static [u8] = b">1_9_13
ACTG
>2
ACTG
>3
ACTG
";
    #[test]
    fn fasta_shorter_than_position() {
        let mut out: Vec<u8> = Vec::new();

        let f = Split {};
        {
            let reader = bio::io::fasta::Reader::new(SHORT_FASTA_FILE);
            let mut writer = bio::io::fasta::Writer::new(&mut out);

            for result in reader.records() {
                let record = result.unwrap();
                for out in f.check(&REMOVE_READS, &record) {
                    writer.write_record(&out).unwrap()
                }
            }
        }

        println!("{}", String::from_utf8_lossy(&out));
        println!("{}", String::from_utf8_lossy(SHORT_FASTA_FILE_SPLIT));
        assert_eq!(out, SHORT_FASTA_FILE_SPLIT);
    }
}
