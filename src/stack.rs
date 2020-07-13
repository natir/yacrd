/*
Copyright (c) 2019 Pierre Marijon <pmarijon@mpi-inf.mpg.de>

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

/* std use */
use std::cmp::Reverse;

/* crate use */
use anyhow::{Context, Result};

/* local use */
use crate::error;
use crate::reads2ovl;
use crate::util;

pub trait BadPart {
    fn get_bad_part(&mut self, id: &str) -> Result<&(Vec<(u32, u32)>, usize)>;

    fn get_reads(&self) -> std::collections::HashSet<String>;
}

pub struct FromOverlap {
    ovl: Box<dyn reads2ovl::Reads2Ovl>,
    coverage: u64,
    buffer: std::collections::HashMap<String, (Vec<(u32, u32)>, usize)>,
}

impl FromOverlap {
    pub fn new(ovl: Box<dyn reads2ovl::Reads2Ovl>, coverage: u64) -> Self {
        FromOverlap {
            ovl,
            coverage,
            buffer: std::collections::HashMap::new(),
        }
    }

    fn compute_bad_part(&self, id: &str) -> Result<(Vec<(u32, u32)>, usize)> {
        let mut gaps: Vec<(u32, u32)> = Vec::new();
        let mut stack: std::collections::BinaryHeap<Reverse<u32>> =
            std::collections::BinaryHeap::new();

        let mut ovl = self.ovl.overlap(&id)?;
        ovl.sort();

        let length = self.ovl.length(&id);

        let mut first_covered = 0;
        let mut last_covered = 0;

        for interval in ovl {
            while let Some(head) = stack.peek() {
                if head.0 > interval.0 {
                    break;
                }

                if stack.len() > self.coverage as usize {
                    last_covered = head.0;
                }
                stack.pop();
            }

            if stack.len() <= self.coverage as usize {
                if last_covered != 0 {
                    gaps.push((last_covered, interval.0));
                } else {
                    first_covered = interval.0;
                }
            }
            stack.push(Reverse(interval.1));
        }

        while stack.len() > self.coverage as usize {
            last_covered = stack
                .peek()
                .with_context(|| error::Error::NotReachableCode {
                    name: format!("{} {}", file!(), line!()),
                })?
                .0;
            if last_covered as usize >= length {
                break;
            }
            stack.pop();
        }

        if first_covered != 0 {
            gaps.insert(0, (0, first_covered));
        }

        if last_covered as usize != length {
            gaps.push((last_covered, length as u32));
        }

        if gaps.is_empty() {
            return Ok((gaps, length));
        }

        /* clean overlapped bad region */
        let mut clean_gaps: Vec<(u32, u32)> = Vec::new();
        let mut begin = gaps[0].0;
        let mut end = gaps[0].1;
        for gaps in gaps.windows(2) {
            let g1 = gaps[0];
            let g2 = gaps[1];

            if g1.0 == g2.0 {
                begin = g1.0;
                end = g1.1.max(g2.1);
            } else {
                clean_gaps.push((begin, end));
                begin = g2.0;
                end = g2.1;
            }
        }
        clean_gaps.push((begin, end));

        Ok((clean_gaps, length))
    }
}

impl BadPart for FromOverlap {
    fn get_bad_part(&mut self, id: &str) -> Result<&(Vec<(u32, u32)>, usize)> {
        if !self.buffer.contains_key(id) {
            self.buffer
                .insert(id.to_string(), self.compute_bad_part(id)?);
        }

        Ok(self
            .buffer
            .get(id)
            .with_context(|| error::Error::NotReachableCode {
                name: format!("{} {}", file!(), line!()),
            })?)
    }

    fn get_reads(&self) -> std::collections::HashSet<String> {
        self.ovl.get_reads()
    }
}

pub struct FromReport {
    buffer: std::collections::HashMap<String, (Vec<(u32, u32)>, usize)>,
    empty: (Vec<(u32, u32)>, usize),
}

impl FromReport {
    pub fn new(input_path: &str) -> Result<Self> {
        let input =
            std::io::BufReader::new(std::fs::File::open(input_path).with_context(|| {
                error::Error::CantReadFile {
                    filename: input_path.to_string(),
                }
            })?);
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(input);

        let mut buffer = std::collections::HashMap::new();
        for (line, record) in reader.records().enumerate() {
            let result = record.with_context(|| error::Error::ReadingError {
                filename: input_path.to_string(),
                format: util::FileType::Fasta,
            })?;

            let id = result[1].to_string();
            let len = util::str2usize(&result[2])?;
            let bad_part = FromReport::parse_bad_string(&result[3]).with_context(|| {
                error::Error::CorruptYacrdReport {
                    name: input_path.to_string(),
                    line,
                }
            })?;

            buffer.insert(id, (bad_part, len));
        }

        let empty = (Vec::new(), 0);
        Ok(FromReport { buffer, empty })
    }

    fn parse_bad_string(bad_string: &str) -> Result<Vec<(u32, u32)>> {
        let mut ret = Vec::new();

        for sub in bad_string.split(';') {
            let mut iter = sub.split(',');
            iter.next();

            ret.push((
                util::str2u32(
                    iter.next()
                        .with_context(|| error::Error::CorruptYacrdReportInPosition)?,
                )?,
                util::str2u32(
                    iter.next()
                        .with_context(|| error::Error::CorruptYacrdReportInPosition)?,
                )?,
            ));
        }

        Ok(ret)
    }
}

impl BadPart for FromReport {
    fn get_bad_part(&mut self, id: &str) -> Result<&(Vec<(u32, u32)>, usize)> {
        match self.buffer.get(id) {
            Some(v) => Ok(v),
            None => Ok(&self.empty),
        }
    }

    fn get_reads(&self) -> std::collections::HashSet<String> {
        self.buffer.keys().map(|x| x.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;

    extern crate tempfile;
    use self::tempfile::NamedTempFile;

    use reads2ovl::Reads2Ovl;

    #[test]
    fn from_report() {
        let mut report = NamedTempFile::new().expect("Can't create tmpfile");

        writeln!(
            report.as_file_mut(),
            "NotBad	SRR8494940.65223	2706	1131,0,1131;16,2690,2706
NotCovered	SRR8494940.141626	30116	326,0,326;27159,2957,30116
Chimeric	SRR8494940.91655	15691	151,0,151;4056,7213,11269;58,15633,15691"
        )
        .expect("Error durring write of report in temp file");

        let mut stack = FromReport::new(report.into_temp_path().to_str().unwrap())
            .expect("Error when create stack object");

        assert_eq!(
            [
                "SRR8494940.65223".to_string(),
                "SRR8494940.141626".to_string(),
                "SRR8494940.91655".to_string()
            ]
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<String>>(),
            stack.get_reads()
        );

        assert_eq!(
            &(vec![(0, 1131), (2690, 2706)], 2706),
            stack.get_bad_part("SRR8494940.65223").unwrap()
        );
        assert_eq!(
            &(vec![(0, 326), (2957, 30116)], 30116),
            stack.get_bad_part("SRR8494940.141626").unwrap()
        );
        assert_eq!(
            &(vec![(0, 151), (7213, 11269), (15633, 15691)], 15691),
            stack.get_bad_part("SRR8494940.91655").unwrap()
        );
    }

    #[test]
    fn from_overlap() {
        let mut ovl = reads2ovl::FullMemory::new();

        ovl.add_overlap("A".to_string(), (10, 990)).unwrap();
        ovl.add_length("A".to_string(), 1000);

        ovl.add_overlap("B".to_string(), (10, 90)).unwrap();
        ovl.add_length("B".to_string(), 1000);

        ovl.add_overlap("C".to_string(), (10, 490)).unwrap();
        ovl.add_overlap("C".to_string(), (510, 990)).unwrap();
        ovl.add_length("C".to_string(), 1000);

        ovl.add_overlap("D".to_string(), (0, 990)).unwrap();
        ovl.add_length("D".to_string(), 1000);

        ovl.add_overlap("E".to_string(), (10, 1000)).unwrap();
        ovl.add_length("E".to_string(), 1000);

        ovl.add_overlap("F".to_string(), (0, 490)).unwrap();
        ovl.add_overlap("F".to_string(), (510, 1000)).unwrap();
        ovl.add_length("F".to_string(), 1000);

        let mut stack = FromOverlap::new(Box::new(ovl), 0);

        assert_eq!(
            [
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
                "E".to_string(),
                "F".to_string()
            ]
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<String>>(),
            stack.get_reads()
        );

        assert_eq!(
            &(vec![(0, 10), (990, 1000)], 1000),
            stack.get_bad_part("A").unwrap()
        );
        assert_eq!(
            &(vec![(0, 10), (90, 1000)], 1000),
            stack.get_bad_part("B").unwrap()
        );
        assert_eq!(
            &(vec![(0, 10), (490, 510), (990, 1000)], 1000),
            stack.get_bad_part("C").unwrap()
        );
        assert_eq!(&(vec![(990, 1000)], 1000), stack.get_bad_part("D").unwrap());
        assert_eq!(&(vec![(0, 10)], 1000), stack.get_bad_part("E").unwrap());
        assert_eq!(&(vec![(490, 510)], 1000), stack.get_bad_part("F").unwrap());
    }

    #[test]
    fn coverage_upper_than_0() {
        let mut ovl = reads2ovl::FullMemory::new();

        ovl.add_length("A".to_string(), 1000);

        ovl.add_overlap("A".to_string(), (0, 425)).unwrap();
        ovl.add_overlap("A".to_string(), (0, 450)).unwrap();
        ovl.add_overlap("A".to_string(), (0, 475)).unwrap();

        ovl.add_overlap("A".to_string(), (525, 1000)).unwrap();
        ovl.add_overlap("A".to_string(), (550, 1000)).unwrap();
        ovl.add_overlap("A".to_string(), (575, 1000)).unwrap();

        let mut stack = FromOverlap::new(Box::new(ovl), 2);

        assert_eq!(&(vec![(425, 575)], 1000), stack.get_bad_part("A").unwrap());
    }

    #[test]
    fn failled_correctly_on_corrupt_yacrd() {
        let mut report = NamedTempFile::new().expect("Can't create tmpfile");

        writeln!(
            report.as_file_mut(),
            "NotBad	SRR8494940.65223	2706	1131,0,1131;16,2690,2706
NotCovered	SRR8494940.141626	30116	326,0,326;27159,2957,30116
Chimeric	SRR8494940.91655	15691	151,0,151;4056,7213,11269;58,156"
        )
        .unwrap();

        let stack = FromReport::new(report.into_temp_path().to_str().unwrap());

        if !stack.is_err() {
            assert!(false);
        }
    }
}
