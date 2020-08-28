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

/* local use */
use crate::error;
use crate::io;
use crate::reads2ovl;
use crate::reads2ovl::Reads2Ovl;
use crate::util;

pub struct FullMemory {
    reads2ovl: reads2ovl::MapReads2Ovl,
    no_overlap: Vec<(u32, u32)>,
    read_buffer_size: usize,
}

impl FullMemory {
    pub fn new(read_buffer_size: usize) -> Self {
        FullMemory {
            reads2ovl: rustc_hash::FxHashMap::default(),
            no_overlap: Vec::new(),
            read_buffer_size,
        }
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
}

impl reads2ovl::Reads2Ovl for FullMemory {
    fn init(&mut self, filename: &str) -> Result<()> {
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

    fn get_overlaps(&mut self, new: &mut reads2ovl::MapReads2Ovl) -> bool {
        std::mem::swap(&mut self.reads2ovl, new);

        true
    }

    fn add_overlap(&mut self, id: String, ovl: (u32, u32)) -> Result<()> {
        self.reads2ovl
            .entry(id)
            .or_insert((Vec::new(), 0))
            .0
            .push(ovl);

        Ok(())
    }

    fn add_length(&mut self, id: String, length: usize) {
        self.reads2ovl.entry(id).or_insert((Vec::new(), 0)).1 = length;
    }

    fn add_overlap_and_length(&mut self, id: String, ovl: (u32, u32), length: usize) -> Result<()> {
        if let Some(value) = self.reads2ovl.get_mut(&id) {
            value.0.push(ovl);
        } else {
            self.reads2ovl.insert(id, (vec![ovl], length));
        }

        Ok(())
    }

    fn get_reads(&self) -> rustc_hash::FxHashSet<String> {
        self.reads2ovl.keys().map(|x| x.to_string()).collect()
    }

    fn read_buffer_size(&self) -> usize {
        self.read_buffer_size
    }
}
