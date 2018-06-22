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

extern crate csv;
extern crate xz2;
extern crate clap;
extern crate bzip2;
extern crate flate2;
#[macro_use]
extern crate serde_derive;

use clap::{Arg, App};

/* local use */
mod file;
mod utils;
mod filter;
mod chimera;

/* crates use */

/* standard use */
use std::io;
use std::fs::File;
use std::collections::{HashSet};

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
             .help("Mapping input file in PAF or MHAP format (with .paf or .mhap extension), use - for read standard input (no compression allowed)")
             )
        .arg(Arg::with_name("output")
             .short("o")
             .long("output")
             .display_order(2)
             .takes_value(true)
             .default_value("-")
             .help("Path where yacrd report are write, use - for write in standard output same compression as input or use --compression-out")
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
             .help("Force the format used")
             .possible_values(&["paf", "mhap"])
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
        .arg(Arg::with_name("filtered-suffix")
             .display_order(7)
             .takes_value(true)
             .long("filtered-suffix")
             .default_value("_filtered")
             .help("Change the suffix of file generate by filter option")
             )
        .arg(Arg::with_name("compression-out")
             .short("C")
             .display_order(8)
             .takes_value(true)
             .long("compression-out")
             .possible_values(&["gzip", "bzip2", "lzma", "no"])
             .help("Overlap depth threshold below which a gap should be created")
             )
        .get_matches();

    let (input, compression) = file::get_input(matches.value_of("input").unwrap());

    let out_compression = file::choose_compression(compression, matches.is_present("compression-out"), matches.value_of("compression-out").unwrap_or("no"));
    let output: Box<io::Write> = file::get_output(matches.value_of("output").unwrap(), out_compression);

    let filters: Vec<_> = match matches.is_present("filter") {
        true => matches.values_of("filter").unwrap().collect(),
        false => Vec::new()
    };

    let format = utils::get_format(&matches).expect("Format of input can be determinate check file extension or value of --format option");

    let chim_thres = matches.value_of("chimeric-threshold").unwrap().parse::<u64>().unwrap();
    let ncov_thres = matches.value_of("not-covered-threshold").unwrap().parse::<f64>().unwrap();
    let filterd_suffix = matches.value_of("filtered-suffix").unwrap();

    let remove_reads: Box<HashSet<String>> = chimera::find(input, output, format, chim_thres, ncov_thres);
}
