#!/bin/bash

sol=$1
seqid=
open_file_manager=0

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

mru m20-fetch -c NAVCAM_LEFT -s $sol -f NCAM00500 NCAM0051 NCAM0052 NCAM0053 ${seqid} -n

mru calibrate -i *J0?.png 


for seqid in `ls *NCAM005{1,2,3}*2I*.png 2> /dev/null | cut -c 36-44 | sort | uniq`; do
    mru -v diffgif -i *${seqid}*2I*-rjcal.png -o DiffGif_${sol}_${seqid}.gif -b 0 -w 3.0 -g 1.0 -l 5 -d 20 -p stacked
done


if [ `ls *NCAM00500*-rjcal.png 2> /dev/null | wc -l` -eq 15 ]; then
    rm DiffGif_${sol}_NCAM00500.gif
    mru -v diffgif -i `ls *NCAM00500*-rjcal.png | head -n 3` -o DustDevil_${sol}_NCAM00500_part1_rjcal.gif -b 0 -w 2.0 -g 2.5 -l 5 -d 40 -p stacked
    mru -v diffgif -i `ls *NCAM00500*-rjcal.png | head -n 6 | tail -n 3` -o DustDevil_${sol}_NCAM00500_part2_rjcal.gif -b 0 -w 2.0 -g 2.5 -l 5 -d 40 -p stacked
    mru -v diffgif -i `ls *NCAM00500*-rjcal.png | head -n 9 | tail -n 3` -o DustDevil_${sol}_NCAM00500_part3_rjcal.gif -b 0 -w 2.0 -g 2.5 -l 5 -d 40 -p stacked
    mru -v diffgif -i `ls *NCAM00500*-rjcal.png | head -n 12 | tail -n 3` -o DustDevil_${sol}_NCAM00500_part4_rjcal.gif -b 0 -w 2.0 -g 2.5 -l 5 -d 40 -p stacked
    mru -v diffgif -i `ls *NCAM00500*-rjcal.png | head -n 15 | tail -n 3` -o DustDevil_${sol}_NCAM00500_part5_rjcal.gif -b 0 -w 2.0 -g 2.5 -l 5 -d 40 -p stacked
    /usr/bin/convert DustDevil_${sol}_NCAM00500_part1_rjcal.gif  DustDevil_${sol}_NCAM00500_part2_rjcal.gif DustDevil_${sol}_NCAM00500_part3_rjcal.gif \
        DustDevil_${sol}_NCAM00500_part4_rjcal.gif DustDevil_${sol}_NCAM00500_part5_rjcal.gif DustDevil_${sol}_NCAM00500_rjcal.gif 
    rm DustDevil_${sol}_NCAM00500_part*_rjcal.gif 
fi


if [ $open_file_manager -eq 1 ]; then 
    if [ `which dolphin | wc -l` -eq 1 ]; then         # KDE on Linux
        dolphin --new-window . &
    elif [ `which explorer.exe | wc -l` -eq 1 ]; then   # Windows Subsystem for Linux
        explorer.exe .
    elif [ `which open | wc -l` -eq 1 ]; then          # macOS
        open . &
    fi # Nautilus (GNOME), whatever Xfce uses, etc.
fi
