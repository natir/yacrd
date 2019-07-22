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

/* crates use */
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("yacrd")
        .version("0.6 Mewtow")
        .author("Pierre Marijon <pierre@marijon.fr>")
        .about("\nyacrd detect poor quality reads region, reads was classify in three type:\n\t- chimeric: a bad region was identify in middle of read\n\t- Not covered: a majority of read was bad region\n\t- Not to bad: all other read")
        .arg(Arg::with_name("input")
             .short("i")
             .long("input")
             .takes_value(true)
             .default_value("-")
             .help("mapping input file in PAF or M4 format (with .paf or .mhap extension), use - for read standard input (no compression allowed, paf format by default)")
        )
        .arg(Arg::with_name("chimeric-threshold")
             .short("c")
             .takes_value(true)
             .default_value("0")
             .long("chimeric-threshold")
             .help("if a region of read have a depth below this value this region was marked as read")
        )
        .arg(Arg::with_name("not-covered-threshold")
             .short("n")
             .takes_value(true)
             .default_value("0.40")
             .long("not-covered-threshold")
             .help("if ratio 'bad region length' on 'total read length' are upper than this value read was marked as not covered")
        )
        .arg(Arg::with_name("format")
             .short("f")
             .long("format")
             .display_order(50)
             .takes_value(true)
             .help("overwrite format")
             .possible_values(&["paf", "mhap"])
        )
        .subcommand(SubCommand::with_name("report")
                    .setting(clap::AppSettings::AllowExternalSubcommands)
                    .about("How yacrd output her analysis")
                    .arg(Arg::with_name("output")
                         .short("o")
                         .long("(output")
                         .takes_value(true)
                         .default_value("-")
                         .help("path where yacrd report are write, use '-' for write in standard output same compression as input")
                    )
                    .arg(Arg::with_name("chimeric")
                         .short("c")
                         .long("chimeric")
                         .help("if this option is present, report didn't contains information on chimeric read")
                    )
                    .arg(Arg::with_name("not-covered")
                         .short("n")
                         .long("not-covered")
                         .help("if this option is present, report didn't contains information on not covered read")
                    )
                    .arg(Arg::with_name("not-to-bad")
                         .short("b")
                         .long("not-to-bad")
                         .help("if this option is present, report didn't contains information on not to bad read"))
                    .arg(Arg::with_name("json")
                         .short("j")
                         .long("json")
                         .help("yacrd report are write in json format")
                    )
        )
        .subcommand(SubCommand::with_name("filter")
                    .setting(clap::AppSettings::AllowExternalSubcommands)
                    .about("Yacrd filter record contains a read are in category")
                    .arg(Arg::with_name("input")
                         .short("i")
                         .long("input")
                         .takes_value(true)
                         .help("file you want filter record, support format fasta|fastq|mhap|paf")
                    )
                    .arg(Arg::with_name("output")
                         .short("o")
                         .long("output")
                         .takes_value(true)
                         .help("file where filter record was write, same format and compression as input")
                    )
                    .arg(Arg::with_name("chimeric")
                         .short("c")
                         .long("chimeric")
                         .help("if is present record with chimeric reads was filter out")
                    )
                    .arg(Arg::with_name("not-covered")
                         .short("n")
                         .long("not-covered")
                         .help("if is present record with not-covered reads was filter out")
                    )
                    .arg(Arg::with_name("not-to-bad")
                         .short("b")
                         .long("not-to-bad")
                         .help("if is present record with not-to-bad reads was filter out")
                    )
        )
        .subcommand(SubCommand::with_name("extract")
                    .setting(clap::AppSettings::AllowExternalSubcommands)
                    .about("Yacrd extract record contains a read are in category")
                    .arg(Arg::with_name("input")
                         .short("i")
                         .long("input")
                         .takes_value(true)
                         .help("file you want extract record, support format fasta|fastq|mhap|paf")
                    )
                    .arg(Arg::with_name("output")
                         .short("o")
                         .long("output")
                         .takes_value(true)
                         .help("file where extract record was write, same format and compression as input")
                    )
                    .arg(Arg::with_name("chimeric")
                         .short("c")
                         .long("chimeric")
                         .help("if is present chimeric reads was extract out")
                    )
                    .arg(Arg::with_name("not-covered")
                         .short("n")
                         .long("not-covered")
                         .help("if is present not-covered reads was extract out")
                    )
                    .arg(Arg::with_name("not-to-bad")
                         .short("b")
                         .long("not-to-bad")
                         .help("if is present not-to-bad reads was extract out")
                    )
        )
        .subcommand(SubCommand::with_name("split")
                    .setting(clap::AppSettings::AllowExternalSubcommands)
                    .about("Yacrd remove bad quality region in middle of reads, aka chimeric region")
                    .arg(Arg::with_name("input")
                         .short("i")
                         .long("input")
                         .takes_value(true)
                         .help("file of reads where you want chimeric was removed")
                    )
                    .arg(Arg::with_name("output")
                         .short("o")
                         .long("output")
                         .takes_value(true)
                         .help("file where reads without chimeric region of reads was write")
                    )
        )
        .subcommand(SubCommand::with_name("scrubbing")
                    .setting(clap::AppSettings::AllowExternalSubcommands)
                    .about("Yacrd remove bad quality region of reads")
                    .arg(Arg::with_name("input")
                         .short("i")
                         .long("input")
                         .takes_value(true)
                         .help("file of reads where you want all bad region was removed")
                    )
                    .arg(Arg::with_name("output")
                         .short("o")
                         .long("output")
                         .takes_value(true)
                         .help("file where only good region of reads was write")
                    )
        )
}

pub fn get_subcmd<'a, 'b>(
    app: &mut App<'a, 'b>,
) -> std::collections::HashMap<String, ArgMatches<'a>> {
    let basic_cli = vec!["yacrd".to_string(), "-i".to_string(), "foo".to_string()];
    let mut sub2matches = std::collections::HashMap::new();

    let mut cli: Vec<String> = std::env::args().collect();
    loop {
        /* parse cli */
        let matches = match app.get_matches_from_safe_borrow(cli) {
            Ok(x) => x,
            Err(x) => x.exit(),
        };

        let (name, sub) = match matches.subcommand() {
            (n, Some(s)) => (n, s),
            (_, None) => break,
        };

        sub2matches.insert(name.to_string(), sub.clone());

        let (subname, subsub) = match sub.subcommand() {
            (n, Some(s)) => (n, s),
            (_, None) => break,
        };

        if subsub.values_of("").is_none() {
            break;
        }

        /* rebuild a new cli*/
        cli = basic_cli.clone();
        cli.push(subname.to_string());
        cli.extend(subsub.values_of("").unwrap().map(|x| x.to_string()));
    }

    return sub2matches;
}
