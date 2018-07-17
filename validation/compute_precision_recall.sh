#!/bin/bash

./validation/generation_merge.py validation/longislnd_t_roseus.fastq.gz validation/longislnd_t_roseus.chimeric.fastq.gz $1

minimap -x ava10k validation/longislnd_t_roseus.chimeric.fastq.gz validation/longislnd_t_roseus.chimeric.fastq.gz > validation/longislnd_t_roseus.minimap1.paf 2> /dev/null

TP=$(cargo run -- -i validation/longislnd_t_roseus.minimap1.paf | grep Chimeric | grep "_" -c)
FP=$(cargo run -- -i validation/longislnd_t_roseus.minimap1.paf | grep Chimeric | grep -v "_" -c)
FN=$(($1 - $TP))

echo "minimap :"
echo -ne "\tprecision : " 
echo "scale=2; ${TP}/(${TP}+${FP})" | bc
echo -ne "\trecall: "
echo "scale=2; ${TP}/(${TP}+${FN})" | bc

minimap2 -x ava-pb validation/longislnd_t_roseus.chimeric.fastq.gz validation/longislnd_t_roseus.chimeric.fastq.gz > validation/longislnd_t_roseus.minimap2.paf 2> /dev/null

TP=$(cargo run -- -i validation/longislnd_t_roseus.minimap2.paf | grep Chimeric | grep "_" -c)
FP=$(cargo run -- -i validation/longislnd_t_roseus.minimap2.paf | grep Chimeric | grep -v "_" -c)
FN=$(($1 - $TP))

echo "minimap2 :"
echo -ne "\tprecision : " 
echo "scale=2; ${TP}/(${TP}+${FP})" | bc
echo -ne "\trecall : "
echo "scale=2; ${TP}/(${TP}+${FN})" | bc

