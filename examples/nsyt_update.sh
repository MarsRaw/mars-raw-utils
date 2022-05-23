#!/bin/bash
# msl_update.sh - Retrieve latest InSight raw images to local storage and run calibration

# Set this to the base directory of your InSight archive
DATAROOT=/data/NSYT


function update_on_sol() {
    sol=$1
    SOLROOT=`printf "%s/%04d" $DATAROOT $sol`
    mkdir -p $SOLROOT


    ##############
    # ICC
    ##############
    echo Checking ICC on Sol $sol...
    INSTROOT=$SOLROOT/ICC
    mkdir -p $INSTROOT

    mru nsyt-fetch -c ICC -s $sol -o $INSTROOT -n
    
    cd $INSTROOT
    if [ `ls *M_.JPG 2> /dev/null | wc -l` -gt 0 ]; then
        mru calibrate -i *M_.JPG
    fi

    if [ `ls *rjcal* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p RDR
        mv *rjcal* RDR
    fi

    ##############
    # IDC
    ##############
    echo Checking IDC on Sol $sol...
    INSTROOT=$SOLROOT/IDC
    mkdir -p $INSTROOT

    mru nsyt-fetch -c IDC -s $sol -o $INSTROOT -n
    
    cd $INSTROOT
    if [ `ls *M_.JPG 2> /dev/null | wc -l` -gt 0 ]; then
        mru calibrate -i *M_.JPG
    fi

    if [ `ls *rjcal* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p RDR
        mv *rjcal* RDR
    fi


}


# If the script was called with a sol number as the first parameter
# only update that sol in the archive, regardless if it has
# new images or not. Otherwise, ask the website which
# sols have new images and then cycle through those.
if [ "x$1" !=  "x" ]; then 
    update_on_sol $1
else
    for sol in `nsyt_latest -l`; do 
        update_on_sol $sol
    done
fi