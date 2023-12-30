#!/bin/bash

# Deletes lower-versioned images from an M20 image directory, keeping the highest version of 
# each file. 

for f in `ls *-rjcal-*tif | cut -c 1-53`; do
    
    if [ `ls -1 $f*tif | wc -l` -gt 1 ]; then
        echo Removing low versions for $f

        for n in `ls -1 $f*-rjcal-*tif | head -n -1`; do
            rm $n
        done

        for n in `ls -1 $f*-rjcal-*json | head -n -1`; do
            rm $n
        done

    else
        echo Skipping $f
    fi

done