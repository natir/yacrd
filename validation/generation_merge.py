#!/usr/bin/env python3

import sys
import gzip
import random

from Bio import SeqIO
from Bio.Seq import Seq
from Bio.SeqRecord import SeqRecord

random.seed(42) # fix seed

output = gzip.open(sys.argv[2], "wt")
input_1 = gzip.open(sys.argv[1], "rt")
input_2 = gzip.open(sys.argv[1], "rt")
p = SeqIO.parse(input_1, "fastq")
o = SeqIO.parse(input_2, "fastq")
next(o)


read2 = ""

for i, (read1, read2) in enumerate(zip(p, o)):
    if i >= int(sys.argv[3]):
        break

    insert = "".join(random.sample(['A', 'C', 'T', 'G']*13, 50))
    record = SeqRecord(Seq(str(read1.seq) + insert + str(read2.seq)),
                       id=read1.id+"_"+read2.id,
                       letter_annotations={
                           "phred_quality": read1.letter_annotations["phred_quality"] + [13]*50 + read2.letter_annotations["phred_quality"]
                       },
                       description="")
    SeqIO.write(record, output, "fastq")

SeqIO.write(read2, output, "fastq")

for i in o:
    SeqIO.write(i, output, "fastq")

