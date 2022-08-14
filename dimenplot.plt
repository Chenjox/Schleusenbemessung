set datafile separator ','

set terminal wxt size 1300,600
set multiplot layout 2,5

set view 110,75
set style function lp

set xrange[1:1.25]
set yrange[1:2]
set zrange[400:3000]


FilterValid(coll) = column(coll) < 1200 ? column(coll) : NaN
FilterInvalid(coll) = column(coll) > 1200 ? column(coll) : NaN

do for [t=0:9] {

infile = sprintf('dimen%03.0f.csv',t*10)

set xtics
set ytics
set ztics

set title infile

splot infile using 1:2:(FilterValid(3)) notitle linecolor 1

#set multiplot prev
#
#unset title
#unset xtics
#unset ytics
#unset ztics
#
#splot infile using 1:2:(FilterInvalid(3)) notitle linecolor 2


}

pause -1
