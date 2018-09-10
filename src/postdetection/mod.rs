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

pub mod paf;
pub mod mhap;
pub mod fasta;
pub mod fastq;

pub fn in_read(begin: usize, end: usize, length: usize) -> bool {
    return begin < length || end < length;
}

pub fn generate_out_name(filename: String, suffix: &str) -> String {
    return filename.replacen(".", &format!("{}.", suffix), 1);
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn out_name() {
        assert_eq!(
            generate_out_name("test.paf".to_string(), "_test"),
            "test_test.paf"
        );
        assert_eq!(
            generate_out_name("test.paf.gz".to_string(), "_test"),
            "test_test.paf.gz"
        );
        assert_eq!(
            generate_out_name("test.fasta".to_string(), "_test"),
            "test_test.fasta"
        );
    }
}
