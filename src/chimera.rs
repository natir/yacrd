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
use io;
use utils;

/* crates use */

/* standard use */
use std;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::hash::{Hash, Hasher};

/* begin of type declaration */

#[derive(Debug, PartialEq)]
pub enum BadReadType {
    Chimeric,
    NotCovered,
    NotBad,
}

impl Eq for BadReadType {}

impl BadReadType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BadReadType::Chimeric => "Chimeric",
            BadReadType::NotCovered => "Not_covered",
            BadReadType::NotBad => "NotBad",
        }
    }
}

#[derive(Debug)]
struct NameLen {
    name: String,
    len: u64,
}

impl PartialEq for NameLen {
    fn eq(&self, other: &NameLen) -> bool {
        self.name == other.name
    }
}

impl Eq for NameLen {}

impl Hash for NameLen {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct Interval {
    pub begin: u64,
    pub end: u64,
}

impl Ord for Interval {
    fn cmp(&self, other: &Interval) -> Ordering {
        let r = self.begin.cmp(&other.begin);

        return match r {
            Ordering::Equal => self.end.cmp(&other.end),
            _ => r,
        };
    }
}

impl PartialOrd for Interval {
    fn partial_cmp(&self, other: &Interval) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Interval) -> bool {
        self.begin == other.begin && self.end == other.end
    }
}

impl Eq for Interval {}

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

pub type BadReadMap = HashMap<String, (BadReadType, Vec<Interval>)>;

/* End of type declaration */

pub fn find<R: std::io::Read, W: std::io::Write>(
    inputs: Vec<R>,
    mut output: W,
    formats: Vec<utils::Format>,
    chim_thres: u64,
    ncov_thres: f64,
    remove_reads: &mut BadReadMap,
) {
    let mut read2mapping: HashMap<NameLen, Vec<Interval>> = HashMap::new();

    for (input, format) in inputs.into_iter().zip(formats.iter()) {
        parse(input, format, &mut read2mapping);
    }

    let mut middle_gaps: Vec<Interval> = Vec::new();
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

            if stack.len() == chim_thres as usize {
                if last_covered != 0 {
                    middle_gaps.push(Interval {
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

        let uncovered_extremities = first_covered + (key.len - last_covered);

        let label = if !middle_gaps.is_empty() {
            BadReadType::Chimeric
        } else if uncovered_extremities > (ncov_thres * key.len as f64) as u64 {
            BadReadType::NotCovered
        } else {
            BadReadType::NotBad
        };

        if label != BadReadType::NotBad {
            if first_covered != 0 {
                middle_gaps.insert(
                    0,
                    Interval {
                        begin: 0,
                        end: first_covered,
                    },
                );
            }

            if last_covered != key.len {
                middle_gaps.push(Interval {
                    begin: last_covered,
                    end: key.len,
                });
            }

            write_result(&mut output, &label, &key.name, &key.len, &middle_gaps);

            remove_reads.insert(key.name.to_string(), (label, middle_gaps.clone()));
        }
    }
}

pub fn write_result<W: std::io::Write>(
    mut output: &mut W,
    label: &BadReadType,
    name: &str,
    len: &u64,
    gaps: &Vec<Interval>,
) {
    output
        .write_fmt(format_args!("{}\t{}\t{}\t", label.as_str(), name, len))
        .expect("Error durring writting of result");

    for (i, interval) in gaps.iter().enumerate() {
        write_gap(interval, &mut output, gaps.len() - i);
    }

    output
        .write(b"\n")
        .expect("Error durring writting of result");
}

fn write_gap<W: std::io::Write>(gap: &Interval, output: &mut W, i: usize) {
    output
        .write_fmt(format_args!(
            "{},{},{}",
            gap.end - gap.begin,
            gap.begin,
            gap.end
        ))
        .expect("Error durring writting of result");
    if i > 1 {
        output
            .write(b";")
            .expect("Error durring writting of result");
    }
}

fn parse<R: std::io::Read>(
    input: R,
    format: &utils::Format,
    read2mapping: &mut HashMap<NameLen, Vec<Interval>>,
) {
    match format {
        utils::Format::Paf => parse_paf(input, read2mapping),
        utils::Format::Mhap => parse_mhap(input, read2mapping),
        _ => panic!("Isn't a mapping format"),
    }
}

fn parse_paf<R: std::io::Read>(input: R, read2mapping: &mut HashMap<NameLen, Vec<Interval>>) {
    let mut reader = io::paf::Reader::new(input);

    for result in reader.records() {
        let record = result.unwrap();

        let key_a = NameLen {
            name: record.read_a,
            len: record.length_a,
        };
        let val_a = Interval {
            begin: record.begin_a,
            end: record.end_a,
        };

        let key_b = NameLen {
            name: record.read_b,
            len: record.length_b,
        };
        let val_b = Interval {
            begin: record.begin_b,
            end: record.end_b,
        };

        read2mapping.entry(key_a).or_insert(Vec::new()).push(val_a);
        read2mapping.entry(key_b).or_insert(Vec::new()).push(val_b);
    }
}

fn parse_mhap<R: std::io::Read>(input: R, read2mapping: &mut HashMap<NameLen, Vec<Interval>>) {
    let mut reader = io::mhap::Reader::new(input);

    for result in reader.records() {
        let record = result.unwrap();

        let key_a = NameLen {
            name: record.read_a,
            len: record.length_a,
        };
        let val_a = Interval {
            begin: record.begin_a,
            end: record.end_a,
        };

        let key_b = NameLen {
            name: record.read_b,
            len: record.length_b,
        };
        let val_b = Interval {
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

    const MHAP_FILE: &'static [u8] = b"1 2 0.1 2 0 20 4500 12000 0 5500 10000 10000
1 3 0.1 2 0 5500 10000 12000 0 0 4500 10000
";

    const NOT_COVERED_FILE: &'static [u8] =
        b"1\t10000\t1000\t10000\t-\t2\t10000\t0\t9000\t9000\t9000\t255
1\t10000\t0\t1000\t-\t3\t10000\t9000\t10000\t1000\t1000\t255
";

    #[test]
    fn find_chimera() {
        let good = b"Chimeric\t1\t12000\t20,0,20;1000,4500,5500;2000,10000,12000\n";

        let mut remove_reads: BadReadMap = HashMap::new();
        let mut writer: Vec<u8> = Vec::new();

        find(
            vec![PAF_FILE],
            &mut writer,
            vec![utils::Format::Paf],
            0,
            0.8,
            &mut remove_reads,
        );

        assert_eq!(writer, good.to_vec());

        writer.clear();
        find(
            vec![MHAP_FILE],
            &mut writer,
            vec![utils::Format::Mhap],
            0,
            0.8,
            &mut remove_reads,
        );
        assert_eq!(writer, good.to_vec());
    }

    #[test]
    fn find_chimera_cov_1() {
        let result = "Chimeric\t4\t6000\t1000,2500,3500\nChimeric\t1\t10000\t2000,0,2000;1000,4500,5500;2000,8000,10000\n".to_string();
        let good: HashSet<&str> = result.split("\n").collect();
        let mut remove_reads: BadReadMap = HashMap::new();
        let mut writer: Vec<u8> = Vec::new();

        find(
            vec![PAF_FILE_COV_1],
            &mut writer,
            vec![utils::Format::Paf],
            1,
            0.8,
            &mut remove_reads,
        );

        assert_eq!(
            String::from_utf8_lossy(&writer)
                .split("\n")
                .collect::<HashSet<&str>>(),
            good
        );
    }

    #[test]
    fn find_not_covered() {
        let mut remove_reads: BadReadMap = HashMap::new();
        let mut writer: Vec<u8> = Vec::new();

        find(
            vec![NOT_COVERED_FILE],
            &mut writer,
            vec![utils::Format::Paf],
            0,
            0.8,
            &mut remove_reads,
        );

        let good = b"Not_covered\t3\t10000\t9000,0,9000\n";
        assert_eq!(writer, good.to_vec());
    }

    #[test]
    fn formating_gap() {
        let mut writer: Vec<u8> = Vec::new();
        let input = vec![
            Interval { begin: 0, end: 10 },
            Interval {
                begin: 50,
                end: 100,
            },
            Interval {
                begin: 150,
                end: 200,
            },
        ];

        for (i, gaps) in input.iter().enumerate() {
            write_gap(gaps, &mut writer, input.len() - i);
        }

        assert_eq!(writer, b"10,0,10;50,50,100;50,150,200");
    }

    lazy_static! {
        static ref READ2MAPPING: HashMap<NameLen, Vec<Interval>> = {
            let mut m = HashMap::new();

            m.insert(
                NameLen {
                    name: "3".to_string(),
                    len: 10000,
                },
                vec![Interval {
                    begin: 0,
                    end: 4500,
                }],
            );
            m.insert(
                NameLen {
                    name: "2".to_string(),
                    len: 10000,
                },
                vec![Interval {
                    begin: 5500,
                    end: 10000,
                }],
            );
            m.insert(
                NameLen {
                    name: "1".to_string(),
                    len: 12000,
                },
                vec![
                    Interval {
                        begin: 20,
                        end: 4500,
                    },
                    Interval {
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
        let mut hash: HashMap<NameLen, Vec<Interval>> = HashMap::new();
        parse(Box::new(PAF_FILE), &utils::Format::Paf, &mut hash);
        assert_eq!(*READ2MAPPING, hash);

        hash.clear();
        parse(Box::new(MHAP_FILE), &utils::Format::Mhap, &mut hash);
        assert_eq!(*READ2MAPPING, hash);
    }
}
