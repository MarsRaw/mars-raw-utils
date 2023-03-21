#!/bin/bash
# A script to search a directory for Mars2020 NavCam image subframes 
# and assemble them into full frame images. Groups images using the 
# filename by left or right eye and by SCLK. Attempts to skip over
# mono and TRAV images.

ONLY_SCLK=
CLEANUP=1

CAM=NCAM
CAM1L=N
while [ $# -gt 0 ]; do
    if [ $1 == "-ncam" ]; then  # Default
        CAM=NCAM
        CAM1L=N
    elif [ $1 == "-fhaz" ]; then
        CAM=FHAZ
        CAM1L=F
    elif [ $1 == "-rhaz" ]; then
        CAM=RHAZ
        CAM1L=R
    elif [ $1 == "-sclk" -a "x$2" != "x" ]; then
        ONLY_SCLK=$2
        shift
    fi
    shift
done

function composite_sclk() {
    sclk=$1
    for eye in L R; do
        prefix=${CAM1L}${eye}F
        num_parts=`ls ${prefix}*${sclk}*J0?.png 2> /dev/null | wc -l`
        if [ $num_parts -ge 2 ]; then
            output_image=`ls ${prefix}*${sclk}*J0?.png | head -n 1 | sed -e 's/_01_/_00_/' | sed -e 's/.png/-assembled.png/'`
            mru -v m20-ecam-assemble -i ${prefix}*${sclk}*J0?.png -o $output_image
        fi
    done
}

if [ "x$ONLY_SCLK" == "x" ];then 
    for sclk in `ls ${CAM1L}{R,L}F_*${CAM}*J0?.png | cut -c 10-19 | sort | uniq`; do
        composite_sclk $sclk
    done
else 
    composite_sclk $ONLY_SCLK
fi