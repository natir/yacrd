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

use std::mem::MaybeUninit;

/* crate use */
use anyhow::{anyhow, Context, Result};
use log::info;

/* local use */
use crate::error;
use crate::io;
use crate::reads2ovl;
use crate::reads2ovl::MapReads2Ovl;
use crate::util;

pub struct Index {
    index: csv::Reader<std::io::BufReader<std::fs::File>>,
    reader: MaybeUninit<csv::Reader<std::io::BufReader<std::fs::File>>>,
    buffer_size: u64,
    read_buffer_size: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct IndexRecord<'a> {
    id: &'a str,
    offsets: Vec<(u64, u64)>,
}

impl Index {
    pub fn new(index_path: String, buffer_size: u64, read_buffer_size: usize) -> Self {
        let index = csv::ReaderBuilder::new()
            .delimiter(b',')
            .has_headers(false)
            .flexible(true)
            .from_reader(std::io::BufReader::new(
                std::fs::File::open(&index_path)
                    .with_context(|| error::Error::CantReadFile {
                        filename: index_path.clone(),
                    })
                    .unwrap(),
            ));

        Self {
            index,
            reader: MaybeUninit::uninit(),
            buffer_size,
            read_buffer_size,
        }
    }
}

impl reads2ovl::Reads2Ovl for Index {
    fn init(&mut self, filename: &str) -> Result<()> {
        self.reader = MaybeUninit::new(
            csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .has_headers(false)
                .flexible(true)
                .from_reader(std::io::BufReader::new(
                    std::fs::File::open(filename).with_context(|| error::Error::CantReadFile {
                        filename: filename.to_string(),
                    })?,
                )),
        );

        Ok(())
    }

    fn get_overlaps(&mut self, new: &mut MapReads2Ovl) -> bool {
        info!("Call get_overlaps");
        let mut overlaps = MapReads2Ovl::default();

        let paf_reader = unsafe { &mut *self.reader.as_mut_ptr() };
        let mut pos = csv::Position::new();
        let mut indexrec = csv::StringRecord::new();
        let mut pafrec = csv::StringRecord::new();

        for _ in 0..self.buffer_size {
            let index_out = self
                .index
                .read_record(&mut indexrec)
                .with_context(|| error::Error::ReadingErrorNoFilename {
                    format: util::FileType::Paf,
                })
                .unwrap();

            if !index_out {
                std::mem::swap(&mut overlaps, new);
                info!("Finish");
                return true;
            }

            let i_record: IndexRecord = indexrec
                .deserialize(None)
                .with_context(|| error::Error::ReadingErrorNoFilename {
                    format: util::FileType::Paf,
                })
                .unwrap();

            for poss in i_record.offsets {
                pos.set_byte(poss.0);
                paf_reader.seek(pos.clone()).expect("TODO");

                while paf_reader.read_record(&mut pafrec).unwrap() {
                    let p_record: io::PafRecord = pafrec
                        .deserialize(None)
                        .with_context(|| error::Error::ReadingErrorNoFilename {
                            format: util::FileType::Paf,
                        })
                        .unwrap();

                    if p_record.read_a == i_record.id {
                        overlaps
                            .entry(i_record.id.to_string())
                            .or_insert((Vec::new(), p_record.length_a))
                            .0
                            .push((p_record.begin_a, p_record.end_a));
                    } else if p_record.read_b == i_record.id {
                        overlaps
                            .entry(i_record.id.to_string())
                            .or_insert((Vec::new(), p_record.length_b))
                            .0
                            .push((p_record.begin_b, p_record.end_b));
                    } else {
                        break;
                    }
                }
            }
        }

        info!("Not Finish");
        std::mem::swap(&mut overlaps, new);
        false
    }

    fn add_overlap(&mut self, id: String, ovl: (u32, u32)) -> Result<()> {
        Ok(())
    }

    fn add_length(&mut self, id: String, ovl: usize) {}

    fn add_overlap_and_length(&mut self, id: String, ovl: (u32, u32), length: usize) -> Result<()> {
        Ok(())
    }

    fn get_reads(&self) -> rustc_hash::FxHashSet<String> {
        rustc_hash::FxHashSet::default()
    }

    fn read_buffer_size(&self) -> usize {
        self.read_buffer_size
    }
}
