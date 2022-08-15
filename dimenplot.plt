set datafile separator ','

set terminal wxt size 1300,600
set multiplot layout 2,5

set view 110,75
set style function lp

set xrange[0.3:0.6]
set yrange[1:2.0]
set zrange[400:3000]

set style line 11 lc rgb '#808080' lt 1


FilterValid(coll,was) = (column(coll) < 1260) ? column(coll) : NaN
FilterInvalid(coll) = column(coll) > 1200 ? column(coll) : NaN

do for [t=0:9] {

infile = sprintf('dimen%03.0f.csv',t*10)

set xtics
set ytics
set ztics

set title infile

# Ob alle Querschnitte geöffnet worden sind.
set cbrange[0:4]
set palette model RGB
set palette defined (0 "green", 1 "red")
unset colorbox
splot infile using 1:2:($3*0.0+450):5 with image notitle

set multiplot prev

# Höhenlage
set palette model RGB
set palette defined (0 "0x77ffffff", 1 "0x77000000")

set cbrange[*:*]
set colorbox

splot infile using 1:2:(FilterValid(3,4)):(column(4)) notitle palette z

#
#unset title
#unset xtics
#unset ytics
#unset ztics
#
#splot infile using 1:2:(FilterInvalid(3)) notitle linecolor 2


}

pause -1
