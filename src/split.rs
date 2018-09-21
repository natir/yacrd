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
use utils;

/* crates use */
use bio;

/* standard use */
use std;

pub fn run(reads: &chimera::BadReadMap, filename: &str, split_suffix: &str) {
    let split_name = &generate_split_name(filename.to_owned(), split_suffix);

    let (raw_input, compression) = file::get_readable_file(filename);

    let input = Box::new(raw_input);
    let output = Box::new(file::get_output(split_name, compression));

    match utils::get_format(filename).unwrap() {
        utils::Format::Fasta => split_fasta(reads, input, output),
        utils::Format::Fastq => split_fastq(reads, input, output),
        _ => (),
    }
}

fn generate_split_name(filename: String, split_suffix: &str) -> String {
    return filename.replacen(".", &format!("{}.", split_suffix), 1);
}

fn split_fasta<R: std::io::Read, W: std::io::Write>(
    reads: &chimera::BadReadMap,
    input: R,
    output: W,
) {
    let reader = bio::io::fasta::Reader::new(input);
    let mut writer = bio::io::fasta::Writer::new(output);

    for r in reader.records() {
        let record = r.expect("Trouble in fasta parsing process");
        if reads.contains_key(record.id()) {
            let (read_type, gap) = reads.get(record.id()).unwrap();

            if *read_type == chimera::BadReadType::NotCovered {
                continue;
            }

            if gap[0].begin != 0 {
                if !write_fasta_record(&mut writer, &record, 0, 0, gap[0].begin as usize) {
                    continue;
                }
            }

            for (i, objects) in gap.windows(2).enumerate() {
                if !write_fasta_record(
                    &mut writer,
                    &record,
                    (i + 1) as u64,
                    objects[0].end as usize,
                    objects[1].begin as usize,
                )
                {
                    continue;
                }
            }

            if gap.last().unwrap().end as usize != record.seq().len() {
                if !write_fasta_record(
                    &mut writer,
                    &record,
                    gap.len() as u64,
                    gap.last().unwrap().end as usize,
                    record.seq().len(),
                )
                {
                    continue;
                }
            }
        } else {
            writer.write_record(&record).unwrap();
        }
    }
}

fn split_fastq<R: std::io::Read, W: std::io::Write>(
    reads: &chimera::BadReadMap,
    input: R,
    output: W,
) {
    let reader = bio::io::fastq::Reader::new(input);
    let mut writer = bio::io::fastq::Writer::new(output);

    for r in reader.records() {
        let record = r.expect("Trouble in fasta parsing process");
        if reads.contains_key(record.id()) {
            let (read_type, gap) = reads.get(record.id()).unwrap();

            if *read_type == chimera::BadReadType::NotCovered {
                continue;
            }

            if gap[0].begin != 0 {
                if !write_fastq_record(&mut writer, &record, 0, 0, gap[0].begin as usize) {
                    continue;
                }
            }

            for (i, objects) in gap.windows(2).enumerate() {
                if !write_fastq_record(
                    &mut writer,
                    &record,
                    (i + 1) as u64,
                    objects[0].end as usize,
                    objects[1].begin as usize,
                )
                {
                    continue;
                }
            }

            if gap.last().unwrap().end as usize != record.seq().len() {
                if !write_fastq_record(
                    &mut writer,
                    &record,
                    gap.len() as u64,
                    gap.last().unwrap().end as usize,
                    record.seq().len(),
                )
                {
                    continue;
                }
            }
        } else {
            writer.write_record(&record).unwrap();
        }
    }
}

fn good_order(begin: usize, end: usize) -> (usize, usize) {
    return if begin < end {
        (begin, end)
    } else {
        (end, begin)
    };
}

fn in_read(begin: usize, end: usize, length: usize) -> bool {
    return begin > length || end > length;
}

fn write_fasta_record<W: std::io::Write>(
    writer: &mut bio::io::fasta::Writer<W>,
    record: &bio::io::fasta::Record,
    id: u64,
    begin: usize,
    end: usize,
) -> bool {
    let (begin, end) = good_order(begin, end);

    match in_read(begin, end, record.seq().len()) {
        e @ true => return e,
        false => (),
    };

    writer
        .write_record(&bio::io::fasta::Record::with_attrs(
            &format!("{}_{}", record.id(), id),
            record.desc(),
            &record.seq()[begin..end],
        ))
        .expect("Trouble durring fasta valid sequence writing");

    return true;
}

fn write_fastq_record<W: std::io::Write>(
    writer: &mut bio::io::fastq::Writer<W>,
    record: &bio::io::fastq::Record,
    id: u64,
    begin: usize,
    end: usize,
) -> bool {
    let (begin, end) = good_order(begin, end);

    match in_read(begin, end, record.seq().len()) {
        e @ true => return e,
        false => (),
    };

    writer
        .write_record(&bio::io::fastq::Record::with_attrs(
            &format!("{}_{}", record.id(), id),
            record.desc(),
            &record.seq()[begin..end],
            &record.qual()[begin..end],
        ))
        .expect("Trouble durring fasta valid sequence writing");

    return true;
}

#[cfg(test)]
mod test {

    use super::*;

    use std::collections::HashMap;

    #[test]
    fn split_name() {
        assert_eq!(
            generate_split_name("test.paf".to_string(), "_test"),
            "test_test.paf"
        );
        assert_eq!(
            generate_split_name("test.paf.gz".to_string(), "_test"),
            "test_test.paf.gz"
        );
        assert_eq!(
            generate_split_name("test.fasta".to_string(), "_test"),
            "test_test.fasta"
        );
    }

    lazy_static! {
        static ref REMOVE_READS: Box<chimera::BadReadMap> = {
            let mut m = Box::new(HashMap::new());
            m.insert(
                "1".to_string(),
                (
                    chimera::BadReadType::Chimeric,
                    vec![
                        chimera::Interval { begin: 4, end: 9 },
                        chimera::Interval { begin: 13, end: 18 },
                    ],
                ),
            );
            m
        };
    }

    const FASTA_FILE: &'static [u8] = b">1
ACTGGGGGGACTGGGGGGACTG
>2
ACTG
>3
ACTG
";

    const FASTA_FILE_SPLITED: &'static [u8] = b">1_0
ACTG
>1_1
ACTG
>1_2
ACTG
>2
ACTG
>3
ACTG
";

    #[test]
    fn fasta() {
        let mut writer: Vec<u8> = Vec::new();

        split_fasta(&REMOVE_READS, FASTA_FILE, &mut writer);

        assert_eq!(writer, FASTA_FILE_SPLITED);
    }

    const FASTQ_FILE: &'static [u8] = b"@1
ACTGGGGGGACTGGGGGGACTG
+
!!!!.....!!!!.....!!!!
@2
ACTG
+
!!!!
@3
ACTG
+
!!!!
";

    const FASTQ_FILE_SPLIT: &'static [u8] = b"@1_0
ACTG
+
!!!!
@1_1
ACTG
+
!!!!
@1_2
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

    #[test]
    fn fastq() {
        let mut writer: Vec<u8> = Vec::new();

        split_fastq(&REMOVE_READS, FASTQ_FILE, &mut writer);

        assert_eq!(writer, FASTQ_FILE_SPLIT);
    }

    const SHORT_FASTA_FILE: &'static [u8] = b">1
ACTGGGGGGACTG
>2
ACTG
>3
ACTG
";

    const SHORT_FASTA_FILE_SPLIT: &'static [u8] = b">1_0
ACTG
>1_1
ACTG
>2
ACTG
>3
ACTG
";
    #[test]
    fn fasta_short() {
        let mut writer: Vec<u8> = Vec::new();

        split_fasta(&REMOVE_READS, SHORT_FASTA_FILE, &mut writer);

        assert_eq!(writer, SHORT_FASTA_FILE_SPLIT);
    }

}
