/*
Copyright (c) 2019 Pierre Marijon <pmarijon@mpi-inf.mpg.de>

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

/* crate use */
use thiserror::Error;

/* local use */
use crate::util;

#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "Reading of the file '{filename:}' impossible, does it exist and can be read by the user?"
    )]
    CantReadFile { filename: String },

    #[error("Creation/opening of the file '{filename:}' impossible, directory in path exist? can be written by the user?")]
    CantWriteFile { filename: String },

    #[error("Format detection for '{filename:}' file not possible, filename need to contains .fasta, .fa, .fastq, fq, .paf, .m4, .mhap or .yacrd")]
    UnableToDetectFileFormat { filename: String },

    #[error(
        "This operation {operation:} can't be run on this type ({filetype:?}) of file {filename:}"
    )]
    CantRunOperationOnFile {
        operation: String,
        filetype: util::FileType,
        filename: String,
    },

    #[error("Error durring reading of file {filename:} in format {format:?}")]
    ReadingError {
        filename: String,
        format: util::FileType,
    },

    #[error("Error during reading a file in format {format:?}")]
    ReadingErrorNoFilename { format: util::FileType },

    #[error("Error during writing of file {filename:} in format {format:?}")]
    WritingError {
        filename: String,
        format: util::FileType,
    },

    #[error("Error during writing of file in format {format:?}")]
    WritingErrorNoFilename { format: util::FileType },

    #[error("Error during yacrd overlap path creation {path:?}")]
    PathCreationError { path: std::path::PathBuf },

    #[error("Error during yacrd overlap path destruction {path:?}")]
    PathDestructionError { path: std::path::PathBuf },

    #[error("If you get this error please contact the author with this message and command line you use: {name:?}")]
    NotReachableCode { name: String },

    #[error("Yacrd postion seems corrupt")]
    CorruptYacrdReportInPosition,

    #[error("Your yacrd file {name} seems corrupt at line {line} you probably need to relaunch analisys with overlapping file")]
    CorruptYacrdReport { name: String, line: usize },
}
