#!/bin/bash


if [ $# -ne 3 ]; then
	echo "usage: go <msl|m20> <sol> <instrument>"
else


	if [ $1 == "msl" ]; then
		cd ~/data/MSL
	elif [ $1 == "m20" ]; then 
		cd ~/data/M20
	fi

	soldir=`printf "%04i" $2`
	instrument=${3^^}

	mkdir -p $soldir/$instrument
	cd $soldir/$instrument
fi
