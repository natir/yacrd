/*
Copyright (c) 2018 Pierre Marijon <pmarijon@mpi-inf.mpg.de>

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

/// Yacrd use overlap between reads, to detect 'good' and 'bad' region,
/// a region with coverage over the threshold is 'good' others are 'bad'.
/// If read has a 'bad' region in middle this reads is mark as 'Chimeric'.
/// If the ratio of 'bad' region length on total read length is larger than threshold this reads is marked as 'Not_covered'.

/// Yacrd can make some other actions:
/// - filter: for sequence or overlap file, record with reads marked as Chimeric or NotCovered isn't written in the output
/// - extract: for sequence or overlap file, record contains reads marked as Chimeric or NotCovered is written in the output
/// - split: for sequence file bad region in the middle of reads are removed, NotCovered read is removed
/// - scrubb: for sequence file all bad region are removed, NotCovered read is removed
#[derive(clap::Parser, Debug)]
#[clap(
    version = "1.0.0 Magby",
    author = "Pierre Marijon <pierre@marijon.fr>",
    name = "yacrd"
)]
pub struct Command {
    /// path to input file overlap (.paf|.m4|.mhap) or yacrd report (.yacrd), format is autodetected and compression input is allowed (gz|bzip2|lzma)
    #[clap(short = 'i', long = "input")]
    pub input: String,

    /// path output file
    #[clap(short = 'o', long = "output")]
    pub output: String,

    /// number of thread use by yacrd, 0 mean all threads available, default 1
    #[clap(short = 't', long = "thread")]
    pub threads: Option<usize>,

    /// if coverage reach this value region is marked as bad
    #[clap(short = 'c', long = "coverage", default_value = "0")]
    pub coverage: u64,

    /// if the ratio of bad region length on total length is lower than this value, read is marked as NotCovered
    #[clap(short = 'n', long = "not-coverage", default_value = "0.8")]
    pub not_coverage: f64,

    /// Control the size of the buffer used to read paf file
    #[clap(long = "read-buffer-size", default_value = "8192")]
    pub buffer_size: usize,

    /// yacrd switches to 'ondisk' mode which will reduce memory usage but increase computation time. The value passed as a parameter is used as a prefix for the temporary files created by yacrd. Be careful if the prefix contains path separators (`/` for unix or `\\` for windows) this folder will be deleted
    #[clap(short = 'd', long = "ondisk")]
    pub ondisk: Option<String>,

    /// with the default value yacrd in 'ondisk' mode use around 1 GBytes, you can increase to reduce runtime but increase memory usage
    #[clap(long = "ondisk-buffer-size", default_value = "64000000")]
    pub ondisk_buffer_size: String,

    #[clap(subcommand)]
    pub subcmd: Option<SubCommand>,
}

#[derive(clap::Parser, Debug)]
pub enum SubCommand {
    /// All bad region of read is removed
    #[clap()]
    Scrubb(Scrubb),

    /// Record mark as chimeric or NotCovered is filter
    #[clap()]
    Filter(Filter),

    /// Record mark as chimeric or NotCovered is extract
    #[clap()]
    Extract(Extract),

    /// Record mark as chimeric or NotCovered is split
    #[clap()]
    Split(Split),
}

#[derive(clap::Parser, Debug)]
pub struct Scrubb {
    /// path to sequence input (fasta|fastq), compression is autodetected (none|gzip|bzip2|lzma)
    #[clap(short = 'i', long = "input", required = true)]
    pub input: String,

    /// path to output file, format and compression of input is preserved
    #[clap(short = 'o', long = "output", required = true)]
    pub output: String,
}

#[derive(clap::Parser, Debug)]
pub struct Filter {
    /// path to sequence input (fasta|fastq), compression is autodetected (none|gzip|bzip2|lzma)
    #[clap(short = 'i', long = "input", required = true)]
    pub input: String,

    /// path to output file, format and compression of input is preserved
    #[clap(short = 'o', long = "output", required = true)]
    pub output: String,
}

#[derive(clap::Parser, Debug)]
pub struct Extract {
    /// path to sequence input (fasta|fastq), compression is autodetected (none|gzip|bzip2|lzma)
    #[clap(short = 'i', long = "input", required = true)]
    pub input: String,

    /// path to output file, format and compression of input is preserved
    #[clap(short = 'o', long = "output", required = true)]
    pub output: String,
}

#[derive(clap::Parser, Debug)]
pub struct Split {
    /// path to sequence input (fasta|fastq), compression is autodetected (none|gzip|bzip2|lzma)
    #[clap(short = 'i', long = "input", required = true)]
    pub input: String,

    /// path to output file, format and compression of input is preserved
    #[clap(short = 'o', long = "output", required = true)]
    pub output: String,
}
