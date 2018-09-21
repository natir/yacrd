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
use chimera;
use file;
use io;
use utils;

/* crates use */
use bio;
use std::path::Path;

/* standard use */
use std;

pub fn run(reads: &chimera::BadReadMap, filename: &str, extract_suffix: &str) {
    let filterd_name = &generate_extracted_name(filename, extract_suffix);

    let (raw_input, compression) = file::get_readable_file(filename);
    let input = Box::new(raw_input);
    let output = Box::new(file::get_output(filterd_name, compression));

    match utils::get_format(filename).unwrap() {
        utils::Format::Paf => extract_paf(reads, input, output),
        utils::Format::Mhap => extract_mhap(reads, input, output),
        utils::Format::Fasta => extract_fasta(reads, input, output),
        utils::Format::Fastq => extract_fastq(reads, input, output),
    }
}

fn generate_extracted_name(filename: &str, filterd_suffix: &str) -> String {
    let path = Path::new(filename);
    let mut filename = path.file_name().unwrap().to_str().unwrap().to_string();
    
    filename = filename.replacen(".", &format!("{}.", filterd_suffix), 1);

    let mut buffer = path.to_path_buf();
    buffer.set_file_name(filename);

    return buffer.to_str().unwrap().to_string();
}

fn extract_paf<R: std::io::Read, W: std::io::Write>(
    reads: &chimera::BadReadMap,
    input: R,
    output: W,
) {
    let mut reader = io::paf::Reader::new(input);
    let mut writer = io::paf::Writer::new(output);

    for result in reader.records() {
        let record = result.unwrap();
        if reads.contains_key(&record.read_a) || reads.contains_key(&record.read_b) {
            writer.write(&record).unwrap();
        }
    }
}

fn extract_mhap<R: std::io::Read, W: std::io::Write>(
    reads: &chimera::BadReadMap,
    input: R,
    output: W,
) {
    let mut reader = io::mhap::Reader::new(input);
    let mut writer = io::mhap::Writer::new(output);

    for result in reader.records() {
        let record = result.unwrap();
        if reads.contains_key(&record.read_a) || reads.contains_key(&record.read_b) {
            writer.write(&record).unwrap();
        }
    }
}

fn extract_fasta<R: std::io::Read, W: std::io::Write>(
    reads: &chimera::BadReadMap,
    input: R,
    output: W,
) {
    let reader = bio::io::fasta::Reader::new(input);
    let mut writer = bio::io::fasta::Writer::new(output);

    for r in reader.records() {
        let record = r.expect("Trouble in fasta parsing process");
        if reads.contains_key(record.id()) {
            writer.write_record(&record).expect(
                "Trouble durring fasta valid sequence writing",
            );
        }
    }
}

fn extract_fastq<R: std::io::Read, W: std::io::Write>(
    reads: &chimera::BadReadMap,
    input: R,
    output: W,
) {
    let reader = bio::io::fastq::Reader::new(input);
    let mut writer = bio::io::fastq::Writer::new(output);

    for r in reader.records() {
        let record = r.expect("Trouble in fastq parsing process");
        if reads.contains_key(record.id()) {
            writer.write_record(&record).expect(
                "Trouble durring fasta valid sequence writing",
            );
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use std::collections::HashMap;

    #[test]
    fn filtred_name() {
        assert_eq!(
            generate_extracted_name("test.paf", "_test"),
            "test_test.paf"
        );
        assert_eq!(
            generate_extracted_name("test.paf.gz", "_test"),
            "test_test.paf.gz"
        );
        assert_eq!(
            generate_extracted_name("test.fasta", "_filtred"),
            "test_filtred.fasta"
        );
        assert_eq!(
            generate_extracted_name("../something/test.fasta", "_filtred"),
            "../something/test_filtred.fasta"
        );
        assert_eq!(
            generate_extracted_name("../something.other/test.fasta", "_filtred"),
            "../something.other/test_filtred.fasta"
        );
    }

    lazy_static! {
        static ref REMOVE_READS: chimera::BadReadMap = {
            let mut m = HashMap::new();
            m.insert(
                "1".to_string(),
                (
                    chimera::BadReadType::Chimeric,
                    vec![chimera::Interval {
                        begin: 4500,
                        end: 5500,
                    }],
                ),
            );
            m
        };
    }

    const PAF_FILE: &'static [u8] = b"1\t12000\t20\t4500\t-\t2\t10000\t5500\t10000\t4500\t4500\t255
1\t12000\t5500\t10000\t-\t3\t10000\t0\t4500\t4500\t4500\t255
";

    const PAF_FILE_EXTRACTED: &'static [u8] = b"1\t12000\t20\t4500\t-\t2\t10000\t5500\t10000\t4500\t4500\t255
1\t12000\t5500\t10000\t-\t3\t10000\t0\t4500\t4500\t4500\t255
";

    #[test]
    fn paf() {
        let mut writer: Vec<u8> = Vec::new();

        extract_paf(&REMOVE_READS, PAF_FILE, &mut writer);

        assert_eq!(writer, PAF_FILE_EXTRACTED);
    }

    const MHAP_FILE: &'static [u8] = b"1 2 0.1 2 0 100 450 1000 0 550 900 1000
1 3 0.1 2 0 550 900 1000 0 100 450 1000
";

    const MHAP_FILE_EXTRACTED: &'static [u8] = b"1 2 0.1 2 0 100 450 1000 0 550 900 1000
1 3 0.1 2 0 550 900 1000 0 100 450 1000
";

    #[test]
    fn mhap() {
        let mut writer: Vec<u8> = Vec::new();

        extract_mhap(&REMOVE_READS, MHAP_FILE, &mut writer);

        assert_eq!(writer, MHAP_FILE_EXTRACTED);
    }

    const FASTA_FILE: &'static [u8] = b">1
ACTG
>2
ACTG
>3
ACTG
";

    const FASTA_FILE_EXTRACTED: &'static [u8] = b">1
ACTG
";

    #[test]
    fn fasta() {
        let mut writer: Vec<u8> = Vec::new();

        extract_fasta(&REMOVE_READS, FASTA_FILE, &mut writer);

        assert_eq!(writer, FASTA_FILE_EXTRACTED);
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

    const FASTQ_FILE_FILTRED: &'static [u8] = b"@1
ACTG
+
!!!!
";

    #[test]
    fn fastq() {
        let mut writer: Vec<u8> = Vec::new();

        extract_fastq(&REMOVE_READS, FASTQ_FILE, &mut writer);

        assert_eq!(writer, FASTQ_FILE_FILTRED);
    }

}
