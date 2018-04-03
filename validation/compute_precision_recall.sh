#!/bin/bash

./generation_merge.py longislnd_t_roseus.fastq.gz longislnd_t_roseus.chimeric.fastq.gz $1

minimap -x ava10k  longislnd_t_roseus.chimeric.fastq.gz longislnd_t_roseus.chimeric.fastq.gz > longislnd_t_roseus.minimap1.paf 2> /dev/null

TP=$(../build/yacrd -i longislnd_t_roseus.minimap1.paf | grep "_" -c)
FP=$(../build/yacrd -i longislnd_t_roseus.minimap1.paf | grep -v "_" -c)
FN=$(($1 - $TP))

echo "minimap :"
echo -ne "\tprecision : " 
echo "scale=2; ${TP}/(${TP}+${FP})" | bc
echo -ne "\trecall: "
echo "scale=2; ${TP}/(${TP}+${FN})" | bc

minimap2 -x ava-pb longislnd_t_roseus.chimeric.fastq.gz longislnd_t_roseus.chimeric.fastq.gz > longislnd_t_roseus.minimap2.paf 2> /dev/null

TP=$(../build/yacrd -i longislnd_t_roseus.minimap2.paf | grep "_" -c)
FP=$(../build/yacrd -i longislnd_t_roseus.minimap2.paf | grep -v "_" -c)
FN=$(($1 - $TP))

echo "minimap2 :"
echo -ne "\tprecision : " 
echo "scale=2; ${TP}/(${TP}+${FP})" | bc
echo -ne "\trecall : "
echo "scale=2; ${TP}/(${TP}+${FN})" | bc

