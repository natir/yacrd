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
extern crate itertools;
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
mod cli;
mod file;
mod io;
mod overlap;
mod subcommand;
mod utils;

/* crates use */

/* standard use */
use std::collections::HashMap;

fn main() {
    let mut app = cli::app();
    let matches = match app.get_matches_from_safe_borrow(std::env::args()) {
        Ok(x) => x,
        Err(x) => x.exit(),
    };

    let subcmd = cli::get_subcmd(&mut app);

    /* Manage input and output file */
    let (input, _) = file::get_input(matches.value_of("input").unwrap());

    let mut format: utils::Format = utils::get_mapping_format(&matches);

    let format = utils::get_mapping_format(&matches);

    let chimeric_th = matches
        .value_of("chimeric-threshold")
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let not_covered_th = matches
        .value_of("not-covered-threshold")
        .unwrap()
        .parse::<f64>()
        .unwrap();

    let mut reads_info: chimera::BadReadMap = HashMap::new();

    overlap::find(input, format, chimeric_th, not_covered_th, &mut reads_info);

    for (cmd, arg) in subcmd {
        match cmd.as_ref() {
            "report" => subcommand::report::run(&reads_info, arg),
            "filter" => subcommand::filter::run(&reads_info, arg),
            "scrubbing" => subcommand::scrubbing::run(&reads_info, arg),

            _ => (),
        }
    }
    return;
}
