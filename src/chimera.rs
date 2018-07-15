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
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::{Hash, Hasher};

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

#[derive(Debug)]
struct Interval {
    begin: u64,
    end: u64,
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

pub fn find(
    input: Box<std::io::Read>,
    output: &mut Box<std::io::Write>,
    format: utils::Format,
    chim_thres: u64,
    ncov_thres: f64,
) -> Box<HashSet<String>> {
    let mut remove_reads: HashSet<String> = HashSet::new();
    let mut read2mapping: HashMap<NameLen, Vec<Interval>> = HashMap::new();

    parse(input, format, &mut read2mapping);

    let mut middle_gaps: Vec<Interval> = Vec::new();
    let mut stack: BinaryHeap<u64> = BinaryHeap::new();

    for (key, val) in read2mapping.iter_mut() {
        stack.clear();
        middle_gaps.clear();

        val.sort();

        let mut first_covered = 0;
        let mut last_covered = 0;
        for interval in val {
            while !stack.is_empty() && stack.peek().unwrap() < &interval.begin {
                if stack.len() > chim_thres as usize {
                    last_covered = *stack.peek().unwrap();
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

            stack.push(interval.end);
        }

        while stack.len() > chim_thres as usize {
            last_covered = *stack.peek().unwrap();
            if last_covered >= key.len {
                break;
            }
            stack.pop();
        }

        let uncovered_extremities = first_covered + (key.len - last_covered);

        let label = if !middle_gaps.is_empty() {
            "Chimeric"
        } else if uncovered_extremities > (ncov_thres * key.len as f64) as u64 {
            "Not_covered"
        } else {
            ""
        };

        if label != "" {
            remove_reads.insert(key.name.to_string());

            output
                .write_fmt(format_args!("{}\t{}\t{}\t", label, key.name, key.len))
                .expect("Error durring writting of result");

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

            for (i, interval) in middle_gaps.iter().enumerate() {
                write_gap(interval, output, middle_gaps.len() - i);
            }

            output
                .write(b"\n")
                .expect("Error durring writting of result");
        }
    }

    return Box::new(remove_reads);
}

fn write_gap(gap: &Interval, output: &mut Box<std::io::Write>, i: usize) {
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

fn parse(
    input: Box<std::io::Read>,
    format: utils::Format,
    read2mapping: &mut HashMap<NameLen, Vec<Interval>>,
) -> () {
    match format {
        utils::Format::Paf => parse_paf(input, read2mapping),
        utils::Format::Mhap => parse_mhap(input, read2mapping),
        _ => panic!("Isn't a mapping format"),
    }
}

fn parse_paf(input: Box<std::io::Read>, read2mapping: &mut HashMap<NameLen, Vec<Interval>>) -> () {
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

fn parse_mhap(input: Box<std::io::Read>, read2mapping: &mut HashMap<NameLen, Vec<Interval>>) -> () {
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
