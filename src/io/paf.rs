/*
Copyright (c) 2018 Pierre Marijon <pierre.marijon@inria.fr>

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

/* local use */

/* crates use */
use csv;

/* standard use */
use std;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Record {
    pub read_a: String,
    pub length_a: u64,
    pub begin_a: u64,
    pub end_a: u64,
    pub strand: char,
    pub read_b: String,
    pub length_b: u64,
    pub begin_b: u64,
    pub end_b: u64,
    pub nb_match_base: u64,
    pub nb_base: u64,
    pub mapping_quality: u64,
    pub sam_field: Vec<String>,
}

type RecordInner = (String, u64, u64, u64, char, String, u64, u64, u64, u64, u64, Vec<String>);

pub struct Records<'a, R: 'a + std::io::Read> {
    inner: csv::DeserializeRecordsIter<'a, R, RecordInner>,
}

impl<'a, R: std::io::Read> Iterator for Records<'a, R> {
    type Item = csv::Result<Record>;

    fn next(&mut self) -> Option<csv::Result<Record>> {
        self.inner.next().map(|res| {
            res.map(|(read_a,
              length_a,
              begin_a,
              end_a,
              strand,
              read_b,
              length_b,
              begin_b,
              end_b,
              nb_match_base,
              nb_base,
              mapping_quality_and_sam)| {
                let mapping_quality = mapping_quality_and_sam[0].parse::<u64>().unwrap();

                let mut sam_field = Vec::new();
                if mapping_quality_and_sam.len() > 1 {
                    sam_field = mapping_quality_and_sam[1..].to_vec();
                }

                Record {
                    read_a,
                    length_a,
                    begin_a,
                    end_a,
                    strand,
                    read_b,
                    length_b,
                    begin_b,
                    end_b,
                    nb_match_base,
                    nb_base,
                    mapping_quality,
                    sam_field,
                }
            })
        })
    }
}

pub struct Reader<R: std::io::Read> {
    inner: csv::Reader<R>,
}

impl<R: std::io::Read> Reader<R> {
    pub fn new(reader: R) -> Self {
        Reader {
            inner: csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .has_headers(false)
                .flexible(true)
                .from_reader(reader),
        }
    }

    /// Iterate over all records.
    pub fn records(&mut self) -> Records<R> {
        Records { inner: self.inner.deserialize() }
    }
}

#[derive(Debug)]
pub struct Writer<W: std::io::Write> {
    inner: csv::Writer<W>,
}

impl Writer<fs::File> {
    /// Write to a given file path in given format.
    #[allow(dead_code)]
    pub fn to_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        fs::File::create(path).map(|f| Writer::new(f))
    }
}

impl<W: std::io::Write> Writer<W> {
    /// Write to a given writer.
    pub fn new(writer: W) -> Self {
        Writer {
            inner: csv::WriterBuilder::new()
                .delimiter(b'\t')
                .has_headers(false)
                .flexible(true)
                .from_writer(writer),
        }
    }

    /// Write a given GFF record.
    pub fn write(&mut self, record: &Record) -> csv::Result<()> {
        self.inner.serialize((
            &record.read_a,
            record.length_a,
            record.begin_a,
            record.end_a,
            record.strand,
            &record.read_b,
            record.length_b,
            record.begin_b,
            record.end_b,
            record.nb_match_base,
            record.nb_base,
            record.mapping_quality,
            &record.sam_field,
        ))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    const PAF_FILE: &'static [u8] = b"1\t12000\t20\t4500\t-\t2\t10000\t5500\t10000\t4500\t4500\t255
1\t12000\t5500\t10000\t-\t3\t10000\t0\t4500\t4500\t4500\t255
";

    const PAF_SAM_FIELD_FILE: &'static [u8] = b"1\t12000\t20\t4500\t-\t2\t10000\t5500\t10000\t4500\t4500\t255\tam:I:5
1\t12000\t5500\t10000\t-\t3\t10000\t0\t4500\t4500\t4500\t255\ttest:B:true\tam:I:5
";

    const READ_A: &'static [&str; 2] = &["1", "1"];
    const LENGTH_A: &'static [u64; 2] = &[12000, 12000];
    const BEGIN_A: &'static [u64; 2] = &[20, 5500];
    const END_A: &'static [u64; 2] = &[4500, 10000];
    const STRAND: &'static [char; 2] = &['-', '-'];
    const READ_B: &'static [&str; 2] = &["2", "3"];
    const LENGTH_B: &'static [u64; 2] = &[10000, 10000];
    const BEGIN_B: &'static [u64; 2] = &[5500, 0];
    const END_B: &'static [u64; 2] = &[10000, 4500];
    const NB_MATCH_BASE: &'static [u64; 2] = &[4500, 4500];
    const NB_BASE: &'static [u64; 2] = &[4500, 4500];
    const MAPPING_QUALITY: &'static [u64; 2] = &[255, 255];

    #[test]
    fn read() {
        let mut reader = Reader::new(PAF_FILE);

        let sam_field: [Vec<String>; 2] = [Vec::new(), Vec::new()];

        for (i, r) in reader.records().enumerate() {
            let record = r.unwrap();

            assert_eq!(record.read_a, READ_A[i]);
            assert_eq!(record.length_a, LENGTH_A[i]);
            assert_eq!(record.begin_a, BEGIN_A[i]);
            assert_eq!(record.end_a, END_A[i]);
            assert_eq!(record.strand, STRAND[i]);
            assert_eq!(record.read_b, READ_B[i]);
            assert_eq!(record.length_b, LENGTH_B[i]);
            assert_eq!(record.begin_b, BEGIN_B[i]);
            assert_eq!(record.end_b, END_B[i]);
            assert_eq!(record.nb_match_base, NB_MATCH_BASE[i]);
            assert_eq!(record.nb_base, NB_BASE[i]);
            assert_eq!(record.mapping_quality, MAPPING_QUALITY[i]);
            assert_eq!(record.sam_field, sam_field[i]);
        }
    }

    #[test]
    fn read_sam_field() {
        let mut reader = Reader::new(PAF_SAM_FIELD_FILE);

        let sam_field = &[vec!["am:I:5"], vec!["test:B:true", "am:I:5"]];

        for (i, r) in reader.records().enumerate() {
            let record = r.unwrap();

            assert_eq!(record.read_a, READ_A[i]);
            assert_eq!(record.length_a, LENGTH_A[i]);
            assert_eq!(record.begin_a, BEGIN_A[i]);
            assert_eq!(record.end_a, END_A[i]);
            assert_eq!(record.strand, STRAND[i]);
            assert_eq!(record.read_b, READ_B[i]);
            assert_eq!(record.length_b, LENGTH_B[i]);
            assert_eq!(record.begin_b, BEGIN_B[i]);
            assert_eq!(record.end_b, END_B[i]);
            assert_eq!(record.nb_match_base, NB_MATCH_BASE[i]);
            assert_eq!(record.nb_base, NB_BASE[i]);
            assert_eq!(record.mapping_quality, MAPPING_QUALITY[i]);
            assert_eq!(record.sam_field, sam_field[i]);
        }
    }

    #[test]
    fn write() {
        let mut reader = Reader::new(PAF_FILE);
        let mut writer = Writer::new(vec![]);
        for r in reader.records() {
            writer
                .write(&r.ok().expect("Error reading record"))
                .ok()
                .expect("Error writing record");
        }
        assert_eq!(writer.inner.into_inner().unwrap(), PAF_FILE);
    }

    #[test]
    fn write_sam_field() {
        let mut reader = Reader::new(PAF_SAM_FIELD_FILE);
        let mut writer = Writer::new(vec![]);
        for r in reader.records() {
            writer
                .write(&r.ok().expect("Error reading record"))
                .ok()
                .expect("Error writing record");
        }
        assert_eq!(writer.inner.into_inner().unwrap(), PAF_SAM_FIELD_FILE);
    }
}
