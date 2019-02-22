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
extern crate serde;
extern crate xz2;

#[macro_use]
extern crate enum_primitive;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

/* local mod */
mod chimera;
mod extract;
mod file;
mod filter;
mod io;
mod overlap;
mod split;
mod utils;

mod postdetection;

/* crates use */
use clap::{App, Arg, SubCommand};

/* standard use */
use std::collections::HashMap;

fn main() {
    let matches = App::new("yacrd")
        .version("0.5.1 Omanyte")
        .author("Pierre Marijon <pierre.marijon@inria.fr>")
        .about("Yet Another Chimeric Read Detector")
        .subcommand(SubCommand::with_name("chimeric")
                    .about("In chimeric mode yacrd detect chimera if coverage gap are in middle of read")
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
                    .arg(Arg::with_name("format")
                         .short("F")
                         .long("format")
                         .display_order(50)
                         .takes_value(true)
                         .help("Force the format used")
                         .possible_values(&["paf", "mhap"])
                    )
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
                         .help("Output compression format, the input compression format is chosen by default")
                    )
                    .arg(Arg::with_name("json-output")
                         .short("j")
                         .display_order(110)
                         .takes_value(false)
                         .long("json")
                         .help("Yacrd report are write in json format")
                    )
        )
        .subcommand(SubCommand::with_name("scrubbing")
                    .about("In scrubbing mode yacrd remove all part of read not covered")
                    .arg(Arg::with_name("mapping")
                         .short("m")
                         .long("mapping")
                         .display_order(10)
                         .takes_value(true)
                         .required(true)
                         .help("Path to mapping file in PAF or MHAP format (with .paf or .mhap extension, paf format by default)")
                    )
                    .arg(Arg::with_name("sequence")
                         .short("s")
                         .long("sequence")
                         .display_order(20)
                         .takes_value(true)
                         .required(true)
                         .help("Path to sequence you want scrubbed, format support fasta|fastq")
                    )
                    .arg(Arg::with_name("report")
                         .short("r")
                         .long("report")
                         .display_order(30)
                         .takes_value(true)
                         .required(true)
                         .help("Path where yacrd report are writen")
                    )
                    .arg(Arg::with_name("scrubbed")
                         .short("S")
                         .long("scrubbed")
                         .display_order(40)
                         .takes_value(true)
                         .default_value("-")
                         .required(true)
                         .help("Path where scrubbed read are write")
                    )
                    .arg(Arg::with_name("chimeric-threshold")
                         .short("c")
                         .display_order(50)
                         .takes_value(true)
                         .default_value("0")
                         .long("chimeric-threshold")
                         .help("Overlap depth threshold below which a gap should be created")
                    )
                    .arg(Arg::with_name("not-covered-threshold")
                         .short("n")
                         .display_order(60)
                         .takes_value(true)
                         .default_value("0.80")
                         .long("not-covered-threshold")
                         .help("Coverage depth threshold above which a read are marked as not covered")
                    )
                    .arg(Arg::with_name("format")
                         .short("M")
                         .long("mapping-format")
                         .display_order(70)
                         .takes_value(true)
                         .help("Force the format used")
                         .possible_values(&["paf", "mhap"])
                    )
                    .arg(Arg::with_name("json-output")
                         .short("j")
                         .display_order(110)
                         .takes_value(false)
                         .long("json")
                         .help("Yacrd report are write in json format")
                    )
        )
        .get_matches();

    if let Some(chimeric_matches) = matches.subcommand_matches("chimeric") {
        let mut compression: file::CompressionFormat = file::CompressionFormat::No;
        let mut inputs: Vec<Box<std::io::Read>> = Vec::new();

        for input_name in chimeric_matches.values_of("input").unwrap() {
            let tmp = file::get_input(input_name);
            inputs.push(tmp.0);
            compression = tmp.1;
        }

        let out_compression = file::choose_compression(
            compression,
            chimeric_matches.is_present("compression-out"),
            chimeric_matches.value_of("compression-out").unwrap_or("no"),
        );
        let mut output: Box<std::io::Write> = file::get_output(
            chimeric_matches.value_of("output").unwrap(),
            out_compression,
        );

        let filters: Vec<_> = match chimeric_matches.is_present("filter") {
            true => chimeric_matches.values_of("filter").unwrap().collect(),
            false => Vec::new(),
        };
        let extracts: Vec<_> = match chimeric_matches.is_present("extract") {
            true => chimeric_matches.values_of("extract").unwrap().collect(),
            false => Vec::new(),
        };
        let splits: Vec<_> = match chimeric_matches.is_present("split") {
            true => chimeric_matches.values_of("split").unwrap().collect(),
            false => Vec::new(),
        };

        let filterd_suffix = chimeric_matches.value_of("filtered-suffix").unwrap();
        let extract_suffix = chimeric_matches.value_of("extracted-suffix").unwrap();
        let split_suffix = chimeric_matches.value_of("splited-suffix").unwrap();

        let mut remove_reads: chimera::BadReadMap = HashMap::new();

        let mut formats: Vec<utils::Format> = Vec::new();
        utils::get_mapping_formats(&chimeric_matches, &mut formats);

        let chim_thres = chimeric_matches
            .value_of("chimeric-threshold")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let ncov_thres = chimeric_matches
            .value_of("not-covered-threshold")
            .unwrap()
            .parse::<f64>()
            .unwrap();

        overlap::find(
            inputs,
            formats,
            chim_thres,
            ncov_thres,
            &mut remove_reads,
            false,
        );

        chimera::write(
            &mut output,
            &remove_reads,
            chimeric_matches.is_present("json-output"),
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
    } else if let Some(scrubbing_matches) = matches.subcommand_matches("scrubbing") {
        let (mapping, map_compression) =
            file::get_input(scrubbing_matches.value_of("mapping").unwrap());
        let mut format =
            utils::get_mapping_format(scrubbing_matches.value_of("mapping").unwrap()).unwrap();

        let mut raw_path = scrubbing_matches.value_of("sequence").unwrap();
        let mut scrubbed_path = scrubbing_matches.value_of("scrubbed").unwrap();

        let out_compression = file::choose_compression(
            map_compression,
            scrubbing_matches.is_present("compression-out"),
            scrubbing_matches
                .value_of("compression-out")
                .unwrap_or("no"),
        );

        let mut report: Box<std::io::Write> = file::get_output(
            scrubbing_matches.value_of("report").unwrap(),
            out_compression,
        );

        let chim_thres = scrubbing_matches
            .value_of("chimeric-threshold")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let ncov_thres = scrubbing_matches
            .value_of("not-covered-threshold")
            .unwrap()
            .parse::<f64>()
            .unwrap();

        let mut remove_reads: chimera::BadReadMap = HashMap::new();

        overlap::find(
            vec![mapping],
            vec![format],
            chim_thres,
            ncov_thres,
            &mut remove_reads,
            true,
        );
        split::run(&remove_reads, raw_path, "_splited");

        chimera::write(
            &mut report,
            &remove_reads,
            scrubbing_matches.is_present("json-output"),
        );

        let tmp_name = postdetection::generate_out_name(raw_path, "_splited");

        std::fs::rename(tmp_name, scrubbed_path).unwrap();
    }
}
