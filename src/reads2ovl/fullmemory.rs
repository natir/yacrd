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
use reads2ovl;

pub struct FullMemory {
    reads2ovl: std::collections::HashMap<String, Vec<(u32, u32)>>,
    reads2len: std::collections::HashMap<String, usize>,
    no_overlap: Vec<(u32, u32)>,
}

impl FullMemory {
    pub fn new() -> Self {
        FullMemory {
            reads2ovl: std::collections::HashMap::new(),
            reads2len: std::collections::HashMap::new(),
            no_overlap: Vec::new(),
        }
    }
}

impl reads2ovl::Reads2Ovl for FullMemory {
    fn overlap(&self, id: &str) -> Result<Vec<(u32, u32)>> {
        Ok(self
            .reads2ovl
            .get(&id.to_string())
            .unwrap_or(&self.no_overlap)
            .to_vec())
    }

    fn length(&self, id: &str) -> usize {
        *self.reads2len.get(&id.to_string()).unwrap_or(&0)
    }

    fn add_overlap(&mut self, id: String, ovl: (u32, u32)) -> Result<()> {
        self.reads2ovl.entry(id).or_insert_with(Vec::new).push(ovl);

        Ok(())
    }

    fn add_length(&mut self, id: String, length: usize) {
        self.reads2len.entry(id).or_insert(length);
    }

    fn get_reads(&self) -> Vec<String> {
        self.reads2len.keys().map(|x| x.to_string()).collect()
    }
}
