#!/bin/bash
# msl_update.sh - Retrieve latest MSL raw images to local storage and run calibration

# Set this to the base directory of your MSL archive
DATAROOT=/data/MSL


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

    msl_fetch_raw -c NAV_RIGHT NAV_LEFT HAZ_FRONT HAZ_REAR -s $sol -o $INSTROOT -n
    
    cd $INSTROOT
    if [ `ls *JPG 2> /dev/null | wc -l` -gt 0 ]; then
        msl_ecam_calibrate -i *JPG 
    fi

    if [ `ls *rjcal* 2> /dev/null | wc -l ` -gt 0 ]; then
        mkdir -p RDR
        mv *rjcal* RDR

        # Generate ENV Dust Devil/Cloud movies if the images exist
        cd RDR
        if [ `ls *NCAM00593*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
            diffgif -i NRB*NCAM00593*-rjcal.png -o DustDevil_${sol}_NCAM00593_rjcal.gif -v -b 0 -w 2.0 -g 2.5 -l 5 -d 20 -p stacked
        fi

        if [ `ls *NCAM00595*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
            diffgif -i *NCAM00595*-rjcal.png -o DustDevil_${sol}_NCAM00595_rjcal.gif -v -b 0 -w 2.0 -g 2.5 -l 5 -d 20 -p stacked
        fi

        if [ `ls *FHAZ00595*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
            diffgif -i *FHAZ00595*-rjcal.png -o DustDevil_FHAZ_${sol}_FHAZ00595_rjcal.gif -v -b 0 -w 2.0 -g 2.5 -l 5 -d 20 -p stacked
        fi

        if [ `ls *RHAZ00595*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
            diffgif -i *RHAZ00595*-rjcal.png -o DustDevil_RHAZ_${sol}_RHAZ00595_rjcal.gif -v -b 0 -w 2.0 -g 2.5 -l 5 -d 20 -p stacked
        fi

        if [ `ls *NCAM00556*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
            diffgif -i *NCAM00556*-rjcal.png -o CloudShadow_${sol}_NCAM00556_rjcal.gif -v -b 0 -w 1.0 -g 2.5 -l 5 -d 20
        fi

        if [ `ls *NCAM00551*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
            diffgif -i *NCAM00551*-rjcal.png -o ZenithMovie_${sol}_NCAM00551_rjcal.gif -v -b 0 -w 1.0 -g 2.5 -l 5 -d 20
        fi

        if [ `ls *NCAM00536*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
            diffgif -i *NCAM00536*-rjcal.png -o ZenithMovie_${sol}_NCAM00536_rjcal.gif -v -b 0 -w 1.0 -g 2.5 -l 5 -d 20
        fi

        if [ `ls *NCAM00543*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
            diffgif -i *NCAM00543*-rjcal.png -o ZenithMovie_EnvNorth_${sol}_NCAM00543_rjcal.gif -v -b 0 -w 1.0 -g 2.5 -l 5 -d 20
        fi

        if [ `ls *NCAM00567*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
            diffgif -i *NCAM00567*-rjcal.png -o SupraHorizonMovie_${sol}_NCAM00567_rjcal.gif -v -b 0 -w 1.0 -g 2.5 -l 5 -d 20
        fi

        if [ `ls *NCAM00560*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
            diffgif -i *NCAM00560*-rjcal.png -o SupraHorizonMovie_EnvNorth_${sol}_NCAM00560_rjcal.gif -v -b 0 -w 1.0 -g 2.5 -l 5 -d 20
        fi

        if [ `ls *NCAM00596*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
            diffgif -i *_S*NCAM00596*rjcal.png -o DustDevil596_${sol}_NCAM00596_rjcal.gif -v -b 0 -w 5.0 -g 1.5 -l 5 -d 40
        fi

        if [ `ls *NCAM00597*-rjcal.png 2> /dev/null | wc -l` -gt 0 ]; then
            diffgif -i `ls *NCAM00597*-rjcal.png | head -n 16` -v -o SPENDI_NCAM00597_Set1_${sol}.gif  -b 0 -w 5.0 -g 1.5 -l 5 -d 40 -p stacked
            diffgif -i *_F*NCAM00597*rjcal.png -o SPENDI_NCAM00597_Set2_${sol}.gif -v -b 0 -w 3.0 -g 1.5 -l 5 -d 40
            diffgif -i `ls *NCAM00597*-rjcal.png | tail -n 23` -v -o SPENDI_NCAM00597_Set3_${sol}.gif  -b 0 -w 5.0 -g 1.5 -l 5 -d 40 -p stacked
        fi



    fi

    

    ##############
    # MAHLI
    ##############
    echo Checking MAHLI on Sol $sol...
    INSTROOT=$SOLROOT/MAHLI
    mkdir -p $INSTROOT

    msl_fetch_raw -c MAHLI -s $sol -o $INSTROOT -n

    cd $INSTROOT

    if [ `ls *jpg 2> /dev/null | wc -l` -gt 0 ]; then
        msl_mahli_calibrate -i *jpg -P msl_mahli_bay msl_mahli_ilt msl_mahli_rad msl_mahli_cwb
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

    ##############
    # MastCam
    ##############
    echo Checking MastCam on Sol $sol...
    INSTROOT=$SOLROOT/MCAM
    mkdir -p $INSTROOT

    msl_fetch_raw -c MASTCAM -s $sol -o $INSTROOT -n

    cd $INSTROOT

    if [ `ls *jpg 2> /dev/null | wc -l` -gt 0 ]; then
        msl_mcam_calibrate -i *jpg -P msl_mcam_bay msl_mcam_ilt msl_mcam_rad 
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


    ##############
    # ChemCam RMI
    ##############
    echo Checking ChemCam RMI on Sol $sol...
    INSTROOT=$SOLROOT/CCAM
    mkdir -p $INSTROOT

    msl_fetch_raw -c CHEMCAM -s $sol -o $INSTROOT -S PRC -n
    
    cd $INSTROOT
    if [ `ls *PNG 2> /dev/null | wc -l` -gt 0 ]; then
        msl_ccam_calibrate -i CR*L?.PNG 
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
    for sol in `msl_latest -l`; do 
        update_on_sol $sol
    done
fi