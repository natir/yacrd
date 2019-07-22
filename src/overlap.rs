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

/* local use */
use chimera;
use io;
use utils;

/* crates use */

/* standard use */
use std;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, PartialEq)]
struct MinInteger(u64);

impl Eq for MinInteger {}

impl PartialOrd for MinInteger {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for MinInteger {
    fn cmp(&self, other: &MinInteger) -> Ordering {
        let ord = self.partial_cmp(other).unwrap();
        match ord {
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => ord,
        }
    }
}

/* End of type declaration */

pub fn find<R: std::io::Read>(
    input: R,
    format: utils::Format,
    chim_thres: u64,
    ncov_thres: f64,
    remove_reads: &mut chimera::BadReadMap,
) {
    let mut read2mapping: HashMap<chimera::NameLen, Vec<chimera::Interval>> = HashMap::new();

    parse(input, &format, &mut read2mapping);

    let mut middle_gaps: Vec<chimera::Interval> = Vec::new();
    let mut stack: BinaryHeap<MinInteger> = BinaryHeap::new();

    for (key, val) in read2mapping.iter_mut() {
        middle_gaps.clear();
        stack.clear();

        val.sort();

        let mut first_covered = 0;
        let mut last_covered = 0;

        for interval in val {
            while !stack.is_empty() && stack.peek().unwrap().0 < interval.begin {
                if stack.len() > chim_thres as usize {
                    last_covered = stack.peek().unwrap().0;
                }
                stack.pop();
            }

            if stack.len() <= chim_thres as usize {
                if last_covered != 0 {
                    middle_gaps.push(chimera::Interval {
                        begin: last_covered,
                        end: interval.begin,
                    });
                } else {
                    first_covered = interval.begin;
                }
            }

            stack.push(MinInteger(interval.end));
        }

        while stack.len() > chim_thres as usize {
            last_covered = stack.peek().unwrap().0;
            if last_covered >= key.len {
                break;
            }
            stack.pop();
        }

        let mut uncovered_len = first_covered + (key.len - last_covered);
        for gap in middle_gaps.iter() {
            uncovered_len += gap.end - gap.begin
        }

        let label = if uncovered_len > (ncov_thres * key.len as f64) as u64 {
            chimera::BadReadType::NotCovered
        } else if !middle_gaps.is_empty() {
            chimera::BadReadType::Chimeric
        } else {
            chimera::BadReadType::NotBad
        };

        if first_covered != 0 {
            middle_gaps.insert(
                0,
                chimera::Interval {
                    begin: 0,
                    end: first_covered,
                },
            );
        }

        if last_covered != key.len {
            middle_gaps.push(chimera::Interval {
                begin: last_covered,
                end: key.len,
            });
        }

        remove_reads.insert(key.name.to_string(), (label, key.len, middle_gaps.clone()));
    }
}

fn parse<R: std::io::Read>(
    input: R,
    format: &utils::Format,
    read2mapping: &mut HashMap<chimera::NameLen, Vec<chimera::Interval>>,
) {
    match format {
        utils::Format::Paf => parse_paf(input, read2mapping),
        utils::Format::Mhap => parse_mhap(input, read2mapping),
        _ => panic!("Isn't a mapping format"),
    }
}

fn parse_paf<R: std::io::Read>(
    input: R,
    read2mapping: &mut HashMap<chimera::NameLen, Vec<chimera::Interval>>,
) {
    let mut reader = io::paf::Reader::new(input);

    for result in reader.records() {
        let record = result.unwrap();

        let key_a = chimera::NameLen {
            name: record.read_a,
            len: record.length_a,
        };
        let val_a = chimera::Interval {
            begin: record.begin_a,
            end: record.end_a,
        };

        let key_b = chimera::NameLen {
            name: record.read_b,
            len: record.length_b,
        };
        let val_b = chimera::Interval {
            begin: record.begin_b,
            end: record.end_b,
        };

        read2mapping.entry(key_a).or_insert(Vec::new()).push(val_a);
        read2mapping.entry(key_b).or_insert(Vec::new()).push(val_b);
    }
}

fn parse_mhap<R: std::io::Read>(
    input: R,
    read2mapping: &mut HashMap<chimera::NameLen, Vec<chimera::Interval>>,
) {
    let mut reader = io::mhap::Reader::new(input);

    for result in reader.records() {
        let record = result.unwrap();

        let key_a = chimera::NameLen {
            name: record.read_a,
            len: record.length_a,
        };
        let val_a = chimera::Interval {
            begin: record.begin_a,
            end: record.end_a,
        };

        let key_b = chimera::NameLen {
            name: record.read_b,
            len: record.length_b,
        };
        let val_b = chimera::Interval {
            begin: record.begin_b,
            end: record.end_b,
        };

        read2mapping.entry(key_a).or_insert(Vec::new()).push(val_a);
        read2mapping.entry(key_b).or_insert(Vec::new()).push(val_b);
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use std::collections::HashSet;

    const PAF_FILE: &'static [u8] = b"1\t12000\t20\t4500\t-\t2\t10000\t5500\t10000\t4500\t4500\t255
1\t12000\t5500\t10000\t-\t3\t10000\t0\t4500\t4500\t4500\t255
";

    const PAF_FILE_COV_1: &'static [u8] =
        b"1\t10000\t0\t4500\t-\t2\t10000\t5500\t10000\t4500\t4500\t255
1\t10000\t5500\t10000\t-\t3\t10000\t0\t4500\t4500\t4500\t255
1\t10000\t2000\t8000\t+\t4\t6000\t0\t6000\t6000\t6000\t255
2\t10000\t7500\t10000\t-\t4\t6000\t0\t2500\t2500\t2500\t255
3\t10000\t0\t2500\t-\t4\t6000\t3500\t6000\t2500\t2500\t255
";

    const PAF_FILE_NOTCOV_PRIOR: &'static [u8] =
        b"1\t10000\t4000\t4500\t-\t2\t10000\t1000\t9000\t7000\t7000\t255
1\t10000\t5000\t5500\t-\t3\t10000\t1000\t9000\t7000\t7000\t255
";

    const PAF_FILE_NOTCOV_OVEXT: &'static [u8] =
        b"1\t10000\t500\t1500\t-\t2\t10000\t1000\t9000\t7000\t7000\t255
1\t10000\t9000\t9500\t-\t3\t10000\t1000\t9000\t7000\t7000\t255
";

    const MHAP_FILE: &'static [u8] = b"1 2 0.1 2 0 20 4500 12000 0 5500 10000 10000
1 3 0.1 2 0 5500 10000 12000 0 0 4500 10000
";

    const MHAP_FILE_MIN_MERS_FLOAT: &'static [u8] =
        b"1 2 0.1 2.0 0 20 4500 12000 0 5500 10000 10000
1 3 0.1 2.0 0 5500 10000 12000 0 0 4500 10000
";

    const NOT_COVERED_FILE: &'static [u8] =
        b"1\t10000\t1000\t10000\t-\t2\t10000\t0\t9000\t9000\t9000\t255
1\t10000\t0\t1000\t-\t3\t10000\t9000\t10000\t1000\t1000\t255
";

    #[test]
    fn find_chimera() {
        let good = b"Chimeric\t1\t12000\t20,0,20;1000,4500,5500;2000,10000,12000\n";

        let mut remove_reads: chimera::BadReadMap = HashMap::new();
        let mut writer: Vec<u8> = Vec::new();

        find(PAF_FILE, utils::Format::Paf, 0, 0.8, &mut remove_reads);

        chimera::write(&mut writer, &remove_reads, false);

        assert_eq!(writer, good.to_vec());

        writer.clear();

        find(MHAP_FILE, utils::Format::Mhap, 0, 0.8, &mut remove_reads);

        chimera::write(&mut writer, &remove_reads, false);

        assert_eq!(writer, good.to_vec());
    }

    #[test]
    fn find_chimera_mhap_min_mers_float() {
        let good = b"Chimeric\t1\t12000\t20,0,20;1000,4500,5500;2000,10000,12000\n";

        let mut remove_reads: chimera::BadReadMap = HashMap::new();
        let mut writer: Vec<u8> = Vec::new();

        find(
            MHAP_FILE_MIN_MERS_FLOAT,
            utils::Format::Mhap,
            0,
            0.8,
            &mut remove_reads,
        );

        chimera::write(&mut writer, &remove_reads, false);

        assert_eq!(writer, good.to_vec());
    }

    #[test]
    fn find_chimera_cov_1() {
        let result = "Chimeric\t4\t6000\t1000,2500,3500\nChimeric\t1\t10000\t2000,0,2000;1000,4500,5500;2000,8000,10000\n".to_string();
        let good: HashSet<&str> = result.split("\n").collect();
        let mut remove_reads: chimera::BadReadMap = HashMap::new();
        let mut writer: Vec<u8> = Vec::new();

        find(
            PAF_FILE_COV_1,
            utils::Format::Paf,
            1,
            0.8,
            &mut remove_reads,
        );

        chimera::write(&mut writer, &remove_reads, false);

        assert_eq!(
            String::from_utf8_lossy(&writer)
                .split("\n")
                .collect::<HashSet<&str>>(),
            good
        );
    }

    #[test]
    fn find_chimera_json_output() {
        let result = "{\"1\":{\"gaps\":[{\"begin\":0,\"end\":20},{\"begin\":4500,\"end\":5500},{\"begin\":10000,\"end\":12000}],\"length\":12000,\"type\":\"Chimeric\"}}".to_string();
        let good: HashSet<&str> = result.split("\n").collect();
        let mut remove_reads: chimera::BadReadMap = HashMap::new();
        let mut writer: Vec<u8> = Vec::new();

        find(PAF_FILE, utils::Format::Paf, 0, 0.8, &mut remove_reads);

        chimera::write(&mut writer, &remove_reads, true);

        assert_eq!(
            String::from_utf8_lossy(&writer)
                .split("\n")
                .collect::<HashSet<&str>>(),
            good
        );
    }

    #[test]
    fn notcovered_prior_to_chimera() {
        let result =
            "Not_covered\t1\t10000\t4000,0,4000;500,4500,5000;4500,5500,10000\n".to_string();
        let good: HashSet<&str> = result.split("\n").collect();
        let mut remove_reads: chimera::BadReadMap = HashMap::new();
        let mut writer: Vec<u8> = Vec::new();

        find(
            PAF_FILE_NOTCOV_PRIOR,
            utils::Format::Paf,
            0,
            0.8,
            &mut remove_reads,
        );

        chimera::write(&mut writer, &remove_reads, false);

        assert_eq!(
            String::from_utf8_lossy(&writer)
                .split("\n")
                .collect::<HashSet<&str>>(),
            good
        );
    }

    #[test]
    fn notcovered_overlap_extremity() {
        let result = "Not_covered\t1\t10000\t500,0,500;7500,1500,9000;500,9500,10000\n".to_string();
        let good: HashSet<&str> = result.split("\n").collect();
        let mut remove_reads: chimera::BadReadMap = HashMap::new();
        let mut writer: Vec<u8> = Vec::new();

        find(
            PAF_FILE_NOTCOV_OVEXT,
            utils::Format::Paf,
            0,
            0.8,
            &mut remove_reads,
        );

        chimera::write(&mut writer, &remove_reads, false);

        assert_eq!(
            String::from_utf8_lossy(&writer)
                .split("\n")
                .collect::<HashSet<&str>>(),
            good
        );
    }

    #[test]
    fn find_not_covered() {
        let mut remove_reads: chimera::BadReadMap = HashMap::new();
        let mut writer: Vec<u8> = Vec::new();

        find(
            NOT_COVERED_FILE,
            utils::Format::Paf,
            0,
            0.8,
            &mut remove_reads,
        );

        chimera::write(&mut writer, &remove_reads, false);

        let good = b"Not_covered\t3\t10000\t9000,0,9000\n";
        assert_eq!(writer, good.to_vec());
    }

    lazy_static! {
        static ref READ2MAPPING: HashMap<chimera::NameLen, Vec<chimera::Interval>> = {
            let mut m = HashMap::new();

            m.insert(
                chimera::NameLen {
                    name: "3".to_string(),
                    len: 10000,
                },
                vec![chimera::Interval {
                    begin: 0,
                    end: 4500,
                }],
            );
            m.insert(
                chimera::NameLen {
                    name: "2".to_string(),
                    len: 10000,
                },
                vec![chimera::Interval {
                    begin: 5500,
                    end: 10000,
                }],
            );
            m.insert(
                chimera::NameLen {
                    name: "1".to_string(),
                    len: 12000,
                },
                vec![
                    chimera::Interval {
                        begin: 20,
                        end: 4500,
                    },
                    chimera::Interval {
                        begin: 5500,
                        end: 10000,
                    },
                ],
            );
            m
        };
    }

    #[test]
    fn mapping2read2mapping() {
        let mut hash: HashMap<chimera::NameLen, Vec<chimera::Interval>> = HashMap::new();
        parse(Box::new(PAF_FILE), &utils::Format::Paf, &mut hash);
        assert_eq!(*READ2MAPPING, hash);

        hash.clear();
        parse(Box::new(MHAP_FILE), &utils::Format::Mhap, &mut hash);
        assert_eq!(*READ2MAPPING, hash);
    }

    #[test]
    fn find_chimera_report_all() {
        let result = "NotBad\t3\t10000\t7500,2500,10000\nChimeric\t4\t6000\t1000,2500,3500\nChimeric\t1\t10000\t2000,0,2000;1000,4500,5500;2000,8000,10000\nNotBad\t2\t10000\t7500,0,7500\n".to_string();

        let good: HashSet<&str> = result.split("\n").collect();
        let mut remove_reads: chimera::BadReadMap = HashMap::new();
        let mut writer: Vec<u8> = Vec::new();

        find(
            PAF_FILE_COV_1,
            utils::Format::Paf,
            1,
            0.8,
            &mut remove_reads,
        );

        chimera::write(&mut writer, &remove_reads, false);

        assert_eq!(
            String::from_utf8_lossy(&writer)
                .split("\n")
                .collect::<HashSet<&str>>(),
            good
        );
    }

}
