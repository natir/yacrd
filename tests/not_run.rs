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

static HELP_MESSAGE: &'static str = "yacrd 0.5.1 Omanyte
Pierre Marijon <pierre.marijon@inria.fr>
Yet Another Chimeric Read Detector

USAGE:
    yacrd [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    chimeric     In chimeric mode yacrd detect chimera if coverage gap are in middle of read
    help         Prints this message or the help of the given subcommand(s)
    scrubbing    In scrubbing mode yacrd remove all part of read not covered
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

        assert_eq!(output.stdout, b"yacrd 0.5.1 Omanyte\n");
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
        let output = Command::new("./target/debug/yacrd")
            .output()
            .expect("Could not run yacrd");

        assert_eq!(&output.stdout, b"");
    }
}
