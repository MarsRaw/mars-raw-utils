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


msl_fetch_raw -c NAV_RIGHT -s $sol

calibrate -i *JPG -t 2.0

if [ ! -d RDR ]; then
    mkdir RDR
fi

mv *rjcal* RDR
cd RDR

if [ `ls *NCAM00593*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    diffgif -i NRB*NCAM00593*-rjcal.png -o DustDevil_${sol}_NCAM00593_rjcal.gif -v -b 0 -w 2.0 -g 2.5 -l 5 -d 20 -p stacked
fi

if [ `ls *${seqid}*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    diffgif -i *${seqid}*-rjcal.png -o DustDevil_${sol}_${seqid}_rjcal.gif -v -b 0 -w 2.0 -g 2.5 -l 5 -d 20 -p stacked
fi

if [ `ls *FHAZ00595*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    diffgif -i *FHAZ00595*-rjcal.png -o DustDevil_FHAZ_${sol}_FHAZ00595_rjcal.gif -v -b 0 -w 2.0 -g 2.5 -l 5 -d 20 -p stacked
fi

if [ `ls *RHAZ00595*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    diffgif -i *RHAZ00595*-rjcal.png -o DustDevil_RHAZ_${sol}_RHAZ00595_rjcal.gif -v -b 0 -w 2.0 -g 2.5 -l 5 -d 20 -p stacked
fi

if [ `ls *NCAM00556*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    diffgif -i *NCAM00556*-rjcal.png -o CloudShadow_${sol}.gif -v -b 0 -w 1.0 -g 2.5 -l 5 -d 20
fi

if [ `ls *NCAM00551*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    diffgif -i *NCAM00551*-rjcal.png -o ZenithMovie_${sol}.gif -v -b 0 -w 1.0 -g 2.5 -l 5 -d 20
fi

if [ `ls *NCAM00536*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    diffgif -i *NCAM00536*-rjcal.png -o ZenithMovie_${sol}.gif -v -b 0 -w 1.0 -g 2.5 -l 5 -d 20
fi

if [ `ls *NCAM00543*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    diffgif -i *NCAM00543*-rjcal.png -o ZenithMovie_EnvNorth_${sol}.gif -v -b 0 -w 1.0 -g 2.5 -l 5 -d 20
fi

if [ `ls *NCAM00567*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    diffgif -i *NCAM00567*-rjcal.png -o SupraHorizonMovie_${sol}.gif -v -b 0 -w 1.0 -g 2.5 -l 5 -d 20
fi

if [ `ls *NCAM00560*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    diffgif -i *NCAM00560*-rjcal.png -o SupraHorizonMovie_EnvNorth_${sol}.gif -v -b 0 -w 1.0 -g 2.5 -l 5 -d 20
fi

if [ `ls *NCAM00596*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    diffgif -i *_S*NCAM00596*rjcal.png -o DustDevil596_${sol}.gif -v -b 0 -w 5.0 -g 1.5 -l 5 -d 40
fi

if [ `ls *NCAM00597*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
    diffgif -i `ls *NCAM00597*-rjcal.png | head -n 16` -v -o SPENDI_NCAM00597_Set1_${sol}.gif  -b 0 -w 5.0 -g 1.5 -l 5 -d 40 -p stacked
    diffgif -i *_F*NCAM00597*rjcal.png -o SPENDI_NCAM00597_Set2_${sol}.gif -v -b 0 -w 3.0 -g 1.5 -l 5 -d 40
    diffgif -i `ls *NCAM00597*-rjcal.png | tail -n 23` -v -o SPENDI_NCAM00597_Set3_${sol}.gif  -b 0 -w 5.0 -g 1.5 -l 5 -d 40 -p stacked
fi

