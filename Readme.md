# Yet Another Chimeric Read Detector for long reads

[![Build Status](https://travis-ci.org/natir/yacrd.svg?branch=master)](https://travis-ci.org/natir/yacrd)

![yacrd pipeline presentation](image/pipeline.svg)

Using all-against-all read mapping, yacrd performs:

1. computation of pile-up coverage for each read
2. detection of chimeras

Chimera detection is done as follows:

1. for each region where coverage is smaller or equal than `min_coverage` (default 0), yacrd creates a _gap_.
2. if there is a gap that starts at a position strictly after the beginning of the read and ends strictly before the end of the read, the read is marked as `Chimeric`
3. if gaps length of extremity > 0.8 * read length, the read is marked as `Not_covered`

## Rationale

Long read error-correction tools usually detect and also remove chimeras. But it is difficult to isolate or retrieve information from just this step.

DAStrim (from the [DASCRUBBER suite](https://github.com/thegenemyers/DASCRUBBER) does a similar job to yacrd but relies on a different mapping step, and uses different (likely more advanced) heuristics. Yacrd is simpler and easier to use.

## Input

Any set of long reads (PacBio, Nanopore, anything that can be given to [minimap2](https://github.com/lh3/minimap2) ).
yacrd takes the resulting PAF (Pairwise Alignement Format) from minimap2 or MHAP file from some other long reads overlapper as input.

## Requirements

- [Rust](https://www.rust-lang.org/) in stable channel

## Instalation

### With cargo

If you have a rust environment setup you can run :

```
cargo install yacrd
```

### With conda

yacrd is avaible in [bioconda channel](https://bioconda.github.io/)

if bioconda channel is setup you can run :

```
conda install yacrd
```

### From source

```
git clone https://github.com/natir/yacrd.git
cd yacrd
git checkout v0.3

cargo build
cargo test
cargo install
```

## Usage

1) Run Minimap2: `minimap2 reads.fq reads.fq > mapping.paf` or any other long reads overlapper.
2)

```
yacrd 0.3 Ninetales
Pierre Marijon <pierre.marijon@inria.fr>
Yet Another Chimeric Read Detector

USAGE:
    yacrd [-i|--input] <input> [-o|--output] <output> [-f|--filter] <file1, file2, …> 
	yacrd -i map_file.paf -o map_file.yacrd
	yacrd -i map_file.mhap -o map_file.yacrd
	yacrd -i map_file.xyz -F paf -o map_file.yacrd
	yacrd -i map_file.paf -f sequence.fasta -o map_file.yacrd
	zcat map_file.paf.gz | yacrd -i - -o map_file.yacrd
	minimap2 sequence.fasta sequence.fasta | yacrd -o map_file.yacrd --fileterd-suffix _test -f sequence.fastq sequence2.fasta other.fastq
	Or any combination of this.

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input <input>
            Mapping input file in PAF or MHAP format (with .paf or .mhap extension), use - for read standard input (no
            compression allowed, paf format by default) [default: -]
    -o, --output <output>
            Path where yacrd report are writen, use - for write in standard output same compression as input or use
            --compression-out [default: -]
    -f, --filter <filter>...
            File containing reads that will be filtered (fasta|fastq|mhap|paf), new file are create like
            {original_path}_fileterd.{original_extension}
    -F, --format <format>                                  Force the format used [possible values: paf, mhap]
    -c, --chimeric-threshold <chimeric-threshold>
            Overlap depth threshold below which a gap should be created [default: 0]

    -n, --not-covered-threshold <not-covered-threshold>
            Coverage depth threshold above which a read are marked as not covered [default: 0.80]

        --filtered-suffix <filtered-suffix>
            Change the suffix of file generate by filter option [default: _filtered]

    -C, --compression-out <compression-out>
            Overlap depth threshold below which a gap should be created [possible values: gzip, bzip2, lzma, no]
```

## Output

```
type_of_read	id_in_mapping_file  length_of_read  length_of_gap,begin_pos_of_gap,end_pos_of_gap;length_of_gap,be…
```

### Example

```
Not_covered readA 4599	3782,0,3782
```

Here, readA doesn't have sufficient coverage, there is a zero-coverage region of length 3782bp between positions 0 and 3782.

```
Chimeric    readB   10452   862,1260,2122;3209,4319,7528
```

Here, readB is chimeric with 2 zero-coverage regions: one between bases 1260 and 2122, another between 3209 and 7528.
