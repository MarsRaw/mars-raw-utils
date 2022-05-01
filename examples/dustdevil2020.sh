#!/bin/bash

sol=$1
seqid=NCAM00525

if [ "x$2" != "x" ];then
    seqid=$2
fi

cd /data/M20

soldir=$sol
if [ $soldir -lt 1000 ]; then
    soldir=0${soldir}
fi

echo Processing in directory ${soldir}/ECAM

if [ ! -d 0${soldir}/ECAM ]; then
    mkdir -p ${soldir}/ECAM
fi

cd ${soldir}/ECAM 

m20_fetch_raw -c NAVCAM_LEFT -s $sol -S ${seqid}

calibrate -i *J0?.png 

diffgif -i *${seqid}*-rjcal.png -o DiffGif_${sol}_${seqid}.gif -v -b 0 -w 3.0 -g 1.0 -l 5 -d 20 -p stacked


