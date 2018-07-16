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

/* project use */
use file;
use io;
use utils;

/* crates use */
use bio;

/* standard use */
use std;
use std::collections::HashSet;

pub fn run(reads: &Box<HashSet<String>>, filename: &str, filterd_suffix: &str) {
    let filterd_name = &generate_filterd_name(filename.to_owned(), filterd_suffix);

    let (raw_input, compression) = file::get_readable_file(filename);
    let input = Box::new(raw_input);
    let output = Box::new(file::get_output(filterd_name, compression));

    match utils::get_format(filename).unwrap() {
        utils::Format::Paf => filterd_paf(reads, input, output),
        utils::Format::Mhap => filterd_mhap(reads, input, output),
        utils::Format::Fasta => filterd_fasta(reads, input, output),
        utils::Format::Fastq => filterd_fastq(reads, input, output),
    }
}

fn generate_filterd_name(filename: String, filterd_suffix: &str) -> String {
    return filename.replacen(".", &format!("{}.", filterd_suffix), 1);
}

fn filterd_paf<R: std::io::Read, W: std::io::Write>(
    reads: &Box<HashSet<String>>,
    input: R,
    output: W,
) {
    let mut reader = io::paf::Reader::new(input);
    let mut writer = io::paf::Writer::new(output);

    for result in reader.records() {
        let record = result.unwrap();
        if !(reads.contains(&record.read_a) || reads.contains(&record.read_b)) {
            writer.write(&record).unwrap();
        }
    }
}

fn filterd_mhap<R: std::io::Read, W: std::io::Write>(
    reads: &Box<HashSet<String>>,
    input: R,
    output: W,
) {
    let mut reader = io::mhap::Reader::new(input);
    let mut writer = io::mhap::Writer::new(output);

    for result in reader.records() {
        let record = result.unwrap();
        if !(reads.contains(&record.read_a) || reads.contains(&record.read_b)) {
            writer.write(&record).unwrap();
        }
    }
}

fn filterd_fasta<R: std::io::Read, W: std::io::Write>(
    reads: &Box<HashSet<String>>,
    input: R,
    output: W,
) {
    let reader = bio::io::fasta::Reader::new(input);
    let mut writer = bio::io::fasta::Writer::new(output);

    for r in reader.records() {
        let record = r.expect("Trouble in fasta parsing process");
        if !reads.contains(record.id()) {
            writer
                .write_record(&record)
                .expect("Trouble durring fasta valid sequence writing");
        }
    }
}

fn filterd_fastq<R: std::io::Read, W: std::io::Write>(
    reads: &Box<HashSet<String>>,
    input: R,
    output: W,
) {
    let reader = bio::io::fastq::Reader::new(input);
    let mut writer = bio::io::fastq::Writer::new(output);

    for r in reader.records() {
        let record = r.expect("Trouble in fastq parsing process");
        if !reads.contains(record.id()) {
            writer
                .write_record(&record)
                .expect("Trouble durring fasta valid sequence writing");
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn filtred_name() {
       assert_eq!(generate_filterd_name("test.paf".to_string(), "_test"), "test_test.paf"); 
       assert_eq!(generate_filterd_name("test.paf.gz".to_string(), "_test"), "test_test.paf.gz"); 
       assert_eq!(generate_filterd_name("test.fasta".to_string(), "_filtred"), "test_filtred.fasta"); 
    }

    lazy_static! {
        static ref REMOVE_READS: Box<HashSet<String>> = {
            let mut m = Box::new(HashSet::new());
            m.insert("1".to_string());
            m
        };
    }

    const PAF_FILE: &'static [u8] = b"1\t12000\t20\t4500\t-\t2\t10000\t5500\t10000\t4500\t4500\t255
1\t12000\t5500\t10000\t-\t3\t10000\t0\t4500\t4500\t4500\t255
";
    
    const PAF_FILE_FILTRED: &'static [u8] = b"";

    #[test]
    fn paf() {
        let mut writer: Vec<u8> = Vec::new();

        filterd_paf(&REMOVE_READS, PAF_FILE, &mut writer);

        assert_eq!(writer, PAF_FILE_FILTRED);
    }
    
    const MHAP_FILE: &'static [u8] = b"1 2 0.1 2 0 100 450 1000 0 550 900 1000
1 3 0.1 2 0 550 900 1000 0 100 450 1000
";
    
    const MHAP_FILE_FILTRED: &'static [u8] = b"";
    
    #[test]
    fn mhap() {
        let mut writer: Vec<u8> = Vec::new();

        filterd_mhap(&REMOVE_READS, MHAP_FILE, &mut writer);

        assert_eq!(writer, MHAP_FILE_FILTRED);
    }

    const FASTA_FILE: &'static [u8] = b">1
ACTG
>2
ACTG
>3
ACTG
";

    const FASTA_FILE_FILTRED: &'static [u8] = b">2
ACTG
>3
ACTG
";
    
    #[test]
    fn fasta() {
        let mut writer: Vec<u8> = Vec::new();

        filterd_fasta(&REMOVE_READS, FASTA_FILE, &mut writer);

        assert_eq!(writer, FASTA_FILE_FILTRED);
    }
    
    const FASTQ_FILE: &'static [u8] = b"@1
ACTG
+
!!!!
@2
ACTG
+
!!!!
@3
ACTG
+
!!!!
";

    const FASTQ_FILE_FILTRED: &'static [u8] = b"@2
ACTG
+
!!!!
@3
ACTG
+
!!!!
";
    
    #[test]
    fn fastq() {
        let mut writer: Vec<u8> = Vec::new();

        filterd_fastq(&REMOVE_READS, FASTQ_FILE, &mut writer);

        assert_eq!(writer, FASTQ_FILE_FILTRED);
    }

}
