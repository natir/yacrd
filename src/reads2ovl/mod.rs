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

/* crate use */
use anyhow::{anyhow, bail, Context, Result};

/* local mod */
pub mod fullmemory;
pub mod ondisk;

/* stuff declare in submod need to be accessible from mod level */
pub use self::fullmemory::*;
pub use self::ondisk::*;

/* std use */

/* local use */
use crate::error;
use crate::io;
use crate::util;

pub type MapReads2Ovl = rustc_hash::FxHashMap<String, (Vec<(u32, u32)>, usize)>;

pub trait Reads2Ovl {
    fn init(&mut self, filename: &str) -> Result<()> {
        self.sub_init(filename)
    }

    fn sub_init(&mut self, filename: &str) -> Result<()> {
        let (input, _) = util::read_file(filename, self.read_buffer_size())?;

        match util::get_file_type(filename) {
            Some(util::FileType::Paf) => self
                .init_paf(input)
                .with_context(|| anyhow!("Filename: {}", filename.to_string()))?,
            Some(util::FileType::M4) => self
                .init_m4(input)
                .with_context(|| anyhow!("Filename: {}", filename.to_string()))?,
            Some(util::FileType::Fasta) => bail!(error::Error::CantRunOperationOnFile {
                operation: "overlap parsing".to_string(),
                filetype: util::FileType::Fasta,
                filename: filename.to_string()
            }),
            Some(util::FileType::Fastq) => bail!(error::Error::CantRunOperationOnFile {
                operation: "overlap parsing".to_string(),
                filetype: util::FileType::Fastq,
                filename: filename.to_string()
            }),
            Some(util::FileType::Yacrd) => bail!(error::Error::CantRunOperationOnFile {
                operation: "overlap parsing".to_string(),
                filetype: util::FileType::Yacrd,
                filename: filename.to_string()
            }),
            None | Some(util::FileType::YacrdOverlap) => {
                bail!(error::Error::UnableToDetectFileFormat {
                    filename: filename.to_string()
                })
            }
        }

        Ok(())
    }

    fn init_paf(&mut self, input: Box<dyn std::io::Read>) -> Result<()> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .flexible(true)
            .from_reader(input);

        let mut rec = csv::StringRecord::new();

        while reader.read_record(&mut rec).unwrap() {
            let record: io::PafRecord =
                rec.deserialize(None)
                    .with_context(|| error::Error::ReadingErrorNoFilename {
                        format: util::FileType::Paf,
                    })?;

            let id_a = record.read_a.to_string();
            let id_b = record.read_b.to_string();

            let len_a = record.length_a;
            let len_b = record.length_b;

            let ovl_a = (record.begin_a, record.end_a);
            let ovl_b = (record.begin_b, record.end_b);

            self.add_overlap_and_length(id_a, ovl_a, len_a)?;
            self.add_overlap_and_length(id_b, ovl_b, len_b)?;
        }

        Ok(())
    }

    fn init_m4(&mut self, input: Box<dyn std::io::Read>) -> Result<()> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b' ')
            .has_headers(false)
            .flexible(true)
            .from_reader(input);

        let mut rec = csv::StringRecord::new();

        while reader.read_record(&mut rec).unwrap() {
            let record: io::M4Record =
                rec.deserialize(None)
                    .with_context(|| error::Error::ReadingErrorNoFilename {
                        format: util::FileType::M4,
                    })?;

            let id_a = record.read_a.to_string();
            let id_b = record.read_b.to_string();

            let len_a = record.length_a;
            let len_b = record.length_b;

            let ovl_a = (record.begin_a, record.end_a);
            let ovl_b = (record.begin_b, record.end_b);

            self.add_overlap_and_length(id_a, ovl_a, len_a)?;
            self.add_overlap_and_length(id_b, ovl_b, len_b)?;
        }

        Ok(())
    }

    fn get_overlaps(&mut self, new: &mut MapReads2Ovl) -> bool;

    #[allow(dead_code)]
    fn overlap(&self, id: &str) -> Result<Vec<(u32, u32)>>;
    #[allow(dead_code)]
    fn length(&self, id: &str) -> usize;

    fn add_overlap(&mut self, id: String, ovl: (u32, u32)) -> Result<()>;
    fn add_length(&mut self, id: String, ovl: usize);

    fn add_overlap_and_length(&mut self, id: String, ovl: (u32, u32), length: usize) -> Result<()>;

    #[allow(dead_code)]
    fn get_reads(&self) -> rustc_hash::FxHashSet<String>;

    fn read_buffer_size(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;

    extern crate tempfile;

    const PAF_FILE: &'static [u8] = b"1\t12000\t20\t4500\t-\t2\t10000\t5500\t10000\t4500\t4500\t255
1\t12000\t5500\t10000\t-\t3\t10000\t0\t4500\t4500\t4500\t255
";

    const M4_FILE: &'static [u8] = b"1 2 0.1 2 0 20 4500 12000 0 5500 10000 10000
1 3 0.1 2 0 5500 10000 12000 0 0 4500 10000
";

    #[test]
    fn paf() {
        let mut paf = tempfile::Builder::new()
            .suffix(".paf")
            .tempfile()
            .expect("Can't create tmpfile");

        paf.as_file_mut()
            .write_all(PAF_FILE)
            .expect("Error durring write of paf in temp file");

        let mut ovl = FullMemory::new(8192);

        ovl.init(paf.into_temp_path().to_str().unwrap())
            .expect("Error in overlap init");

        assert_eq!(
            ["1".to_string(), "2".to_string(), "3".to_string(),]
                .iter()
                .cloned()
                .collect::<rustc_hash::FxHashSet<String>>(),
            ovl.get_reads()
        );

        assert_eq!(vec![(20, 4500), (5500, 10000)], ovl.overlap("1").unwrap());
        assert_eq!(vec![(5500, 10000)], ovl.overlap("2").unwrap());
        assert_eq!(vec![(0, 4500)], ovl.overlap("3").unwrap());
    }

    #[test]
    fn m4() {
        let mut m4 = tempfile::Builder::new()
            .suffix(".m4")
            .tempfile()
            .expect("Can't create tmpfile");

        m4.as_file_mut()
            .write_all(M4_FILE)
            .expect("Error durring write of m4 in temp file");

        let mut ovl = FullMemory::new(8192);

        ovl.init(m4.into_temp_path().to_str().unwrap())
            .expect("Error in overlap init");

        assert_eq!(
            ["1".to_string(), "2".to_string(), "3".to_string(),]
                .iter()
                .cloned()
                .collect::<rustc_hash::FxHashSet<String>>(),
            ovl.get_reads()
        );

        assert_eq!(vec![(20, 4500), (5500, 10000)], ovl.overlap("1").unwrap());
        assert_eq!(vec![(5500, 10000)], ovl.overlap("2").unwrap());
        assert_eq!(vec![(0, 4500)], ovl.overlap("3").unwrap());
    }
}
