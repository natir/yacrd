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

/* local mod */
pub mod extract;
pub mod filter;
pub mod scrubbing;
pub mod split;

/* stuff declare in submod need to be accessible from mod level */
pub use self::extract::*;
pub use self::filter::*;
pub use self::scrubbing::*;
pub use self::split::*;

/* crate use */
use anyhow::{Context, Result};

/* local use */
use crate::error;
use crate::util;

#[derive(Debug, PartialEq)]
pub enum ReadType {
    Chimeric,
    NotCovered,
    NotBad,
}

impl Eq for ReadType {}

impl ReadType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReadType::Chimeric => "Chimeric",
            ReadType::NotCovered => "NotCovered",
            ReadType::NotBad => "NotBad",
        }
    }
}

pub fn report<W>(
    read: &str,
    length: usize,
    badregions: &[(u32, u32)],
    not_covered: f64,
    out: &mut W,
) -> Result<()>
where
    W: std::io::Write,
{
    let readtype = type_of_read(length, badregions, not_covered);
    writeln!(
        out,
        "{}\t{}\t{}\t{}",
        readtype.as_str(),
        read,
        length,
        bad_region_format(badregions)
    )
    .with_context(|| error::Error::WritingErrorNoFilename {
        format: util::FileType::Yacrd,
    })
}

pub fn type_of_read(length: usize, badregions: &[(u32, u32)], not_covered: f64) -> ReadType {
    let bad_region_len = badregions.iter().fold(0, |acc, x| acc + (x.1 - x.0));

    if bad_region_len as f64 / length as f64 > not_covered {
        return ReadType::NotCovered;
    }

    let middle_gap = badregions
        .iter()
        .filter(|x| x.0 != 0 && x.1 != length as u32)
        .collect::<Vec<&(u32, u32)>>();
    if !middle_gap.is_empty() {
        return ReadType::Chimeric;
    }

    ReadType::NotBad
}

fn bad_region_format(bads: &[(u32, u32)]) -> String {
    bads.iter()
        .map(|b| format!("{},{},{}", b.1 - b.0, b.0, b.1))
        .collect::<Vec<String>>()
        .join(";")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_type_assignation() {
        let a = (vec![(0, 10), (990, 1000)], 1000);
        let b = (vec![(0, 10), (90, 1000)], 1000);
        let c = (vec![(0, 10), (490, 510), (990, 1000)], 1000);
        let d = (vec![(990, 1000)], 1000);
        let e = (vec![(0, 10)], 1000);
        let f = (vec![(490, 510)], 1000);

        assert_eq!(ReadType::NotBad, type_of_read(a.1, &a.0, 0.8));
        assert_eq!(ReadType::NotCovered, type_of_read(b.1, &b.0, 0.8));
        assert_eq!(ReadType::Chimeric, type_of_read(c.1, &c.0, 0.8));
        assert_eq!(ReadType::NotBad, type_of_read(d.1, &d.0, 0.8));
        assert_eq!(ReadType::NotBad, type_of_read(e.1, &e.0, 0.8));
        assert_eq!(ReadType::Chimeric, type_of_read(f.1, &f.0, 0.8));
    }
}
