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

extern crate clap;

use std::io;
use std::fs::File;

use clap::{Arg, App};

fn main() {
    let matches = App::new("yacrd")
        .version("0.3 Mew")
        .author("Pierre Marijon <pierre.marijon@inria.fr>")
        .about("Yet Another Chimeric Read Detector")
        .usage("yacrd [-i|--input] <input> [-o|--output] <output> [-f|--filter] <file1, file2, …> 
\tyacrd -i map_file.paf -o map_file.yacrd
\tyacrd -i map_file.mhap -o map_file.yacrd
\tyacrd -i map_file.xyz -F paf -o map_file.yacrd
\tyacrd -i map_file.paf -f sequence.fasta -o map_file.yacrd
\tzcat map_file.paf.gz | yacrd -i - -o map_file.yacrd
\tminimap2 sequence.fasta sequence.fasta | yacrd -o map_file.yacrd --fileterd-suffix _test -f sequence.fastq sequence2.fasta other.fastq
\tOr any combination of this.")
        .arg(Arg::with_name("input")
             .short("i")
             .long("input")
             .display_order(1)
             .takes_value(true)
             .default_value("-")
             .help("Mapping input file in PAF or MHAP format (with .paf or .mhap extension), use - for read standard input")
             )
        .arg(Arg::with_name("output")
             .short("o")
             .long("output")
             .display_order(2)
             .takes_value(true)
             .default_value("-")
             .help("Path where yacrd report are write")
             )
        .arg(Arg::with_name("filter")
             .short("f")
             .long("filter")
             .multiple(true)
             .display_order(3)
             .takes_value(true)
             .help("File containing reads that will be filtered (fasta|fastq|mhap|paf), new file are create like {original_path}_fileterd.{original_extension}")
             )
        .arg(Arg::with_name("format")
             .short("F")
             .long("format")
             .display_order(4)
             .takes_value(true)
             .possible_values(&["paf", "mhap"])
             .help("Force the format used")
             )
        .arg(Arg::with_name("chimeric-threshold")
             .short("c")
             .display_order(5)
             .takes_value(true)
             .default_value("0")
             .long("chimeric-threshold")
             .help("Overlap depth threshold below which a gap should be created")
             )
        .arg(Arg::with_name("not-covered-threshold")
             .short("n")
             .display_order(6)
             .takes_value(true)
             .default_value("0.80")
             .long("not-covered-threshold")
             .help("Coverage depth threshold above which a read are mark as not covered")
             )
        .arg(Arg::with_name("fileterd-suffix")
             .display_order(7)
             .takes_value(true)
             .long("fileterd-suffix")
             .default_value("_fileterd")
             .help("Change the suffix of file generate by filter option")
             )
        .get_matches();
}
