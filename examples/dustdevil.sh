#!/bin/bash


sol=$1
seqid=NCAM00595

if [ "x$2" != "x" ];then
    seqid=$2
fi

cd /data/MSL

if [ ! -d $sol/ECAM ]; then
    mkdir -p $sol/ECAM
fi

cd $sol/ECAM 

mru msl-fetch -c NAV_RIGHT NAV_LEFT  -s $sol -n

mru calibrate -i *JPG -t 2.0

if [ ! -d RDR ]; then
    mkdir RDR
fi

mv *rjcal* RDR
cd RDR



# NCAM00594, Dust Devil Survey 3x8 360deg, takes three pictures then rotates for eight pointings.
# We need to be careful that each differential is done on the three images from the same pointing. 
# To make sure we do that, we'll only run the differential if we get all 24 (3 * 8) images.
# If the sequence is run twice on the same sol (3 * 16 = 48), we'll allow them to all go into the same gif.
# This also needs ImageMagick to merge each pointing gif into a single gif. If that's not found we
# will level each part seperated which is more clutter but acceptable. 
num_parts_ncam00594=0
num_frames_ncam00594=`ls *NCAM00594*-rjcal.png 2> /dev/null | wc -l`
if [ $num_frames_ncam00594 -eq 48  ]; then
    num_parts_ncam00594=16
elif [ $num_frames_ncam00594 -eq 24 ]; then 
    num_parts_ncam00594=8
fi

if [ $num_parts_ncam00594 -gt 0 ]; then
    p=1
    n=3
    while [ $p -le $num_parts_ncam00594 ]; do
        mru -v diffgif -i `ls *NCAM00594*-rjcal.png | head -n $n | tail -n 3` -o DustDevil_${sol}_NCAM00594_part${p}_rjcal.gif -b 0 -w 2.0 -g 2.5 -l 5 -d 40 -p stacked
        let p=$p+1
        let n=$n+3
    done
    if [ -f /usr/bin/convert ]; then
        /usr/bin/convert DustDevil_${sol}_NCAM00594_part*_rjcal.gif  DustDevil_${sol}_NCAM00594_rjcal.gif 
        rm DustDevil_${sol}_NCAM00594_part*_rjcal.gif 
    fi
fi


if [ `ls *NCAM00593*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i NRB*NCAM00593*-rjcal.png -o DustDevil_${sol}_NCAM00593_rjcal.gif -b 0 -w 2.0 -g 2.5 -l 5 -d 20 -p stacked
fi

if [ `ls *${seqid}*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *${seqid}*-rjcal.png -o DustDevil_${sol}_${seqid}_rjcal.gif -b 0 -w 2.0 -g 2.5 -l 5 -d 20 -p stacked
fi

if [ `ls *FHAZ00595*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *FHAZ00595*-rjcal.png -o DustDevil_FHAZ_${sol}_FHAZ00595_rjcal.gif -b 0 -w 2.0 -g 2.5 -l 5 -d 20 -p stacked
fi

if [ `ls *RHAZ00595*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *RHAZ00595*-rjcal.png -o DustDevil_RHAZ_${sol}_RHAZ00595_rjcal.gif -b 0 -w 2.0 -g 2.5 -l 5 -d 20 -p stacked
fi

if [ `ls *NCAM00556*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00556*-rjcal.png -o CloudShadow_${sol}.gif -b 0 -w 1.0 -g 2.5 -l 5 -d 20
fi

if [ `ls *NCAM00551*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00551*-rjcal.png -o ZenithMovie_${sol}.gif -b 0 -w 1.0 -g 2.5 -l 5 -d 20
fi

if [ `ls *NCAM00536*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00536*-rjcal.png -o ZenithMovie_${sol}.gif -b 0 -w 1.0 -g 2.5 -l 5 -d 20
fi

if [ `ls *NCAM00543*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00543*-rjcal.png -o ZenithMovie_EnvNorth_${sol}.gif -b 0 -w 1.0 -g 2.5 -l 5 -d 20
fi

if [ `ls *NCAM00567*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00567*-rjcal.png -o SupraHorizonMovie_${sol}.gif -b 0 -w 1.0 -g 2.5 -l 5 -d 20
fi

if [ `ls *NCAM00560*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00560*-rjcal.png -o SupraHorizonMovie_EnvNorth_${sol}.gif -b 0 -w 1.0 -g 2.5 -l 5 -d 20
fi

if [ `ls *NCAM00596*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *_S*NCAM00596*rjcal.png -o DustDevil596_${sol}.gif -b 0 -w 5.0 -g 1.5 -l 5 -d 40
fi

if [ `ls *NCAM00597*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i `ls *NCAM00597*-rjcal.png | head -n 16` -o SPENDI_NCAM00597_Set1_${sol}.gif  -b 0 -w 5.0 -g 1.5 -l 5 -d 40 -p stacked
    mru -v diffgif -i *_F*NCAM00597*rjcal.png -o SPENDI_NCAM00597_Set2_${sol}.gif -b 0 -w 3.0 -g 1.5 -l 5 -d 40
    mru -v diffgif -i `ls *NCAM00597*-rjcal.png | tail -n 23` -o SPENDI_NCAM00597_Set3_${sol}.gif  -b 0 -w 5.0 -g 1.5 -l 5 -d 40 -p stacked
fi

