#!/bin/bash


sol=$1
seqid=NCAM00595
open_file_manager=0

export STUMP_LOG_AT_LEVEL=info

: ${MRUDATAHOME:=/data}

# Make sure we use the ImageMagick convert rather than the ISIS convert
MAGICK="`which magick`"

while [ $# -gt 0 ]; do
    if [ $1 == "-e" ]; then
        open_file_manager=1
    elif [ $1 == "-l" ]; then
        lights_only="-L"
    fi
    shift
done

cd $MRUDATAHOME/MSL

if [ ! -d $sol/ECAM ]; then
    mkdir -p $sol/ECAM
fi

cd $sol/ECAM 

mru msl-fetch -c NAV_RIGHT NAV_LEFT  -s $sol -n -f NCAM

mru calibrate -i *NCAM*JPG -t 2.0

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
num_frames_ncam00594=`ls *NCAM00594*-rjcal.tif 2> /dev/null | wc -l`
if [ $num_frames_ncam00594 -eq 48  ]; then
    num_parts_ncam00594=16
elif [ $num_frames_ncam00594 -eq 24 ]; then 
    num_parts_ncam00594=8
fi

if [ $num_parts_ncam00594 -gt 0 ]; then
    p=1
    n=3
    while [ $p -le $num_parts_ncam00594 ]; do
        mru -v diffgif -i `ls *NCAM00594*-rjcal.tif | head -n $n | tail -n 3` -o DustDevil_${sol}_NCAM00594_part${p}_rjcal.gif ${lights_only} -b 0 -w 2.0 -g 0.4 -l 5 -d 40 -m -p stacked
        let p=$p+1
        let n=$n+3
    done
    if [ -f $MAGICK ]; then
        $MAGICK DustDevil_${sol}_NCAM00594_part*_rjcal.gif  DustDevil_${sol}_NCAM00594_rjcal.gif
        rm DustDevil_${sol}_NCAM00594_part*_rjcal.gif 
    fi
fi

if [ `ls *NCAM00545*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i `ls *NCAM00545*tif | awk 'NR % 3 == 0'` -o TwilightCloudSearch_${sol}_NCAM00545_rjcal_1.gif ${lights_only} -b 0 -w 100.0 -g 0.5 -l 5 -d 20 -m
    mru -v diffgif -i `ls *NCAM00545*tif | awk 'NR % 3 == 1'` -o TwilightCloudSearch_${sol}_NCAM00545_rjcal_2.gif ${lights_only} -b 0 -w 100.0 -g 0.5 -l 5 -d 20 -m
    mru -v diffgif -i `ls *NCAM00545*tif | awk 'NR % 3 == 2'` -o TwilightCloudSearch_${sol}_NCAM00545_rjcal_3.gif ${lights_only} -b 0 -w 100.0 -g 0.5 -l 5 -d 20 -m

    if [ -f $MAGICK ]; then
        $MAGICK TwilightCloudSearch_${sol}_NCAM00545_rjcal_?.gif   TwilightCloudSearch_${sol}_NCAM00545_rjcal.gif
        rm TwilightCloudSearch_${sol}_NCAM00545_rjcal_?.gif 
    fi
fi

if [ `ls *NCAM00590*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    p=1
    n=12
    while [ $p -le 3 ]; do
        mru -v diffgif -i `ls *NCAM00590*-rjcal.tif | head -n $n | tail -n 12` -o DustDevil_${sol}_NCAM00590_part${p}_rjcal.gif ${lights_only} -b 0 -w 2.0 -g 0.4 -l 5 -d 40 -m -p stacked
        let p=$p+1
        let n=$n+12
    done
    if [ -f $MAGICK ]; then
        $MAGICK DustDevil_${sol}_NCAM00590_part*_rjcal.gif  DustDevil_${sol}_NCAM00590_rjcal.gif
        rm DustDevil_${sol}_NCAM00590_part*_rjcal.gif
    fi
fi

if [ `ls *NCAM00592*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    p=1
    n=8
    while [ $p -le 8 ]; do
        mru -v diffgif -i `ls *NCAM00592*-rjcal.tif | head -n $n | tail -n 8` -o DustDevil_${sol}_NCAM00592_part${p}_rjcal.gif ${lights_only} -b 0 -w 2.0 -g 0.4 -l 5 -d 40 -m -p stacked
        let p=$p+1
        let n=$n+8
    done
    if [ -f $MAGICK ]; then
        $MAGICK DustDevil_${sol}_NCAM00592_part*_rjcal.gif  DustDevil_${sol}_NCAM00592_rjcal.gif
        rm DustDevil_${sol}_NCAM00592_part*_rjcal.gif
    fi
fi

if [ `ls *NCAM00593*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i NRB*NCAM00593*-rjcal.tif -o DustDevil_${sol}_NCAM00593_rjcal.gif ${lights_only} -b 0 -w 2.0 -g 1.0 -l 5 -d 20 -m -p stacked
fi

if [ `ls *${seqid}*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *${seqid}*-rjcal.tif -o DustDevil_${sol}_${seqid}_rjcal.gif ${lights_only} -b 0 -w 2.0 -g 0.4 -l 5 -d 20 -m -p stacked
fi

if [ `ls *FHAZ00595*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *FHAZ00595*-rjcal.tif -o DustDevil_FHAZ_${sol}_FHAZ00595_rjcal.gif ${lights_only} -b 0 -w 2.0 -g 1.0 -l 5 -d 20 -m -p stacked
fi

if [ `ls *RHAZ00595*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *RHAZ00595*-rjcal.tif -o DustDevil_RHAZ_${sol}_RHAZ00595_rjcal.gif ${lights_only} -b 0 -w 2.0 -g 0.4 -l 5 -d 20 -m -p stacked
fi

if [ `ls *NCAM00556*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00556*-rjcal.tif -o CloudShadow_${sol}.gif ${lights_only} -b 0 -w 1.0 -g 0.4 -l 5 -d 20 -m
fi

if [ `ls *NCAM00551*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00551*-rjcal.tif -o ZenithMovie_${sol}.gif ${lights_only} -b 0 -w 1.0 -g 0.4 -l 5 -d 20 -m
fi

if [ `ls *NCAM00536*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00536*-rjcal.tif -o ZenithMovie_${sol}.gif ${lights_only} -b 0 -w 1.0 -g 0.4 -l 5 -d 20 -m
fi

if [ `ls *NCAM00543*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00543*-rjcal.tif -o ZenithMovie_EnvNorth_${sol}.gif ${lights_only} -b 0 -w 1.0 -g 0.4 -l 5 -d 20 -m
fi

if [ `ls *NCAM00547*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00547*-rjcal.tif -o TwilightCloudMovie_EnvSouth_${sol}.gif ${lights_only} -b 0 -w 100.0 -g 0.5 -l 5 -d 20 -m
fi

if [ `ls *NCAM00567*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00567*-rjcal.tif -o SupraHorizonMovie_${sol}.gif ${lights_only} -b 0 -w 1.0 -g 0.4 -l 5 -d 20 -m
fi

if [ `ls *NCAM00560*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *NCAM00560*-rjcal.tif -o SupraHorizonMovie_EnvNorth_${sol}.gif ${lights_only} -b 0 -w 1.0 -g 0.4 -l 5 -d 20 -m
fi

if [ `ls *NCAM00596*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i *_S*NCAM00596*rjcal.tif -o DustDevil596_${sol}.gif ${lights_only} -b 0 -w 5.0 -g 0.667 -l 5 -d 40 -m
fi

if [ `ls *NCAM00597*-rjcal.tif 2> /dev/null | wc -l` -gt 0 ]; then
    mru -v diffgif -i `ls *NCAM00597*-rjcal.tif | head -n 16` -o SPENDI_NCAM00597_Set1_${sol}.gif ${lights_only}  -b 0 -w 5.0 -g 0.667 -l 5 -d 40 -m -p stacked
    mru -v diffgif -i *_F*NCAM00597*rjcal.tif -o SPENDI_NCAM00597_Set2_${sol}.gif ${lights_only} -b 0 -w 3.0 -g 1.5 -l 5 -d 40 -m
    mru -v diffgif -i `ls *NCAM00597*-rjcal.tif | tail -n 23` -o SPENDI_NCAM00597_Set3_${sol}.gif ${lights_only}  -b 0 -w 5.0 -g 0.667 -l 5 -d 40 -m -p stacked
fi

if [ `ls *NCAM00599*-rjcal.tif 2> /dev/null | wc -l` -eq 48 ]; then
    mru -v diffgif -i `ls *NCAM00599*-rjcal.tif | head -n 8` -o SPENDI_NCAM00599_Set1_${sol}.gif ${lights_only}  -b 0 -w 5.0 -g 0.667 -l 5 -d 40 -m -p stacked
    mru -v diffgif -i `ls *NCAM00599*-rjcal.tif | head -n 16 | tail -n 8` -o SPENDI_NCAM00599_Set2_${sol}.gif ${lights_only}  -b 0 -w 5.0 -g 0.667 -l 5 -d 40 -m -p stacked
    mru -v diffgif -i `ls *NCAM00599*-rjcal.tif | head -n 24 | tail -n 8` -o SPENDI_NCAM00599_Set3_${sol}.gif ${lights_only}  -b 0 -w 5.0 -g 0.667 -l 5 -d 40 -m -p stacked
    mru -v diffgif -i `ls *NCAM00599*-rjcal.tif | head -n 32 | tail -n 8` -o SPENDI_NCAM00599_Set4_${sol}.gif ${lights_only}  -b 0 -w 5.0 -g 0.667 -l 5 -d 40 -m -p stacked
    mru -v diffgif -i `ls *NCAM00599*-rjcal.tif | head -n 40 | tail -n 8` -o SPENDI_NCAM00599_Set5_${sol}.gif ${lights_only}  -b 0 -w 5.0 -g 0.667 -l 5 -d 40 -m -p stacked
    mru -v diffgif -i `ls *NCAM00599*-rjcal.tif | tail -n 8` -o SPENDI_NCAM00599_Set6_${sol}.gif ${lights_only}  -b 0 -w 5.0 -g 0.667 -l 5 -d 40 -m -p stacked
fi

if [ `ls *NCAM00598*-rjcal.tif 2> /dev/null | wc -l` -eq 48 ]; then

    # Doug's awesome Twilight SPENDI:
    # 1 3x1 Mosaic
    # 2 3x movie
    # 3 3x movie
    # 4 3x movie
    # 5 3x1 mosaic
    # 6 3x movie
    # 7 3x movie
    # 8 3x movie

    # 9 3x1 mosaic
    # 10 3x1 mosaic
    # 11 3x1 mosaic
    # 12 3x1 mosaic
    # 13 3x1 mosaic
    # 14 3x1 mosaic
    # 15 3x1 mosaic
    # 16 3x1 mosaic

    export MARS_OUTPUT_FORMAT=png

    # There's a bug in either my MRU or the image or tiff crates (probably mine) that is causing
    # tiffs to glitch when cropped by ImageMagick (see crop operation below). Switching output
    # format to png seems to alleviate that.
    extension=png

    movie_black=0
    movie_white=100.0
    movie_gamma=0.7
    movie_lowpass=5
    movie_delay=40

    set_head=3
    set_tail=3

    set_no=1

    # Set 1, 3x1 Mosaic:
    mru composite -i `ls *NCAM00598*.tif | head -n ${set_head} | tail -n ${set_tail}` -o TwilightSpendi_NCAM00598_set`printf "%02d" $set_no`.${extension} -r 90
    let set_head=$set_head+3
    let set_no=$set_no+1

    for _ in `seq 2 4`; do
        # Set 2-4, 3x Movie:
        mru -v diffgif -i `ls *NCAM00598*.tif | head -n ${set_head} | tail -n ${set_tail}` -o TwilightSpendi_NCAM00598_set`printf "%02d" $set_no`.gif ${lights_only} -b ${movie_black} -w ${movie_white} -g ${movie_gamma} -l ${movie_lowpass} -d ${movie_delay} -m
        let set_head=$set_head+3
        let set_no=$set_no+1
    done


    # Set 5, 3x1 Mosaic:
    mru composite -i `ls *NCAM00598*.tif | head -n ${set_head} | tail -n ${set_tail}` -o TwilightSpendi_NCAM00598_set`printf "%02d" $set_no`.${extension}
    let set_head=$set_head+3
    let set_no=$set_no+1


    for _ in `seq 6 8`; do
        # Set 2-4, 3x Movie:
        mru -v diffgif -i `ls *NCAM00598*.tif | head -n ${set_head} | tail -n ${set_tail}` -o TwilightSpendi_NCAM00598_set`printf "%02d" $set_no`.gif ${lights_only} -b ${movie_black} -w ${movie_white} -g ${movie_gamma} -l ${movie_lowpass} -d ${movie_delay} -m
        let set_head=$set_head+3
        let set_no=$set_no+1
    done


    for _ in `seq 9 16`; do
        # Set 9-16, 3x1 Mosaic:
        out_image=TwilightSpendi_NCAM00598_set`printf "%02d" $set_no`.${extension} 
        mru composite -i `ls *NCAM00598*.tif | head -n ${set_head} | tail -n ${set_tail}` -o ${out_image} -r -90
        
        # Given slight error in RSM motion, the mosaics will have a slight different in pixel dimensions, so
        # we can center-crop to a smaller size to assure they are all the same when passed into diffgif
        magick ${out_image} -gravity center -crop 3815x1348+0+0 +repage tmp_image.${extension} 
        rm ${out_image}
        mv tmp_image.${extension} ${out_image}

        let set_head=$set_head+3
        let set_no=$set_no+1
    done

    mru diffgif -i TwilightSpendi_NCAM00598_set09.${extension} \
        TwilightSpendi_NCAM00598_set10.${extension} \
        TwilightSpendi_NCAM00598_set11.${extension} \
        TwilightSpendi_NCAM00598_set12.${extension} \
        TwilightSpendi_NCAM00598_set13.${extension} \
        TwilightSpendi_NCAM00598_set14.${extension} \
        TwilightSpendi_NCAM00598_set15.${extension} \
        TwilightSpendi_NCAM00598_set16.${extension} \
        -o TwilightSpendi_NCAM00598_set09-16.gif ${lights_only} -b ${movie_black} -w ${movie_white} -g ${movie_gamma} -l ${movie_lowpass} -d ${movie_delay} -m

    # Gonna leave behind the individual mosaics as they might be individually useful outside of the gif.
    
    unset MARS_OUTPUT_FORMAT
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


