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

#[derive(StructOpt, Debug)]
#[structopt(
    version = "0.6b Mew",
    author = "Pierre Marijon <pmarijon@mpi-inf.mpg.de>",
    name = "yacrd",
    about = "
Yacrd use overlap between reads, to detect 'good' and 'bad' region,
region with coverage over threshold is 'good' other are 'bad'.
If read have a 'bad' region in middle this reads is mark as 'Chimeric'.
If ratio of 'bad' region length on total read length is larger than threshold this reads is mark as 'Not_covered'.

Yacrd can make some other actions:
- filter: for sequence or overlap file, record with reads marked as Chimeric or Not_covered isn't write in output
- extract: for sequence or overlap file, record contain reads marked as Chimeric or Not_covered is write in output
- split: for sequence file bad region in middle of reads are removed, Not_covered read is removed
- scrubb: for sequence file all bad region are removed, Not_covered read is removed
"
)]
pub struct Command {
    #[structopt(
        short = "i",
        long = "input",
        required = true,
        help = "path to input file overlap (.paf|.m4) or yacrd report (.yacrd) format audetected input-format overide detection"
    )]
    pub input: String,

    #[structopt(
        short = "o",
        long = "output",
        required = true,
        help = "path output file, yacrd format by default output-format can overide this value"
    )]
    pub output: String,

    #[structopt(long = "input-format", possible_values = &["paf", "m4", "yacrd", "json"], help = "set the input-format")]
    pub input_format: Option<String>,

    #[structopt(long = "output-format", possible_values = &["yacrd", "json"], default_value = "yacrd", help = "set the output-format")]
    pub output_format: String,

    #[structopt(
        short = "c",
        long = "coverage",
        default_value = "0",
        help = "if coverage reach this value region is mark as bad"
    )]
    pub coverage: u64,

    #[structopt(
        short = "n",
        long = "not-coverage",
        default_value = "0.8",
        help = "if ratio of bad region length on total lengh is lower that this value, all read is mark as bad"
    )]
    pub not_coverage: f64,

    #[structopt(
        short = "d",
        long = "ondisk",
        help = "if it set yacrd create tempory file, with value of this parameter as prefix, to reduce memory usage but increase the runtime"
    )]
    pub ondisk: Option<String>,

    #[structopt(
        long = "ondisk-buffer-size",
	default_value = "64000000",
        help = "with the default value yacrd in ondisk mode use around 800 MBytes, you can increase to reduce runtime but increase memory usage"
    )]
    pub ondisk_buffer_size: String,
        
    #[structopt(subcommand)]
    pub subcmd: Option<SubCommand>,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    #[structopt(about = "All bad region of read is removed")]
    Scrubb(Scrubb),
    #[structopt(about = "Record mark as chimeric or Not_covered is filter")]
    Filter(Filter),
    #[structopt(about = "Record mark as chimeric or Not_covered is extract")]
    Extract(Extract),
    #[structopt(about = "Record mark as chimeric or Not_covered is split")]
    Split(Split),
}

#[derive(StructOpt, Debug)]
pub struct Scrubb {
    #[structopt(
        short = "i",
        long = "input",
        required = true,
        help = "path to sequence input (fasta|fastq) compression is autodetect (none|gzip|bzip2|lzma)"
    )]
    pub input: String,

    #[structopt(
        short = "o",
        long = "output",
        required = true,
        help = "path to output file, format and compression of input is preserved"
    )]
    pub output: String,
}

#[derive(StructOpt, Debug)]
pub struct Filter {
    #[structopt(
        short = "i",
        long = "input",
        required = true,
        help = "path to sequence input (fasta|fastq) compression is autodetect (none|gzip|bzip2|lzma)"
    )]
    pub input: String,

    #[structopt(
        short = "o",
        long = "output",
        required = true,
        help = "path to output file, format and compression of input is preserved"
    )]
    pub output: String,
}

#[derive(StructOpt, Debug)]
pub struct Extract {
    #[structopt(
        short = "i",
        long = "input",
        required = true,
        help = "path to sequence input (fasta|fastq) compression is autodetect (none|gzip|bzip2|lzma)"
    )]
    pub input: String,

    #[structopt(
        short = "o",
        long = "output",
        required = true,
        help = "path to output file, format and compression of input is preserved"
    )]
    pub output: String,
}

#[derive(StructOpt, Debug)]
pub struct Split {
    #[structopt(
        short = "i",
        long = "input",
        required = true,
        help = "path to sequence input (fasta|fastq) compression is autodetect (none|gzip|bzip2|lzma)"
    )]
    pub input: String,

    #[structopt(
        short = "o",
        long = "output",
        required = true,
        help = "path to output file, format and compression of input is preserved"
    )]
    pub output: String,
}
