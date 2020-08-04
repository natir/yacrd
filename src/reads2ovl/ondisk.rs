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
use anyhow::{anyhow, Context, Result};
use log::info;

/* local use */
use crate::error;
use crate::reads2ovl;

pub struct OnDisk {
    reads2ovl: rustc_hash::FxHashMap<String, Vec<(u32, u32)>>,
    reads2len: rustc_hash::FxHashMap<String, usize>,
    db: sled::Db,
    number_of_value: u64,
    buffer_size: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct OnDiskRecord {
    begin: u32,
    end: u32,
}

impl OnDisk {
    pub fn new(on_disk_path: String, buffer_size: u64) -> Self {
        let path = std::path::PathBuf::from(on_disk_path.clone());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| error::Error::PathCreationError { path })
                .unwrap();
        }

        let db = sled::Config::default()
            .path(on_disk_path)
            .open()
            .with_context(|| error::Error::OnDiskOpen)
            .unwrap();

        OnDisk {
            reads2ovl: rustc_hash::FxHashMap::default(),
            reads2len: rustc_hash::FxHashMap::default(),
            db,
            number_of_value: 0,
            buffer_size,
        }
    }

    fn clean_buffer(&mut self) -> Result<()> {
        info!(
            "Clear cache, number of value in cache is {}",
            self.number_of_value
        );

        let mut batch = sled::Batch::default();

        for (key, vs) in self.reads2ovl.drain() {
            let new_val: Vec<(u32, u32)> = match self
                .db
                .get(key.as_bytes())
                .with_context(|| error::Error::OnDiskReadDatabase)?
            {
                Some(x) => {
                    let mut orig: Vec<(u32, u32)> = bincode::deserialize(&x)
                        .with_context(|| error::Error::OnDiskDeserializeVec)?;
                    orig.extend(vs);
                    orig
                }
                None => vs.to_vec(),
            };

            batch.insert(
                key.as_bytes(),
                bincode::serialize(&new_val).with_context(|| error::Error::OnDiskSerializeVec)?,
            );
        }

        self.db
            .apply_batch(batch)
            .with_context(|| error::Error::OnDiskBatchApplication)?;

        self.number_of_value = 0;

        Ok(())
    }

    fn _overlap(&self, id: &str) -> Result<Vec<(u32, u32)>> {
        Ok(bincode::deserialize(
            &self
                .db
                .get(id.as_bytes())
                .with_context(|| error::Error::OnDiskReadDatabase)?
                .unwrap(),
        )
        .with_context(|| error::Error::OnDiskDeserializeVec)?)
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

    fn get_overlaps(&mut self, new: &mut reads2ovl::MapReads2Ovl) -> bool {
        let mut tmp = rustc_hash::FxHashMap::default();

        if self.reads2len.is_empty() {
            std::mem::swap(&mut tmp, new);
            return true;
        }

        let mut remove_reads = Vec::with_capacity(self.buffer_size as usize);

        for (k, v) in self.reads2len.iter().take(self.buffer_size as usize) {
            remove_reads.push(k.clone());
            tmp.insert(k.clone(), (self._overlap(k).unwrap(), *v));
        }

        for k in remove_reads {
            self.reads2len.remove(&k);
        }

        std::mem::swap(&mut tmp, new);
        false
    }

    fn overlap(&self, id: &str) -> Result<Vec<(u32, u32)>> {
        self._overlap(id)
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
