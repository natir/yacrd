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
pub use self::fullmemory::*;

/* local use */
use crate::error;
use crate::util;

pub trait Reads2Ovl {
    fn init(&mut self, filename: &str) -> Result<()> {
        self.sub_init(filename)
    }

    fn sub_init(&mut self, filename: &str) -> Result<()> {
        let (input, _) = util::read_file(filename)?;

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
            .flexible(true)
            .has_headers(false)
            .from_reader(input);

        for record in reader.records() {
            let result = record.with_context(|| error::Error::ReadingErrorNoFilename {
                format: util::FileType::Paf,
            })?;

            if result.len() < 9 {
                bail!(error::Error::ReadingErrorNoFilename {
                    format: util::FileType::Paf,
                });
            }

            let id_a = result[0].to_string();
            let id_b = result[5].to_string();

            let len_a = util::str2usize(&result[1])?;
            let len_b = util::str2usize(&result[6])?;

            let ovl_a = (util::str2u32(&result[2])?, util::str2u32(&result[3])?);
            let ovl_b = (util::str2u32(&result[7])?, util::str2u32(&result[8])?);

            self.add_length(id_a.clone(), len_a);
            self.add_length(id_b.clone(), len_b);

            self.add_overlap(id_a, ovl_a)?;
            self.add_overlap(id_b, ovl_b)?;
        }

        Ok(())
    }

    fn init_m4(&mut self, input: Box<dyn std::io::Read>) -> Result<()> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b' ')
            .has_headers(false)
            .from_reader(input);

        for record in reader.records() {
            let result = record.with_context(|| error::Error::ReadingErrorNoFilename {
                format: util::FileType::M4,
            })?;

            if result.len() < 12 {
                bail!(error::Error::ReadingErrorNoFilename {
                    format: util::FileType::M4,
                });
            }

            let id_a = result[0].to_string();
            let id_b = result[1].to_string();

            let len_a = util::str2usize(&result[7])?;
            let len_b = util::str2usize(&result[11])?;

            let ovl_a = (util::str2u32(&result[5])?, util::str2u32(&result[6])?);
            let ovl_b = (util::str2u32(&result[9])?, util::str2u32(&result[10])?);

            self.add_length(id_a.clone(), len_a);
            self.add_length(id_b.clone(), len_b);

            self.add_overlap(id_a, ovl_a)?;
            self.add_overlap(id_b, ovl_b)?;
        }

        Ok(())
    }

    fn overlap(&self, id: &str) -> Result<Vec<(u32, u32)>>;
    fn length(&self, id: &str) -> usize;

    fn add_overlap(&mut self, id: String, ovl: (u32, u32)) -> Result<()>;
    fn add_length(&mut self, id: String, ovl: usize);

    fn get_reads(&self) -> std::collections::HashSet<String>;
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

        let mut ovl = FullMemory::new();

        ovl.init(paf.into_temp_path().to_str().unwrap())
            .expect("Error in overlap init");

        assert_eq!(
            ["1".to_string(), "2".to_string(), "3".to_string(),]
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<String>>(),
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
            .expect("Error durring write of paf in temp file");

        let mut ovl = FullMemory::new();

        ovl.init(m4.into_temp_path().to_str().unwrap())
            .expect("Error in overlap init");

        assert_eq!(
            ["1".to_string(), "2".to_string(), "3".to_string(),]
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<String>>(),
            ovl.get_reads()
        );

        assert_eq!(vec![(20, 4500), (5500, 10000)], ovl.overlap("1").unwrap());
        assert_eq!(vec![(5500, 10000)], ovl.overlap("2").unwrap());
        assert_eq!(vec![(0, 4500)], ovl.overlap("3").unwrap());
    }
}
