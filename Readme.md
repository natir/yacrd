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
- libgz
- libbzip2
- liblzma

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
git checkout v0.4

cargo build
cargo test
cargo install
```

## How to use Yacrd

### Find chimera

```
minimap2 reads.fq reads.fq | yacrd -o reads.yacrd
```

If you want save mapping intermediate file:
```
minimap2 reads.fq reads.fq > mapping.paf
yacrd -i mapping.paf -o reads.yacrd
```

### Find chimera and run post-detection operation

yacrd can perform thrid post-detection operation, on mapping or sequence file:

- filtering: yacrd generate a new file with only record without chimeric reads
- extracting: yacrd generate a new file with only record with chimeric reads
- spliting (only on sequence file) : yacrd generate a new file without chimeric region 

```
minimap2 reads.fq reads.fq > mapping.paf
yacrd -i mapping.paf -f reads.fasta > reads.yacrd # produce reads_fileterd.fasta
yacrd -i mapping.paf -e reads.fasta > reads.yacrd # produce reads_extracted.fasta
yacrd -i mapping.paf -s reads.fasta > reads.yacrd # produce reads_splited.fasta
```

### Complet Usage

```
yacrd 0.4.1 Hypno
Pierre Marijon <pierre.marijon@inria.fr>
Yet Another Chimeric Read Detector

USAGE:
    yacrd [-i|--input] <input1, input2, …> [-o|--output] <output> [-f|--filter] <file1, file2, …>
	yacrd -i map_file.paf -o map_file.yacrd
	yacrd -i map_file.mhap -o map_file.yacrd
	yacrd -i map_file.xyz -F paf -o map_file.yacrd
	yacrd -i map_file.paf -f sequence.fasta -o map_file.yacrd
	zcat map_file.paf.gz | yacrd -i - -o map_file.yacrd
	minimap2 sequence.fasta sequence.fasta | yacrd -o map_file.yacrd --fileterd-suffix _test -f sequence.fastq sequence2.fasta other.fastq
	Or any combination of this.

FLAGS:
    -j, --json	     Yacrd report are write in json format
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input <input>...
            Mapping input file in PAF or MHAP format (with .paf or .mhap extension), use - for read standard input (no
            compression allowed, paf format by default) [default: -]
    -o, --output <output>
            Path where yacrd report are writen, use - for write in standard output same compression as input or use
            --compression-out [default: -]
    -f, --filter <filter>...
            Create a new file {original_path}_fileterd.{original_extension} with only not chimeric records, format
            support fasta|fastq|mhap|paf
    -e, --extract <extract>...
            Create a new file {original_path}_extracted.{original_extension} with only chimeric records, format support
            fasta|fastq|mhap|paf
    -s, --split <split>...
            Create a new file {original_path}_splited.{original_extension} where chimeric records are split, format
            support fasta|fastq
    -F, --format <format>                                  Force the format used [possible values: paf, mhap]
    -c, --chimeric-threshold <chimeric-threshold>
            Overlap depth threshold below which a gap should be created [default: 0]

    -n, --not-covered-threshold <not-covered-threshold>
            Coverage depth threshold above which a read are marked as not covered [default: 0.80]

        --filtered-suffix <filtered-suffix>
            Change the suffix of file generate by filter option [default: _filtered]

        --extracted-suffix <extracted-suffix>
            Change the suffix of file generate by extract option [default: _extracted]

        --splited-suffix <splited-suffix>
            Change the suffix of file generate by split option [default: _splited]

    -C, --compression-out <compression-out>
	    Output compression format, the input compression format is chosen by default [possible values: gzip, bzip2,
	    lzma, no]
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

### JSON

If flag `-j` are present output are write in json format, an example:

```
{
	"1": {
		"gaps": [{
			"begin": 0,
			"end": 2000
		}, {
			"begin": 4500,
			"end": 5500
		}, {
			"begin": 8000,
			"end": 10000
		}],
		"length": 10000,
		"type": "Chimeric"
	},
	"4": {
		"gaps": [{
			"begin": 2500,
			"end": 3500
		}],
		"length": 6000,
		"type": "Chimeric"
	}
}
```
