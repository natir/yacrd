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

use std::io::Write;

use clap::ArgMatches;

use chimera;
use file;

pub fn run<'a>(reads_info: &chimera::BadReadMap, matches: ArgMatches<'a>) {
    let (mut output, chimeric, not_covered, not_bad, json) = parse(matches);

    for (id, (label, len, gaps)) in reads_info {
        if label == &chimera::BadReadType::Chimeric && chimeric {
            continue;
        }
        if label == &chimera::BadReadType::NotCovered && not_covered {
            continue;
        }
        if label == &chimera::BadReadType::NotBad && not_bad {
            continue;
        }

        if json {
            let mut map = serde_json::map::Map::new();
            map.insert(
                id.to_string(),
                json!({
                    "type": label.as_str(),
                    "length": len,
                    "gaps": gaps
                }),
            );
            output
                .write(&json!(map).to_string().into_bytes())
                .expect("Error durring write result in json format");
        } else {
            write_result(&mut output, &label, &id, &len, &gaps);
        }
    }
}

fn parse<'a>(
    matches: ArgMatches,
) -> (
    std::io::BufWriter<Box<dyn std::io::Write>>,
    bool,
    bool,
    bool,
    bool,
) {
    let output: std::io::BufWriter<Box<dyn std::io::Write>> =
        std::io::BufWriter::new(file::get_output(
            matches.value_of("output").unwrap(),
            file::CompressionFormat::No,
        ));

    let chimeric: bool = matches.is_present("chimeric");
    let not_covered: bool = matches.is_present("not-covered");
    let not_bad: bool = matches.is_present("not-to-bad");
    let json: bool = matches.is_present("json");

    return (output, chimeric, not_covered, not_bad, json);
}

pub fn write_result<W: std::io::Write>(
    mut output: &mut W,
    label: &chimera::BadReadType,
    name: &str,
    len: &u64,
    gaps: &Vec<chimera::Interval>,
) {
    output
        .write_fmt(format_args!("{}\t{}\t{}\t", label.as_str(), name, len))
        .expect("Error durring writting of result");

    for (i, interval) in gaps.iter().enumerate() {
        write_gap(interval, &mut output, gaps.len() - i);
    }

    output
        .write(b"\n")
        .expect("Error durring writting of result");
}

fn write_gap<W: std::io::Write>(gap: &chimera::Interval, output: &mut W, i: usize) {
    output
        .write_fmt(format_args!(
            "{},{},{}",
            gap.end - gap.begin,
            gap.begin,
            gap.end
        ))
        .expect("Error durring writting of result");
    if i > 1 {
        output
            .write(b";")
            .expect("Error durring writting of result");
    }
}
