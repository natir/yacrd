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
use log::info;

/* local use */
use crate::error;
use crate::reads2ovl;
use crate::util;

pub struct OnDisk {
    reads2ovl: rustc_hash::FxHashMap<String, Vec<(u32, u32)>>,
    reads2len: rustc_hash::FxHashMap<String, usize>,
    prefix: String,
    number_of_value: u64,
    buffer_size: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct OnDiskRecord {
    begin: u32,
    end: u32,
}

impl OnDisk {
    pub fn new(prefix: String, buffer_size: u64) -> Self {
        OnDisk {
            reads2ovl: rustc_hash::FxHashMap::default(),
            reads2len: rustc_hash::FxHashMap::default(),
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
            let prefix = self.prefix.clone();
            let mut output = std::io::BufWriter::new(OnDisk::create_yacrd_ovl_file(&prefix, key)?);

            for v in values.iter() {
                writeln!(output, "{},{}", v.0, v.1).with_context(|| {
                    error::Error::WritingError {
                        filename: format!("{}{}", &prefix, key),
                        format: util::FileType::YacrdOverlap,
                    }
                })?;
            }

            values.clear();
        }

        self.number_of_value = 0;

        Ok(())
    }

    fn create_yacrd_ovl_file(prefix: &str, id: &str) -> Result<std::fs::File> {
        /* build path */
        let path = prefix_id2pathbuf(prefix, id);

        /* create parent directory if it's required */
        if let Some(parent_path) = path.parent() {
            std::fs::create_dir_all(parent_path).with_context(|| {
                error::Error::PathCreationError {
                    path: parent_path.to_path_buf(),
                }
            })?;
        }

        /* create file */
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| error::Error::CantWriteFile {
                filename: path.to_string_lossy().to_string(),
            })
    }

    fn _overlap(id: &str, prefix: &str) -> Result<Vec<(u32, u32)>> {
        let filename = format!("{}{}.yovl", prefix, id);
        if std::path::Path::new(&filename).exists() {
            let input =
                std::io::BufReader::new(std::fs::File::open(&filename).with_context(|| {
                    error::Error::CantReadFile {
                        filename: filename.clone(),
                    }
                })?);

            let mut reader = csv::ReaderBuilder::new()
                .delimiter(b',')
                .has_headers(false)
                .from_reader(input);

            let mut rec = csv::StringRecord::new();
            let mut ovls = Vec::new();
            while reader.read_record(&mut rec).unwrap() {
                let record: OnDiskRecord =
                    rec.deserialize(None)
                        .with_context(|| error::Error::ReadingError {
                            filename: filename.clone(),
                            format: util::FileType::YacrdOverlap,
                        })?;

                ovls.push((record.begin, record.end));
            }

            Ok(ovls)
        } else {
            Ok(Vec::new())
        }
    }
}

pub(crate) fn prefix_id2pathbuf(prefix: &str, id: &str) -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(prefix);
    path.push(id);
    path.set_extension("yovl");

    path
}

impl reads2ovl::Reads2Ovl for OnDisk {
    fn init(&mut self, filename: &str) -> Result<()> {
        self.sub_init(filename)?;

        self.clean_buffer()
            .with_context(|| anyhow!("Error durring creation of tempory file"))?;
        self.number_of_value = 0;

        Ok(())
    }

    fn get_overlaps(&mut self, new: &mut reads2ovl::MapReads2Ovl) -> bool {
        let mut tmp = rustc_hash::FxHashMap::default();

        if self.reads2len.is_empty() {
            std::mem::swap(&mut tmp, new);
            return true;
        }

        let prefix = self.prefix.clone();
        let mut remove_reads = Vec::with_capacity(self.buffer_size as usize);

        for (k, v) in self.reads2len.iter_mut().take(self.buffer_size as usize) {
            remove_reads.push(k.clone());
            tmp.insert(k.clone(), (OnDisk::_overlap(k, &prefix).unwrap(), *v));
        }

        for k in remove_reads {
            self.reads2len.remove(&k);
        }

        std::mem::swap(&mut tmp, new);
        false
    }

    fn overlap(&self, id: &str) -> Result<Vec<(u32, u32)>> {
        OnDisk::_overlap(id, &self.prefix)
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

    fn add_overlap_and_length(&mut self, id: String, ovl: (u32, u32), length: usize) -> Result<()> {
        self.add_length(id.clone(), length);

        self.add_overlap(id, ovl)
    }

    fn get_reads(&self) -> rustc_hash::FxHashSet<String> {
        self.reads2len.keys().map(|x| x.to_string()).collect()
    }
}
