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
use anyhow::Result;

/* local use */
use crate::reads2ovl;

pub struct FullMemory {
    reads2ovl: reads2ovl::MapReads2Ovl,
    no_overlap: Vec<(u32, u32)>,
}

impl FullMemory {
    pub fn new() -> Self {
        FullMemory {
            reads2ovl: rustc_hash::FxHashMap::default(),
            no_overlap: Vec::new(),
        }
    }
}

impl reads2ovl::Reads2Ovl for FullMemory {
    fn get_overlaps(&mut self, new: &mut reads2ovl::MapReads2Ovl) -> bool {
        std::mem::swap(&mut self.reads2ovl, new);

        true
    }

    fn overlap(&self, id: &str) -> Result<Vec<(u32, u32)>> {
        if let Some((vec, _)) = self.reads2ovl.get(&id.to_string()) {
            Ok(vec.to_vec())
        } else {
            Ok(self.no_overlap.to_vec())
        }
    }

    fn length(&self, id: &str) -> usize {
        if let Some((_, len)) = self.reads2ovl.get(&id.to_string()) {
            *len
        } else {
            0
        }
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
}
