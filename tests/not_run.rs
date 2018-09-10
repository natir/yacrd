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

use std::process::Command;

static HELP_MESSAGE: &'static str = "yacrd 0.4 Mew
Pierre Marijon <pierre.marijon@inria.fr>
Yet Another Chimeric Read Detector

USAGE:
    yacrd [-i|--input] <input1, input2, …> [-o|--output] <output> [-f|--filter] <file1, file2, …> 
	yacrd -i map_file.paf -o map_file.yacrd
	yacrd -i map_file.mhap -o map_file.yacrd
	yacrd -i map_file.xyz -F paf -o map_file.yacrd
	yacrd -i map_file.paf -f sequence.fasta -o map_file.yacrd
	zcat map_file.paf.gz | yacrd -i - -o map_file.yacrd
	minimap2 sequence.fasta sequence.fasta | yacrd -o map_file.yacrd --fileterd-suffix _test -f sequence.fastq sequence2.fasta other.fastq
	Or any combination of this.

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input <input>...
            Mapping input file in PAF or MHAP format (with .paf or .mhap extension), use - for read standard input (no
            compression allowed, paf format by default) [default: -]
    -o, --output <output>
            Path where yacrd report are writen, use - for write in standard output same compression as input or use
            --compression-out [default: -]
    -f, --filter <filter>...
            Create a new file {original_path}_fileterd.{original_extension} with only not chimeric records, format
            support fasta|fastq|mhap|paf
    -e, --extract <extract>...
            Create a new file {original_path}_extracted.{original_extension} with only chimeric records, format support
            fasta|fastq|mhap|paf
    -s, --split <split>...
            Create a new file {original_path}_splited.{original_extension} where chimeric records are split, format
            support fasta|fastq
    -F, --format <format>                                  Force the format used [possible values: paf, mhap]
    -c, --chimeric-threshold <chimeric-threshold>
            Overlap depth threshold below which a gap should be created [default: 0]

    -n, --not-covered-threshold <not-covered-threshold>
            Coverage depth threshold above which a read are marked as not covered [default: 0.80]

        --filtered-suffix <filtered-suffix>
            Change the suffix of file generate by filter option [default: _filtered]

        --extracted-suffix <extracted-suffix>
            Change the suffix of file generate by extract option [default: _extracted]

        --splited-suffix <splited-suffix>
            Change the suffix of file generate by split option [default: _splited]

    -C, --compression-out <compression-out>
            Overlap depth threshold below which a gap should be created [possible values: gzip, bzip2, lzma, no]

";

#[cfg(test)]
mod not_run {

    use super::*;

    #[test]
    fn version() {
        let output = Command::new("./target/debug/yacrd")
            .arg("-V")
            .output()
            .expect("Could not run yacrd");

        assert_eq!(output.stdout, b"yacrd 0.4 Mew\n");
        println!("{:?}", output);
    }

    #[test]
    fn help() {
        let output = Command::new("./target/debug/yacrd")
            .arg("-h")
            .output()
            .expect("Could not run yacrd");

        assert_eq!(String::from_utf8_lossy(&output.stdout), HELP_MESSAGE);
    }

    #[test]
    fn no_argument() {
        let output = Command::new("./target/debug/yacrd").output().expect(
            "Could not run yacrd",
        );

        assert_eq!(&output.stdout, b"");
    }
}
