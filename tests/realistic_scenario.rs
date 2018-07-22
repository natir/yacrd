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

extern crate flate2;
extern crate xz2;

use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::process::{Command, Stdio};

#[cfg(test)]
mod realistic_scenario {

    use super::*;

    #[test]
    fn default() {
        let child = Command::new("./target/debug/yacrd")
            .stdin(Stdio::from(fs::File::open("tests/data/test.paf").unwrap()))
            .stdout(Stdio::piped())
            .spawn()
            .expect("Could not run yacrd");

        assert_eq!(
            child.wait_with_output().unwrap().stdout,
            b"Chimeric\t1\t10000\t1000,4500,5500\n"
        );
    }

    #[test]
    fn file_mhap_gz_out_same_paf_default_default() {
        let child = Command::new("./target/debug/yacrd")
            .arg("-i")
            .arg("tests/data/test.mhap.gz")
            .arg("-F")
            .arg("mhap")
            .arg("-f")
            .arg("tests/data/test.paf")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Could ot run yacrd");

        assert_eq!(
            child.wait_with_output().unwrap().stdout,
            vec![
                31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 115, 206, 200, 204, 77, 45, 202, 76, 230, 52,
                228, 52, 52, 0, 2, 48, 169, 99, 98, 10, 36, 76, 129, 4, 23, 0, 157, 72, 78, 201,
                32, 0, 0, 0,
            ]
        );
        assert_eq!(fs::read("tests/data/test_filtered.paf").unwrap(), b"");

        fs::remove_file("tests/data/test_filtered.paf").unwrap();
    }

    #[test]
    fn file_mhapgzpafxz_out_same_paf_default_default() {
        let child = Command::new("./target/debug/yacrd")
            .arg("-i")
            .arg("tests/data/test.mhap.gz")
            .arg("tests/data/test.paf.bz2")
            .arg("-C")
            .arg("no")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Could ot run yacrd");

        assert_eq!(
            String::from_utf8_lossy(&child.wait_with_output().unwrap().stdout),
            "Chimeric\t1\t10000\t1000,4500,5500\n"
        );
    }

    #[test]
    fn in_paf_no_file_same_no_other_default() {
        let expected = "Chimeric	4	6000	1000,2500,3500
Chimeric	1	10000	2000,0,2000;1000,4500,5500;2000,8000,10000
";
        let good: HashSet<&str> = expected.split("\n").collect();

        Command::new("./target/debug/yacrd")
            .arg("-i")
            .arg("-")
            .arg("-o")
            .arg("tests/data/test.yacrd")
            .arg("-c")
            .arg("1")
            .stdin(Stdio::from(
                fs::File::open("tests/data/test_cov_1.paf").unwrap(),
            ))
            .stdout(Stdio::piped())
            .output()
            .expect("Could ot run yacrd");

        assert_eq!(
            String::from_utf8_lossy(&fs::read("tests/data/test.yacrd").unwrap())
                .split("\n")
                .collect::<HashSet<&str>>(),
            good
        );

        fs::remove_file("tests/data/test.yacrd").unwrap();
    }

    #[test]
    fn file_paf_bz_out_other_mhapfastq_default_other() {
        let expected = "Not_covered	2	10000	5500,0,5500
Chimeric	1	10000	1000,4500,5500
Not_covered	3	10000	5500,4500,10000
";
        let good: HashSet<&str> = expected.split("\n").collect();

        let child = Command::new("./target/debug/yacrd")
            .arg("-i")
            .arg("tests/data/test.paf.bz2")
            .arg("-o")
            .arg("-")
            .arg("-C")
            .arg("lzma")
            .arg("-n")
            .arg("0.5")
            .arg("-f")
            .arg("tests/data/test.mhap")
            .arg("tests/data/test.fastq.gz")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Could not run yacrd");

        {
            let output_raw: &[u8] = &child.wait_with_output().unwrap().stdout;
            let proxy: &[u8] = &output_raw;

            let mut output = Vec::new();
            xz2::read::XzDecoder::new(proxy)
                .read_to_end(&mut output)
                .unwrap();

            let proxy_deflate = String::from_utf8_lossy(&output);
            let result = proxy_deflate.split("\n").collect::<HashSet<&str>>();

            assert_eq!(good, result);
        }

        assert_eq!(fs::read("tests/data/test_filtered.mhap").unwrap(), vec![]);

        {
            let output_raw = fs::read("tests/data/test_filtered.fastq.gz").unwrap();
            let proxy: &[u8] = &output_raw;

            let mut output = Vec::new();
            flate2::read::GzDecoder::new(proxy)
                .read_to_end(&mut output)
                .unwrap();

            assert_eq!(output, b"@4\nACTG\n+\n!!!!\n");
        }

        fs::remove_file("tests/data/test_filtered.mhap").unwrap();
        fs::remove_file("tests/data/test_filtered.fastq.gz").unwrap();
    }

    #[test]
    fn file_mhap_xz_file_other_fasta_other_other() {
        let expected = "Not_covered	3	10000	7500,2500,10000
Not_covered	2	10000	7500,0,7500
Chimeric	4	6000	1000,2500,3500
Chimeric	1	10000	2000,0,2000;1000,4500,5500;2000,8000,10000
";

        let good: HashSet<&str> = expected.split("\n").collect();

        Command::new("./target/debug/yacrd")
            .arg("-i")
            .arg("tests/data/test_cov_1.mhap.xz")
            .arg("-o")
            .arg("tests/data/test.yacrd.gz")
            .arg("-C")
            .arg("gzip")
            .arg("-n")
            .arg("0.5")
            .arg("-c")
            .arg("1")
            .arg("-f")
            .arg("tests/data/test.fasta")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .output()
            .expect("Could not run yacrd");

        {
            let output_raw = fs::read("tests/data/test.yacrd.gz").unwrap();
            let proxy: &[u8] = &output_raw;

            let mut output = Vec::new();
            flate2::read::GzDecoder::new(proxy)
                .read_to_end(&mut output)
                .unwrap();
            println!("{:?}", String::from_utf8_lossy(&output));
            assert_eq!(
                String::from_utf8_lossy(&output)
                    .split("\n")
                    .collect::<HashSet<&str>>(),
                good
            );
        }

        assert_eq!(
            fs::read("tests/data/test_filtered.fasta").unwrap(),
            b">5\nACTG\n"
        );

        fs::remove_file("tests/data/test.yacrd.gz").unwrap();
        fs::remove_file("tests/data/test_filtered.fasta").unwrap();
    }
}
