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
use anyhow::{anyhow, bail, Context, Result};
use bio::io::{fasta, fastq};

/* local use */
use crate::editor;
use crate::error;
use crate::stack;
use crate::util;

pub fn extract(
    input_path: &str,
    output_path: &str,
    badregions: &mut dyn stack::BadPart,
    not_covered: f64,
) -> Result<()> {
    let (input, compression) = util::read_file(input_path)?;
    let output = util::write_file(output_path, compression)?;

    match util::get_file_type(input_path) {
        Some(util::FileType::Fasta) => fasta(input, output, badregions, not_covered)
            .with_context(|| anyhow!("Filename: {}", input_path.to_string()))?,
        Some(util::FileType::Fastq) => fastq(input, output, badregions, not_covered)
            .with_context(|| anyhow!("Filename: {}", input_path.to_string()))?,
        Some(util::FileType::Paf) => paf(input, output, badregions, not_covered)
            .with_context(|| anyhow!("Filename: {}", input_path.to_string()))?,
        Some(util::FileType::M4) => m4(input, output, badregions, not_covered)
            .with_context(|| anyhow!("Filename: {}", input_path.to_string()))?,
        Some(util::FileType::Yacrd) => bail!(error::Error::CantRunOperationOnFile {
            operation: "scrubbing".to_string(),
            filetype: util::FileType::Yacrd,
            filename: input_path.to_string()
        }),
        None | Some(util::FileType::YacrdOverlap) => {
            bail!(error::Error::UnableToDetectFileFormat {
                filename: input_path.to_string()
            })
        }
    }

    Ok(())
}

fn fasta<R, W>(
    input: R,
    output: W,
    badregions: &mut dyn stack::BadPart,
    not_covered: f64,
) -> Result<()>
where
    R: std::io::Read,
    W: std::io::Write,
{
    let reader = fasta::Reader::new(input);
    let mut writer = fasta::Writer::new(output);

    for result in reader.records() {
        let record = result.with_context(|| error::Error::ReadingErrorNoFilename {
            format: util::FileType::Fasta,
        })?;

        let (badregion, length) = badregions.get_bad_part(&record.id().to_string())?;

        let rtype = editor::type_of_read(*length, badregion, not_covered);

        if rtype != editor::ReadType::NotBad {
            writer
                .write_record(&record)
                .with_context(|| error::Error::WritingErrorNoFilename {
                    format: util::FileType::Fasta,
                })?;
        }
    }

    Ok(())
}

fn fastq<R, W>(
    input: R,
    output: W,
    badregions: &mut dyn stack::BadPart,
    not_covered: f64,
) -> Result<()>
where
    R: std::io::Read,
    W: std::io::Write,
{
    let reader = fastq::Reader::new(input);
    let mut writer = fastq::Writer::new(output);

    for result in reader.records() {
        let record = result.with_context(|| error::Error::ReadingErrorNoFilename {
            format: util::FileType::Fastq,
        })?;

        let (badregion, length) = badregions.get_bad_part(&record.id().to_string())?;

        let rtype = editor::type_of_read(*length, badregion, not_covered);

        if rtype != editor::ReadType::NotBad {
            writer
                .write_record(&record)
                .with_context(|| error::Error::WritingErrorNoFilename {
                    format: util::FileType::Fastq,
                })?;
        }
    }

    Ok(())
}

fn paf<R, W>(
    input: R,
    output: W,
    badregions: &mut dyn stack::BadPart,
    not_covered: f64,
) -> Result<()>
where
    R: std::io::Read,
    W: std::io::Write,
{
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(input);
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_writer(output);

    for result in reader.records() {
        let record = result.with_context(|| error::Error::ReadingErrorNoFilename {
            format: util::FileType::Paf,
        })?;

        let id_a = record[0].to_string();
        let id_b = record[5].to_string();

        let (badregion, length) = badregions.get_bad_part(&id_a)?;
        let rtype_a = editor::type_of_read(*length, badregion, not_covered);

        let (badregion, length) = badregions.get_bad_part(&id_b)?;
        let rtype_b = editor::type_of_read(*length, badregion, not_covered);

        if rtype_a != editor::ReadType::NotBad || rtype_b != editor::ReadType::NotBad {
            writer
                .write_record(&record)
                .with_context(|| error::Error::WritingErrorNoFilename {
                    format: util::FileType::Paf,
                })?;
        }
    }

    Ok(())
}

fn m4<R, W>(
    input: R,
    output: W,
    badregions: &mut dyn stack::BadPart,
    not_covered: f64,
) -> Result<()>
where
    R: std::io::Read,
    W: std::io::Write,
{
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b' ')
        .has_headers(false)
        .from_reader(input);
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b' ')
        .has_headers(false)
        .from_writer(output);

    for result in reader.records() {
        let record = result.with_context(|| error::Error::ReadingErrorNoFilename {
            format: util::FileType::M4,
        })?;

        let id_a = record[0].to_string();
        let id_b = record[1].to_string();

        let (badregion, length) = badregions.get_bad_part(&id_a)?;
        let rtype_a = editor::type_of_read(*length, badregion, not_covered);

        let (badregion, length) = badregions.get_bad_part(&id_b)?;
        let rtype_b = editor::type_of_read(*length, badregion, not_covered);

        if rtype_a != editor::ReadType::NotBad || rtype_b != editor::ReadType::NotBad {
            writer
                .write_record(&record)
                .with_context(|| error::Error::WritingErrorNoFilename {
                    format: util::FileType::M4,
                })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::reads2ovl;
    use crate::reads2ovl::Reads2Ovl;

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
    fn fasta_file() -> () {
        let mut ovlst = reads2ovl::FullMemory::new();

        ovlst.add_length("1".to_string(), 1000);
        ovlst.add_overlap("1".to_string(), (10, 490)).unwrap();
        ovlst.add_overlap("1".to_string(), (510, 1000)).unwrap();

        let mut stack = stack::FromOverlap::new(Box::new(ovlst), 0);

        let mut output: Vec<u8> = Vec::new();
        fasta(FASTA_FILE, &mut output, &mut stack, 0.8).unwrap();

        assert_eq!(FASTA_FILE_EXTRACTED, &output[..]);
    }

    const FASTQ_FILE: &'static [u8] = b"@1
ACTG
+
????
@2
ACTG
+
????
@3
ACTG
+
????
";

    const FASTQ_FILE_EXTRACTED: &'static [u8] = b"@1
ACTG
+
????
";

    #[test]
    fn fastq_file() {
        let mut ovlst = reads2ovl::FullMemory::new();

        ovlst.add_length("1".to_string(), 1000);
        ovlst.add_overlap("1".to_string(), (10, 490)).unwrap();
        ovlst.add_overlap("1".to_string(), (510, 1000)).unwrap();

        let mut stack = stack::FromOverlap::new(Box::new(ovlst), 0);

        let mut output: Vec<u8> = Vec::new();
        fastq(FASTQ_FILE, &mut output, &mut stack, 0.8).unwrap();

        assert_eq!(FASTQ_FILE_EXTRACTED, &output[..]);
    }

    const PAF_FILE: &'static [u8] = b"1\t12000\t20\t4500\t-\t2\t10000\t5500\t10000\t4500\t4500\t255
1\t12000\t5500\t10000\t-\t3\t10000\t0\t4500\t4500\t4500\t255
";

    const PAF_FILE_EXTRACTED: &'static [u8] =
        b"1\t12000\t20\t4500\t-\t2\t10000\t5500\t10000\t4500\t4500\t255
1\t12000\t5500\t10000\t-\t3\t10000\t0\t4500\t4500\t4500\t255
";

    #[test]
    fn paf_file() {
        let mut ovlst = reads2ovl::FullMemory::new();

        ovlst.add_length("1".to_string(), 1000);
        ovlst.add_overlap("1".to_string(), (10, 490)).unwrap();
        ovlst.add_overlap("1".to_string(), (510, 1000)).unwrap();

        let mut stack = stack::FromOverlap::new(Box::new(ovlst), 0);

        let mut output: Vec<u8> = Vec::new();
        paf(PAF_FILE, &mut output, &mut stack, 0.8).unwrap();

        assert_eq!(PAF_FILE_EXTRACTED, &output[..]);
    }

    const M4_FILE: &'static [u8] = b"1 2 0.1 2 0 100 450 1000 0 550 900 1000
1 3 0.1 2 0 550 900 1000 0 100 450 1000
";

    const M4_FILE_EXTRACTED: &'static [u8] = b"1 2 0.1 2 0 100 450 1000 0 550 900 1000
1 3 0.1 2 0 550 900 1000 0 100 450 1000
";

    #[test]
    fn m4_file() {
        let mut ovlst = reads2ovl::FullMemory::new();

        ovlst.add_length("1".to_string(), 1000);
        ovlst.add_overlap("1".to_string(), (10, 490)).unwrap();
        ovlst.add_overlap("1".to_string(), (510, 1000)).unwrap();

        let mut stack = stack::FromOverlap::new(Box::new(ovlst), 0);

        let mut output: Vec<u8> = Vec::new();
        m4(M4_FILE, &mut output, &mut stack, 0.8).unwrap();

        assert_eq!(M4_FILE_EXTRACTED, &output[..]);
    }
}
