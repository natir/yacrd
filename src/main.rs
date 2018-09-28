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

extern crate bio;
extern crate bzip2;
extern crate clap;
extern crate csv;
extern crate flate2;
extern crate xz2;

#[macro_use]
extern crate enum_primitive;

#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

/* local mod */
mod chimera;
mod extract;
mod file;
mod filter;
mod io;
mod split;
mod utils;

mod postdetection;

/* crates use */
use clap::{App, Arg};

/* standard use */
use std::collections::HashMap;

fn main() {
    let matches = App::new("yacrd")
        .version("0.4 Mew")
        .author("Pierre Marijon <pierre.marijon@inria.fr>")
        .about("Yet Another Chimeric Read Detector")
        .usage("yacrd [-i|--input] <input1, input2, …> [-o|--output] <output> [-f|--filter] <file1, file2, …> 
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
             .multiple(true)
             .display_order(10)
             .takes_value(true)
             .default_value("-")
             .help("Mapping input file in PAF or MHAP format (with .paf or .mhap extension), use - for read standard input (no compression allowed, paf format by default)")
             )
        .arg(Arg::with_name("output")
             .short("o")
             .long("output")
             .display_order(20)
             .takes_value(true)
             .default_value("-")
             .help("Path where yacrd report are writen, use - for write in standard output same compression as input or use --compression-out")
             )
        .arg(Arg::with_name("filter")
             .short("f")
             .long("filter")
             .multiple(true)
             .display_order(30)
             .takes_value(true)
             .help("Create a new file {original_path}_fileterd.{original_extension} with only not chimeric records, format support fasta|fastq|mhap|paf")
             )
        .arg(Arg::with_name("extract")
             .short("e")
             .long("extract")
             .multiple(true)
             .display_order(40)
             .takes_value(true)
             .help("Create a new file {original_path}_extracted.{original_extension} with only chimeric records, format support fasta|fastq|mhap|paf")
             )
        .arg(Arg::with_name("split")
             .short("s")
             .long("split")
             .multiple(true)
             .display_order(45)
             .takes_value(true)
             .help("Create a new file {original_path}_splited.{original_extension} where chimeric records are split, format support fasta|fastq")
             )
        .arg(Arg::with_name("format")
             .short("F")
             .long("format")
             .display_order(50)
             .takes_value(true)
             .help("Force the format used")
             .possible_values(&["paf", "mhap"])
             )
        .arg(Arg::with_name("chimeric-threshold")
             .short("c")
             .display_order(60)
             .takes_value(true)
             .default_value("0")
             .long("chimeric-threshold")
             .help("Overlap depth threshold below which a gap should be created")
             )
        .arg(Arg::with_name("not-covered-threshold")
             .short("n")
             .display_order(70)
             .takes_value(true)
             .default_value("0.80")
             .long("not-covered-threshold")
             .help("Coverage depth threshold above which a read are marked as not covered")
             )
        .arg(Arg::with_name("filtered-suffix")
             .display_order(80)
             .takes_value(true)
             .long("filtered-suffix")
             .default_value("_filtered")
             .help("Change the suffix of file generate by filter option")
             )
        .arg(Arg::with_name("extracted-suffix")
             .display_order(90)
             .takes_value(true)
             .long("extracted-suffix")
             .default_value("_extracted")
             .help("Change the suffix of file generate by extract option")
             )
        .arg(Arg::with_name("splited-suffix")
             .display_order(95)
             .takes_value(true)
             .long("splited-suffix")
             .default_value("_splited")
             .help("Change the suffix of file generate by split option")
             )
        .arg(Arg::with_name("compression-out")
             .short("C")
             .display_order(100)
             .takes_value(true)
             .long("compression-out")
             .possible_values(&["gzip", "bzip2", "lzma", "no"])
             .help("Overlap depth threshold below which a gap should be created")
             )
        .get_matches();

    let mut compression: file::CompressionFormat = file::CompressionFormat::No;
    let mut inputs: Vec<Box<std::io::Read>> = Vec::new();
    for input_name in matches.values_of("input").unwrap() {
        let tmp = file::get_input(input_name);
        inputs.push(tmp.0);
        compression = tmp.1;
    }

    let out_compression = file::choose_compression(
        compression,
        matches.is_present("compression-out"),
        matches.value_of("compression-out").unwrap_or("no"),
    );
    let mut output: Box<std::io::Write> =
        file::get_output(matches.value_of("output").unwrap(), out_compression);

    let filters: Vec<_> = match matches.is_present("filter") {
        true => matches.values_of("filter").unwrap().collect(),
        false => Vec::new(),
    };
    let extracts: Vec<_> = match matches.is_present("extract") {
        true => matches.values_of("extract").unwrap().collect(),
        false => Vec::new(),
    };
    let splits: Vec<_> = match matches.is_present("split") {
        true => matches.values_of("split").unwrap().collect(),
        false => Vec::new(),
    };

    let filterd_suffix = matches.value_of("filtered-suffix").unwrap();
    let extract_suffix = matches.value_of("extracted-suffix").unwrap();
    let split_suffix = matches.value_of("splited-suffix").unwrap();

    let mut formats: Vec<utils::Format> = Vec::new();
    utils::get_mapping_format(&matches, &mut formats);;

    let chim_thres = matches
        .value_of("chimeric-threshold")
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let ncov_thres = matches
        .value_of("not-covered-threshold")
        .unwrap()
        .parse::<f64>()
        .unwrap();

    let mut remove_reads: chimera::BadReadMap = HashMap::new();

    chimera::find(
        inputs,
        &mut output,
        formats,
        chim_thres,
        ncov_thres,
        &mut remove_reads,
    );

    for filename in filters {
        filter::run(&remove_reads, filename, filterd_suffix);
    }

    for filename in extracts {
        extract::run(&remove_reads, filename, extract_suffix);
    }

    for filename in splits {
        split::run(&remove_reads, filename, split_suffix);
    }
}
