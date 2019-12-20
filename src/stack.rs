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
use error;
use reads2ovl;
use util;

pub trait BadPart {
    fn get_bad_part(&mut self, id: &str) -> Result<&(Vec<(u32, u32)>, usize)>;

    fn get_reads(&self) -> Vec<String>;
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

        let ovl = self.ovl.overlap(&id)?;
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

        Ok((gaps, length))
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

    fn get_reads(&self) -> Vec<String> {
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
        for record in reader.records() {
            let result = record.with_context(|| error::Error::ReadingError {
                filename: input_path.to_string(),
                format: util::FileType::Fasta,
            })?;

            let id = result[1].to_string();
            let len = util::str2usize(&result[2])?;
            let bad_part = FromReport::parse_bad_string(&result[3])?;

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
                        .with_context(|| error::Error::NotReachableCode {
                            name: format!("{} {}", file!(), line!()),
                        })?,
                )?,
                util::str2u32(
                    iter.next()
                        .with_context(|| error::Error::NotReachableCode {
                            name: format!("{} {}", file!(), line!()),
                        })?,
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

    fn get_reads(&self) -> Vec<String> {
        self.buffer.keys().map(|x| x.to_string()).collect()
    }
}
