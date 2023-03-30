#!/bin/bash


# JPEG'ed LOCO debayer and smoothing prefilter.
# Stolen from UMSF: http://www.unmannedspaceflight.com/index.php?showtopic=7418&hl=debayer&st=30
# Note: This results in a noticable reduction in sharpness and color fidelity. 

echo Prefiltering $1
gmic $1  -bayer2rgb 3,3,3 -pde_flow 5,20 -sharpen 2 -o $1