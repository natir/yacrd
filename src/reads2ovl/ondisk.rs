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

use std::io::Write;

/* crate use */
use anyhow::{anyhow, Context, Result};

/* local use */
use error;
use reads2ovl;
use util;

pub struct OnDisk {
    reads2ovl: std::collections::HashMap<String, Vec<(u32, u32)>>,
    reads2len: std::collections::HashMap<String, usize>,
    prefix: String,
    number_of_value: u64,
    buffer_size: u64,
}

impl OnDisk {
    pub fn new(prefix: String, buffer_size: u64) -> Self {
        OnDisk {
            reads2ovl: std::collections::HashMap::new(),
            reads2len: std::collections::HashMap::new(),
            prefix,
            number_of_value: 0,
            buffer_size,
        }
    }

    fn clean_buffer(&mut self) -> Result<()> {
        info!(
            "Clear cache, number of value in cache is {}",
            self.number_of_value
        );

        for (key, values) in self.reads2ovl.iter_mut() {
            let filename = format!("{}{}.yovl", self.prefix, key);
            let raw_out = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&filename)
                .with_context(|| error::Error::CantWriteFile {
                    filename: filename.clone(),
                })?;

            let mut output = std::io::BufWriter::new(raw_out);

            for v in values.iter() {
                writeln!(output, "{},{}", v.0, v.1).with_context(|| {
                    error::Error::WritingError {
                        filename: filename.clone(),
                        format: util::FileType::YacrdOverlap,
                    }
                })?;
            }

            values.clear();
        }

        self.number_of_value = 0;

        Ok(())
    }
}

impl reads2ovl::Reads2Ovl for OnDisk {
    fn init(&mut self, filename: &str) -> Result<()> {
        self.sub_init(filename)?;

        self.clean_buffer()
            .with_context(|| anyhow!("Error durring creation of tempory file"))?;
        self.number_of_value = 0;

        Ok(())
    }

    fn overlap(&self, id: &str) -> Result<Vec<(u32, u32)>> {
        let filename = format!("{}{}.yovl", self.prefix, id);
        if std::path::Path::new(&filename).exists() {
            let mut reader = csv::ReaderBuilder::new()
                .delimiter(b',')
                .has_headers(false)
                .from_reader(std::io::BufReader::new(
                    std::fs::File::open(&filename).with_context(|| error::Error::CantReadFile {
                        filename: filename.clone(),
                    })?,
                ));

            let mut ovls = Vec::new();
            for record in reader.records() {
                let result = record.with_context(|| error::Error::ReadingError {
                    filename: filename.clone(),
                    format: util::FileType::YacrdOverlap,
                })?;

                ovls.push((util::str2u32(&result[0])?, util::str2u32(&result[1])?));
            }

            Ok(ovls)
        } else {
            Ok(Vec::new())
        }
    }

    fn length(&self, id: &str) -> usize {
        *self.reads2len.get(&id.to_string()).unwrap_or(&0)
    }

    fn add_overlap(&mut self, id: String, ovl: (u32, u32)) -> Result<()> {
        self.reads2ovl.entry(id).or_insert_with(Vec::new).push(ovl);

        self.number_of_value += 1;

        if self.number_of_value >= self.buffer_size {
            self.clean_buffer()?;
        }

        Ok(())
    }

    fn add_length(&mut self, id: String, length: usize) {
        self.reads2len.entry(id).or_insert(length);
    }

    fn get_reads(&self) -> std::collections::HashSet<String> {
        self.reads2len.keys().map(|x| x.to_string()).collect()
    }
}
