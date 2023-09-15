#!/bin/bash

sol=$1
seqid=
open_file_manager=0

export MARS_LOG_AT_LEVEL=info

while [ $# -gt 0 ]; do
    if [ $1 == "-e" ]; then
        open_file_manager=1
    fi
    shift
done

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

mru m20-fetch -c NAVCAM_LEFT -s $sol -f NCAM00500 NCAM00502 NCAM0051 NCAM0052 NCAM0053 ${seqid} -n

mru calibrate -i *J0?.png

# Basic change detection processing using NCAM00502
if [ `ls *NCAM00502*J0?.png | wc -l` -gt 0 ]; then
    for id in `ls *NCAM00502*J0?.png | cut -d _ -f 3 | sort | uniq`; do
        out_filename=`ls NLR*$id*J0?.png | head -n 1 |  sed -e 's/_01_/_00_/' | sed -e 's/.png/-assembled.png/'`
        export MARS_OUTPUT_FORMAT=png
        mru -v m20-ecam-assemble -i NLR*$id*J0?.png -o $out_filename
        export MARS_OUTPUT_FORMAT=tif
        mru calibrate -i $out_filename
    done
    mru -v diffgif -i `ls *NCAM00502*-assembled-rjcal.tif | head -n 3` -o DiffGif_${sol}_NCAM00502_pt1.gif -b 0 -w 70.0 -g 1.0 -l 5 -d 20 -m -L
    mru -v diffgif -i `ls *NCAM00502*-assembled-rjcal.tif | tail -n 3` -o DiffGif_${sol}_NCAM00502_pt2.gif -b 0 -w 70.0 -g 1.0 -l 5 -d 20 -m -L
fi


for seqid in `ls *NCAM00514*.tif 2> /dev/null | cut -c 36-44 | sort | uniq`; do
    echo "Processing gif for ${seqid}"
    mru -v diffgif -i *${seqid}*-rjcal.tif -o DiffGif_${sol}_${seqid}.gif -b 0 -w 70.0 -g 1.0 -l 5 -d 20 -L
done

for seqid in `ls *NCAM00515*.tif 2> /dev/null | cut -c 36-44 | sort | uniq`; do
    echo "Processing gif for ${seqid}"
    mru -v diffgif -i *${seqid}*-rjcal.tif -o DiffGif_${sol}_${seqid}.gif -b 0 -w 70.0 -g 1.0 -l 5 -d 20  -L
done

for seqid in `ls *NCAM005{1,2,3}*2I*.tif 2> /dev/null | cut -c 36-44 | sort | uniq`; do
    echo "Processing gif for ${seqid}"
    mru -v diffgif -i *${seqid}*-rjcal.tif -o DiffGif_${sol}_${seqid}.gif -b 0 -w 70.0 -g 0.5 -l 5 -d 20 -p stacked -m -L
done


if [ `ls *NCAM00500*-rjcal.tif 2> /dev/null | wc -l` -eq 15 ]; then
    echo "Processing gif for NCAM00500"
    rm DiffGif_${sol}_NCAM00500.gif
    mru -v diffgif -i `ls *NCAM00500*-rjcal.tif | head -n 3` -o DustDevil_${sol}_NCAM00500_part1_rjcal.gif -b 0 -w 70.0 -g 0.5 -l 5 -d 40 -p stacked -m -L
    mru -v diffgif -i `ls *NCAM00500*-rjcal.tif | head -n 6 | tail -n 3` -o DustDevil_${sol}_NCAM00500_part2_rjcal.gif -b 0 -w 70.0 -g 0.5 -l 5 -d 40 -p stacked -m -L
    mru -v diffgif -i `ls *NCAM00500*-rjcal.tif | head -n 9 | tail -n 3` -o DustDevil_${sol}_NCAM00500_part3_rjcal.gif -b 0 -w 70.0 -g 0.5 -l 5 -d 40 -p stacked -m -L
    mru -v diffgif -i `ls *NCAM00500*-rjcal.tif | head -n 12 | tail -n 3` -o DustDevil_${sol}_NCAM00500_part4_rjcal.gif -b 0 -w 70.0 -g 0.5 -l 5 -d 40 -p stacked -m -L
    mru -v diffgif -i `ls *NCAM00500*-rjcal.tif | head -n 15 | tail -n 3` -o DustDevil_${sol}_NCAM00500_part5_rjcal.gif -b 0 -w 70.0 -g 0.5 -l 5 -d 40 -p stacked -m -L
    /usr/bin/convert DustDevil_${sol}_NCAM00500_part1_rjcal.gif  DustDevil_${sol}_NCAM00500_part2_rjcal.gif DustDevil_${sol}_NCAM00500_part3_rjcal.gif \
        DustDevil_${sol}_NCAM00500_part4_rjcal.gif DustDevil_${sol}_NCAM00500_part5_rjcal.gif DustDevil_${sol}_NCAM00500_rjcal.gif 
    rm DustDevil_${sol}_NCAM00500_part*_rjcal.gif 
fi


if [ $open_file_manager -eq 1 ]; then 
    if [ `which xdg-open | wc -l` -eq 1 ]; then          # Linux generic
        xdg-open . 2> /dev/null &                        
    elif [ `which dolphin | wc -l` -eq 1 ]; then         # KDE on Linux
        dolphin --new-window . 2> /dev/null &
    elif [ `which explorer.exe | wc -l` -eq 1 ]; then    # Windows Subsystem for Linux
        explorer.exe .
    elif [ `which open | wc -l` -eq 1 ]; then            # macOS
        open . &
    fi 
fi
