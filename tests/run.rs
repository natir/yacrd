/*
Copyright (c) 2020 Pierre Marijon <pmarijon@mpi-inf.mpg.de>

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

/* std use */
use std::io::BufRead;
use std::io::Read;
use std::process::{Command, Stdio};

#[cfg(test)]
mod tests {

    use super::*;

    fn diff_unorder(truth: &str, result: &str) {
        let truth_file = std::io::BufReader::new(
            std::fs::File::open(truth).expect(&format!("Impossible to open {}", truth)),
        );

        let mut truth: std::collections::HashSet<String> = std::collections::HashSet::new();

        for res in truth_file.lines() {
            let line = res.unwrap();
            truth.insert(line);
        }

        let result_file = std::io::BufReader::new(
            std::fs::File::open(result).expect(&format!("Impossible to open {}", result)),
        );

        let mut result: std::collections::HashSet<String> = std::collections::HashSet::new();

        for res in result_file.lines() {
            let line = res.unwrap();
            result.insert(line);
        }

        assert_eq!(truth, result);
    }

    fn diff(truth: &str, result: &str) {
        let truth_file = std::io::BufReader::new(
            std::fs::File::open(truth).expect(&format!("Impossible to open {}", truth)),
        );

        let mut truth: Vec<String> = Vec::new();

        for res in truth_file.lines() {
            let line = res.unwrap();
            truth.push(line);
        }

        let result_file = std::io::BufReader::new(
            std::fs::File::open(result).expect(&format!("Impossible to open {}", result)),
        );

        let mut result: Vec<String> = Vec::new();

        for res in result_file.lines() {
            let line = res.unwrap();
            result.push(line);
        }

        assert_eq!(truth, result);
    }

    #[test]
    fn detection() {
        let mut child = Command::new("./target/debug/yacrd")
            .args(&["-i", "tests/reads.paf", "-o", "tests/result.yacrd"])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Couldn't create yacrd subprocess");

        if !child.wait().expect("Error durring yacrd run").success() {
            let mut stdout = String::new();
            let mut stderr = String::new();

            child.stdout.unwrap().read_to_string(&mut stdout).unwrap();
            child.stderr.unwrap().read_to_string(&mut stderr).unwrap();

            println!("stdout: {}", stdout);
            println!("stderr: {}", stderr);
            panic!();
        }

        diff_unorder("tests/truth.yacrd", "tests/result.yacrd");
    }

    #[test]
    fn detection_ondisk() {
        if cfg!(windows) {
            ()
        } else {
            if std::path::Path::new("tests/ondisk").exists() {
                std::fs::remove_dir_all(std::path::Path::new("tests/ondisk"))
                    .expect("We can't delete temporary directory of ondisk test");
            }

            std::fs::create_dir(std::path::Path::new("tests/ondisk"))
                .expect("We can't create temporary directory for ondisk test");

            let mut child = Command::new("./target/debug/yacrd")
                .args(&[
                    "-i",
                    "tests/reads.paf",
                    "-o",
                    "tests/result.ondisk.yacrd",
                    "-d",
                    "tests/ondisk",
                ])
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Couldn't create yacrd subprocess");

            if !child.wait().expect("Error durring yacrd run").success() {
                let mut stdout = String::new();
                let mut stderr = String::new();

                child.stdout.unwrap().read_to_string(&mut stdout).unwrap();
                child.stderr.unwrap().read_to_string(&mut stderr).unwrap();

                println!("stdout: {}", stdout);
                println!("stderr: {}", stderr);
                panic!();
            }

            diff_unorder("tests/truth.yacrd", "tests/result.ondisk.yacrd");
        }
    }

    #[test]
    fn filter() {
        let mut child = Command::new("./target/debug/yacrd")
            .args(&[
                "-i",
                "tests/reads.paf",
                "-o",
                "tests/result.filter.yacrd",
                "filter",
                "-i",
                "tests/reads.fastq",
                "-o",
                "tests/reads.filter.fastq",
            ])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Couldn't create yacrd subprocess");

        if !child.wait().expect("Error durring yacrd run").success() {
            let mut stdout = String::new();
            let mut stderr = String::new();

            child.stdout.unwrap().read_to_string(&mut stdout).unwrap();
            child.stderr.unwrap().read_to_string(&mut stderr).unwrap();

            println!("stdout: {}", stdout);
            println!("stderr: {}", stderr);
            panic!();
        }

        diff_unorder("tests/truth.yacrd", "tests/result.filter.yacrd");
        diff("tests/truth.filter.fastq", "tests/reads.filter.fastq")
    }

    #[test]
    fn extract() {
        let mut child = Command::new("./target/debug/yacrd")
            .args(&[
                "-i",
                "tests/reads.paf",
                "-o",
                "tests/result.extract.yacrd",
                "extract",
                "-i",
                "tests/reads.fastq",
                "-o",
                "tests/reads.extract.fastq",
            ])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Couldn't create yacrd subprocess");

        if !child.wait().expect("Error durring yacrd run").success() {
            let mut stdout = String::new();
            let mut stderr = String::new();

            child.stdout.unwrap().read_to_string(&mut stdout).unwrap();
            child.stderr.unwrap().read_to_string(&mut stderr).unwrap();

            println!("stdout: {}", stdout);
            println!("stderr: {}", stderr);
            panic!();
        }

        diff_unorder("tests/truth.yacrd", "tests/result.extract.yacrd");
        diff("tests/truth.extract.fastq", "tests/reads.extract.fastq")
    }

    #[test]
    fn split() {
        let mut child = Command::new("./target/debug/yacrd")
            .args(&[
                "-i",
                "tests/reads.paf",
                "-o",
                "tests/result.split.yacrd",
                "split",
                "-i",
                "tests/reads.fastq",
                "-o",
                "tests/reads.split.fastq",
            ])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Couldn't create yacrd subprocess");

        if !child.wait().expect("Error durring yacrd run").success() {
            let mut stdout = String::new();
            let mut stderr = String::new();

            child.stdout.unwrap().read_to_string(&mut stdout).unwrap();
            child.stderr.unwrap().read_to_string(&mut stderr).unwrap();

            println!("stdout: {}", stdout);
            println!("stderr: {}", stderr);
            panic!();
        }

        diff_unorder("tests/truth.yacrd", "tests/result.split.yacrd");
        diff("tests/truth.split.fastq", "tests/reads.split.fastq")
    }

    #[test]
    fn scrubb() {
        let mut child = Command::new("./target/debug/yacrd")
            .args(&[
                "-i",
                "tests/reads.paf",
                "-o",
                "tests/result.scrubb.yacrd",
                "scrubb",
                "-i",
                "tests/reads.fastq",
                "-o",
                "tests/reads.scrubb.fastq",
            ])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Couldn't create yacrd subprocess");

        if !child.wait().expect("Error durring yacrd run").success() {
            let mut stdout = String::new();
            let mut stderr = String::new();

            child.stdout.unwrap().read_to_string(&mut stdout).unwrap();
            child.stderr.unwrap().read_to_string(&mut stderr).unwrap();

            println!("stdout: {}", stdout);
            println!("stderr: {}", stderr);
            panic!();
        }

        diff_unorder("tests/truth.yacrd", "tests/result.scrubb.yacrd");
        diff("tests/truth.scrubb.fastq", "tests/reads.scrubb.fastq")
    }
}
