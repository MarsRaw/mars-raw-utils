#!/bin/bash

if [ $# -ne 3 ]; then
    echo "usage: go <msl|m20> <sol> <instrument>"
    exit
fi

. ~/bin/go.sh $@

mission=${1^^}
sol=$2
instrument=${3^^}

if [ $mission == "MSL" ]; then

    if [ $instrument == "MCAM" ]; then

        mru msl-fetch -c MASTCAM -s $sol -n
        mru calibrate -i *jpg -P msl_mcam_drcx -D amaze

    elif [ $instrument == "NCAM" ]; then

        mru msl-fetch -c NAV_LEFT NAV_RIGHT -s $sol -n
        mru calibrate -i *JPG

    elif [ $instrument == "MAHLI" ]; then

        mru msl-fetch -c MAHLI -s $sol -n
        mru calibrate -i *jpg -P msl_mahli_drcx

    elif [ $instrument == "CCAM" ]; then

        mru msl-fetch -c CHEMCAM -s $sol -f PRC -n
        mru calibrate -i *PNG

    fi

elif [ $mission == "M20" ]; then

    if [ $instrument == "ZCAM" ]; then

        mru m20-fetch -c MASTCAM -s $sol -P ECM -n
        mru calibrate -i *J0?.png -P m20_zcam_drcx -D amaze

    elif [ $instrument == "NCAM" ]; then

        mru m20-fetch -c NAVCAM_LEFT NAVCAM_RIGHT -s $sol -n
        assemble_ncams.sh -ncam
        mru calibrate -i *assembled.tif -P m20_ncam_rad m20_ncam_mcz

    elif [ $instrument == "WATSON" ]; then

        mru m20-fetch -c WATSON -s $sol -P ECM -n
        mru calibrate -i *J0?.png -P  m20_watson_rad -D amaze

    elif [ $instrument == "SCAM" ]; then

        mru m20-fetch -c SUPERCAM -s $sol -P ECM -n
        mru calibrate -i *J0?.png -P  m20_scam_rad -D amaze

    fi

fi
