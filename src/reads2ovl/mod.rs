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

/* local mod */
pub mod fullmemory;
pub mod index;

/* stuff declare in submod need to be accessible from mod level */
pub use self::fullmemory::*;
pub use self::index::*;

/* std use */
pub use self::fullmemory::*;

/* local use */

pub type MapReads2Ovl = rustc_hash::FxHashMap<String, (Vec<(u32, u32)>, usize)>;

pub trait Reads2Ovl {
    fn init(&mut self, filename: &str) -> Result<()>;

    fn get_overlaps(&mut self, new: &mut MapReads2Ovl) -> bool;

    fn add_overlap(&mut self, id: String, ovl: (u32, u32)) -> Result<()>;
    fn add_length(&mut self, id: String, ovl: usize);

    fn add_overlap_and_length(&mut self, id: String, ovl: (u32, u32), length: usize) -> Result<()>;

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

        let mut overlaps = MapReads2Ovl::default();
        ovl.get_overlaps(&mut overlaps);

        assert_eq!(
            vec![(20, 4500), (5500, 10000)],
            overlaps.get("1").unwrap().0
        );
        assert_eq!(vec![(5500, 10000)], overlaps.get("2").unwrap().0);
        assert_eq!(vec![(0, 4500)], overlaps.get("3").unwrap().0);
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

        let mut overlaps = MapReads2Ovl::default();
        ovl.get_overlaps(&mut overlaps);

        assert_eq!(
            vec![(20, 4500), (5500, 10000)],
            overlaps.get("1").unwrap().0
        );
        assert_eq!(vec![(5500, 10000)], overlaps.get("2").unwrap().0);
        assert_eq!(vec![(0, 4500)], overlaps.get("3").unwrap().0);
    }
}
