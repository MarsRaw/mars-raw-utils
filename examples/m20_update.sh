#!/bin/bash
# m20_update.sh - Retrieve latest Mars2020 raw images to local storage and run calibration

# Set this to the base directory of your M20 archive
DATAROOT=/data/M20

function update_on_sol() {
    sol=$1
    SOLROOT=`printf "%s/%04d" $DATAROOT $sol`
    mkdir -p $SOLROOT

    ##############
    # ECAM
    ##############
    echo Checking ECAM on Sol $sol...
    INSTROOT=$SOLROOT/ECAM
    mkdir -p $INSTROOT

    m20_fetch_raw -c NAVCAM HAZ_FRONT HAZ_REAR -s $sol -o $INSTROOT -n

    cd $INSTROOT
    if [ `ls *J0?.png 2> /dev/null | wc -l` -gt 0 ]; then
        m20_ecam_calibrate -i *J0?.png 
    fi

    if [ `ls *rjcal* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p RDR
        mv *rjcal* RDR
    fi

    ##############
    # WATSON
    ##############
    echo Checking Watson on Sol $sol...
    INSTROOT=$SOLROOT/WATSON
    mkdir -p $INSTROOT

    m20_fetch_raw -c WATSON -s $sol -o $INSTROOT -n -S EBY

    cd $INSTROOT

    if [ `ls *J0?.png 2> /dev/null | wc -l` -gt 0 ]; then
        m20_watson_calibrate -i *J0?.png -P m20_watson_bay m20_watson_ilt m20_watson_rad # m20_watson_cwb
    fi

    if [ `ls *rjcal-bay* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p BAY
        mv *rjcal-bay* BAY
    fi

    if [ `ls *rjcal-ilt* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p ILT
        mv *rjcal-ilt* ILT 
    fi 

    if [ `ls *rjcal-rad* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p RAD
        mv *rjcal-rad* RAD
    fi

    # if [ `ls *rjcal-cwb* 2> /dev/null | wc -l ` -gt 0 ]; then
    #     mkdir -p CWB
    #     mv *rjcal-cwb* CWB 
    # fi

    ##############
    # MastCam-Z
    ##############
    echo Checking MastCam-Z on Sol $sol...
    INSTROOT=$SOLROOT/ZCAM
    mkdir -p $INSTROOT

    m20_fetch_raw -c MASTCAM -s $sol -o $INSTROOT -n

    cd $INSTROOT

    if [ `ls *J0?.png 2> /dev/null | wc -l` -gt 0 ]; then
        m20_zcam_calibrate -i *J0?.png -P m20_zcam_bay m20_zcam_ilt m20_zcam_rad m20_zcam_cwb m20_zcam_cb2
    fi

    if [ `ls *rjcal-bay* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p BAY
        mv *rjcal-bay* BAY
    fi

    if [ `ls *rjcal-ilt* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p ILT
        mv *rjcal-ilt* ILT 
    fi 

    if [ `ls *rjcal-rad* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p RAD
        mv *rjcal-rad* RAD
    fi

    if [ `ls *rjcal-cwb* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p CWB
        mv *rjcal-cwb* CWB 
    fi

    if [ `ls *rjcal-cb2* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p CB2
        mv *rjcal-cb2* CB2 
    fi

    ##############
    # SuperCam
    ##############
    echo Checking SuperCam RMI on Sol $sol...
    INSTROOT=$SOLROOT/SCAM
    mkdir -p $INSTROOT

    m20_fetch_raw -c SUPERCAM -s $sol -o $INSTROOT -n -S EBY
    
    cd $INSTROOT
    if [ `ls *J0?.png 2> /dev/null | wc -l` -gt 0 ]; then
        m20_scam_calibrate -i *J0?.png 
    fi

    if [ `ls *rjcal* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p RDR
        mv *rjcal* RDR
    fi

    ##############
    # CacheCam
    ##############
    echo Checking CacheCam on Sol $sol...
    INSTROOT=$SOLROOT/CCAM
    mkdir -p $INSTROOT

    m20_fetch_raw -c CACHECAM -s $sol -o $INSTROOT -n 

    ##############
    # PIXL MCC
    ##############
    echo Checking PIXL MCC on Sol $sol...
    INSTROOT=$SOLROOT/PIXL
    mkdir -p $INSTROOT

    m20_fetch_raw -c PIXL -s $sol -o $INSTROOT -n 

    cd $INSTROOT
    if [ `ls *J0?.png 2> /dev/null | wc -l` -gt 0 ]; then
        m20_pixl_calibrate -i *J0?.png
    fi

    if [ `ls *rjcal* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p RDR
        mv *rjcal* RDR
    fi

    ##############
    # SkyCam
    ##############
    echo Checking SkyCam on Sol $sol...
    INSTROOT=$SOLROOT/SKYCAM
    mkdir -p $INSTROOT

    m20_fetch_raw -c SKYCAM -s $sol -o $INSTROOT -n 

    cd $INSTROOT
    if [ `ls *J0?.png 2> /dev/null | wc -l` -gt 0 ]; then
        m20_skycam_calibrate -i *J0?.png 
    fi

    if [ `ls *rjcal* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p RDR
        mv *rjcal* RDR
    fi

    ##############
    # Ingenuity Nav
    ##############
    echo Checking Ingenuity Navcam on Sol $sol...
    INSTROOT=$SOLROOT/HNAV
    mkdir -p $INSTROOT

    m20_fetch_raw -c HELI_NAV -s $sol -o $INSTROOT -n 

    cd $INSTROOT
    if [ `ls *J0?.png 2> /dev/null | wc -l` -gt 0 ]; then
        m20_hnav_calibrate -i *J0?.png
    fi

    if [ `ls *rjcal* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p RDR
        mv *rjcal* RDR
    fi

    ##############
    # Ingenuity RTE
    ##############
    echo Checking Ingenuity RTE on Sol $sol...
    INSTROOT=$SOLROOT/HRTE
    mkdir -p $INSTROOT

    m20_fetch_raw -c HELI_RTE -s $sol -o $INSTROOT -n 

    cd $INSTROOT
    if [ `ls *J0?.png 2> /dev/null | wc -l` -gt 0 ]; then
        m20_hrte_calibrate -i *J0?.png 
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
    for sol in `m20_latest -l`; do 
        update_on_sol $sol
    done
fi