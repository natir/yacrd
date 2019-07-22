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

use clap::ArgMatches;

use chimera;
use file;
use io;
use utils;

pub fn run<'a>(reads_info: &chimera::BadReadMap, matches: ArgMatches<'a>) {
    let (input, output, format, chimeric, not_covered, not_bad) = parse(matches);

    match format {
        utils::Format::Paf => paf(input, output, reads_info, chimeric, not_covered, not_bad),
        _ => (),
    }
}

fn parse<'a>(
    matches: ArgMatches,
) -> (
    std::io::BufReader<Box<std::io::Read>>,
    std::io::BufWriter<Box<std::io::Write>>,
    utils::Format,
    bool,
    bool,
    bool,
) {
    let input_path = matches.value_of("input").unwrap();
    let (input, compression) = file::get_readable_file(input_path);

    let output = Box::new(file::get_output(
        matches.value_of("output").unwrap(),
        compression,
    ));

    let chimeric: bool = matches.is_present("chimeric");
    let not_covered: bool = matches.is_present("not-covered");
    let not_bad: bool = matches.is_present("not-to-bad");

    return (
        std::io::BufReader::new(input),
        std::io::BufWriter::new(output),
        utils::get_format(input_path).unwrap(),
        chimeric,
        not_covered,
        not_bad,
    );
}

fn paf(
    input: std::io::BufReader<Box<std::io::Read>>,
    output: std::io::BufWriter<Box<std::io::Write>>,
    reads_info: &chimera::BadReadMap,
    chimeric: bool,
    not_covered: bool,
    not_bad: bool,
) -> () {
    let mut reader = io::paf::Reader::new(input);
    let mut writer = io::paf::Writer::new(output);

    for result in reader.records() {
        let record = result.unwrap();

        let read_names = vec![&(record.read_a), &(record.read_b)];
        if read_names
            .iter()
            .all(|x| keep_this_read(x, reads_info, chimeric, not_covered, not_bad))
        {
            writer.write(&record);
        }
    }
}

fn keep_this_read(
    id: &String,
    reads_info: &chimera::BadReadMap,
    chimeric: bool,
    not_covered: bool,
    not_bad: bool,
) -> bool {
    let (label, _, _) = reads_info.get(id).unwrap();

    if label == &chimera::BadReadType::Chimeric && chimeric {
        return false;
    }
    if label == &chimera::BadReadType::NotCovered && not_covered {
        return false;
    }
    if label == &chimera::BadReadType::NotBad && not_bad {
        return false;
    }

    return true;
}
