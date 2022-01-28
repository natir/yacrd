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
use log::error;

/* local use */
use crate::editor;
use crate::error;
use crate::stack;
use crate::util;

pub fn split(
    input_path: &str,
    output_path: &str,
    badregions: &mut dyn stack::BadPart,
    not_covered: f64,
    buffer_size: usize,
) -> Result<()> {
    let (input, compression) = util::read_file(input_path, buffer_size)?;
    let output = util::write_file(output_path, compression, buffer_size)?;

    match util::get_file_type(input_path) {
        Some(util::FileType::Fasta) => fasta(input, output, badregions, not_covered)
            .with_context(|| anyhow!("Filename: {}", input_path.to_string()))?,
        Some(util::FileType::Fastq) => fastq(input, output, badregions, not_covered)
            .with_context(|| anyhow!("Filename: {}", input_path.to_string()))?,
        Some(util::FileType::Paf) => bail!(error::Error::CantRunOperationOnFile {
            operation: "split".to_string(),
            filetype: util::FileType::Paf,
            filename: input_path.to_string()
        }),
        Some(util::FileType::M4) => bail!(error::Error::CantRunOperationOnFile {
            operation: "split".to_string(),
            filetype: util::FileType::M4,
            filename: input_path.to_string()
        }),
        Some(util::FileType::Yacrd) => bail!(error::Error::CantRunOperationOnFile {
            operation: "split".to_string(),
            filetype: util::FileType::Yacrd,
            filename: input_path.to_string()
        }),
        None | Some(util::FileType::YacrdOverlap) => {
            bail!(error::Error::UnableToDetectFileFormat {
                filename: input_path.to_string()
            })
        }
    };

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
    let mut reader = noodles::fasta::Reader::new(std::io::BufReader::new(input));
    let mut writer = noodles::fasta::Writer::new(std::io::BufWriter::new(output));

    for result in reader.records() {
        let record = result.with_context(|| error::Error::ReadingErrorNoFilename {
            format: util::FileType::Fasta,
        })?;

        let (badregion, length) = badregions.get_bad_part(record.name())?;

        let rtype = editor::type_of_read(*length, badregion, not_covered);

        if rtype == editor::ReadType::NotCovered {
            continue;
        } else if rtype == editor::ReadType::NotBad {
            writer
                .write_record(&record)
                .with_context(|| error::Error::WritingErrorNoFilename {
                    format: util::FileType::Fasta,
                })?;
        } else {
            let mut poss = vec![0];
            for interval in badregion {
                if interval.0 == 0 || interval.1 == *length as u32 {
                    continue;
                }

                poss.push(interval.0);
                poss.push(interval.1);
            }
            poss.push(*length as u32);

            for pos in poss.chunks(2) {
                if pos[0] as usize > record.sequence().len()
                    || pos[1] as usize > record.sequence().len()
                {
                    error!("For read {} split position is larger than read, it's strange check your data. For this read, this split position and next are ignore.", record.name());
                    break;
                }

                writer
                    .write_record(&noodles::fasta::Record::new(
                        noodles::fasta::record::Definition::new(
                            &format!("{}_{}_{}", record.name(), pos[0], pos[1]),
                            None,
                        ),
                        noodles::fasta::record::Sequence::from(
                            record.sequence().as_ref()[(pos[0] as usize)..(pos[1] as usize)]
                                .to_vec(),
                        ),
                    ))
                    .with_context(|| error::Error::WritingErrorNoFilename {
                        format: util::FileType::Fasta,
                    })?;
            }
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
    let mut reader = noodles::fastq::Reader::new(std::io::BufReader::new(input));
    let mut writer = noodles::fastq::Writer::new(std::io::BufWriter::new(output));

    for result in reader.records() {
        let record = result.with_context(|| error::Error::ReadingErrorNoFilename {
            format: util::FileType::Fastq,
        })?;

        let (badregion, length) = badregions.get_bad_part(
            std::str::from_utf8(record.name())?
                .split_ascii_whitespace()
                .next()
                .unwrap(),
        )?;

        let rtype = editor::type_of_read(*length, badregion, not_covered);

        if rtype == editor::ReadType::NotCovered {
            continue;
        } else if rtype == editor::ReadType::NotBad {
            writer
                .write_record(&record)
                .with_context(|| error::Error::WritingErrorNoFilename {
                    format: util::FileType::Fastq,
                })?;
        } else {
            let mut sequence_description = std::str::from_utf8(record.name())?.splitn(2, ' ');
            let name = sequence_description.next().unwrap();
            let description = sequence_description.next();

            let mut poss = vec![0];
            for interval in badregion {
                if interval.0 == 0 || interval.1 == *length as u32 {
                    continue;
                }

                poss.push(interval.0);
                poss.push(interval.1);
            }
            poss.push(*length as u32);

            for pos in poss.chunks(2) {
                if pos[0] as usize > record.sequence().len()
                    || pos[1] as usize > record.sequence().len()
                {
                    error!("For read {} split position is larger than read, it's strange check your data. For this read, this split position and next are ignore.", std::str::from_utf8(record.name())?);
                    break;
                }

                writer
                    .write_record(&noodles::fastq::Record::new(
                        match description {
                            Some(desc) => format!("{}_{}_{} {}", name, pos[0], pos[1], desc),
                            None => format!("{}_{}_{}", name, pos[0], pos[1]),
                        }
                        .as_bytes(),
                        record.sequence()[(pos[0] as usize)..(pos[1] as usize)].to_vec(),
                        record.quality_scores()[(pos[0] as usize)..(pos[1] as usize)].to_vec(),
                    ))
                    .with_context(|| error::Error::WritingErrorNoFilename {
                        format: util::FileType::Fasta,
                    })?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::stack::BadPart;

    use crate::reads2ovl;
    use crate::reads2ovl::Reads2Ovl;

    const FASTA_FILE: &'static [u8] = b">1
ACTGGGGGGACTGGGGGGACTG
>2
ACTG
>3
ACTG
";

    const FASTA_FILE_SPLITED: &'static [u8] = b">1_0_13
ACTGGGGGGACTG
>1_18_22
ACTG
>2
ACTG
>3
ACTG
";

    #[test]
    fn fasta_file() -> () {
        let mut ovlst = reads2ovl::FullMemory::new(8192);

        ovlst.add_length("1".to_string(), 22);
        ovlst.add_overlap("1".to_string(), (9, 13)).unwrap();
        ovlst.add_overlap("1".to_string(), (18, 22)).unwrap();

        let mut stack = stack::FromOverlap::new(Box::new(ovlst), 0);

        stack.compute_all_bad_part();

        let mut output: Vec<u8> = Vec::new();
        fasta(FASTA_FILE, &mut output, &mut stack, 0.8).unwrap();

        assert_eq!(FASTA_FILE_SPLITED, &output[..]);
    }

    const FASTQ_FILE: &'static [u8] = b"@1
ACTGGGGGGACTGGGGGGACTG
+
??????????????????????
@2
ACTG
+
????
@3
ACTG
+
????
";

    const FASTQ_FILE_FILTRED: &'static [u8] = b"@1_0_13
ACTGGGGGGACTG
+
?????????????
@1_18_22
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

    #[test]
    fn fastq_file() {
        let mut ovlst = reads2ovl::FullMemory::new(8192);

        ovlst.add_length("1".to_string(), 22);
        ovlst.add_overlap("1".to_string(), (9, 13)).unwrap();
        ovlst.add_overlap("1".to_string(), (18, 22)).unwrap();

        let mut stack = stack::FromOverlap::new(Box::new(ovlst), 0);

        stack.compute_all_bad_part();

        let mut output: Vec<u8> = Vec::new();
        fastq(FASTQ_FILE, &mut output, &mut stack, 0.8).unwrap();

        assert_eq!(FASTQ_FILE_FILTRED, &output[..]);
    }
}
