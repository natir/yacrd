# Yet Another Chimeric Read Detector

yacrd use reads against reads mapping to find chimeric reads.


## Mapping Format

Actualy yacrd support only Pairwise Alignement Format.


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

After building you can move/copy yacrd exectuable binary in your PATH


## Usage

```
yacrd -i mapping.paf
```

yacrd write in standard output the id of chimeric read
