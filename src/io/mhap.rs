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

/* local use */

/* crates use */
use csv;

/* standard use */
use std;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    pub read_a: String,
    pub read_b: String,
    pub error: f64,
    pub shared_min_mers: u64,
    pub strand_a: char,
    pub begin_a: u64,
    pub end_a: u64,
    pub length_a: u64,
    pub strand_b: char,
    pub begin_b: u64,
    pub end_b: u64,
    pub length_b: u64,
}

pub fn get_reader<R: std::io::Read>(input: R) -> csv::Reader<R> {
    csv::ReaderBuilder::new()
        .delimiter(b' ')
        .has_headers(false)
        .from_reader(input)
}

pub fn get_writer<W: std::io::Write>(output: W) -> csv::Writer<W> {
    csv::WriterBuilder::new()
        .delimiter(b' ')
        .has_headers(false)
        .from_writer(output)
}

