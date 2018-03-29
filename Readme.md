# Yet Another Chimeric Read Detector for long reads

yacrd performs:

1. computation of pile-up coverage for each read
2. detection of chimeras

Chimera detection is done as follows:

1. each region where coverage is minus or equal to `min_coverage` (default 0), yacrd creates a /gap/.
2. if a gap is in middle of a read, the read is marked as `Chimeric`
3. if a gap is of size > 0.8 * read length, the read is marked as `Not_covered`

## Input

PAF (Pairwise Alignement Format) file produced by [https://github.com/lh3/minimap2](minimap2).

## Requirements

- CMake >= 2.8

## Building

```
git clone https://github.com/natir/yacrd.git
cd yacrd
mkdir build
cd build
cmake ..
make
```

After building, you can move/copy/add yacrd exectuable binary in your PATH


## Usage

```
usage: yacrd [-h] [-c coverage_min] -i mapping.paf

Options:
        -h                   Print help message
        -c,--min_coverage    If coverage are minus or equal to this create a gap [0]
        -i,--in              Maping input file

```

yacrd writes to standard output (stdout) the id of chimeric or not sufficiently covered reads.

## Output

```
type_of_read:id_in_mapping_file,length_of_read:length_of_gap,begin_pos_of_gap,end_pos_of_gap;length_of_gap,be…
```

### Example

```
Not_covered:readA,4599;3782,0,3782;
```

Here, readA doesn't have sufficient coverage, there is a zero-coverage region of length 3782bp between positions 0 and 3782.

```
Chimeric:readB,10452;862,1260,2122;3209,4319,7528;
```

Here, readB is chimeric with 2 zero-coverage regions: one between bases 1260 and 2122, another and 3209 to 7528.
