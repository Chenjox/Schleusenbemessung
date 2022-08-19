set datafile separator ','
#set terminal postscript eps
#set term epslatex size 5,3
#set output 'inter.eps'
set terminal wxt size 1800,600
set palette defined (0 "blue", 1 "red")

set grid xtics ytics

set xrange[2.0:2.5]
set yrange[0.25:0.35]
set cbrange[0.0005:0.005]

set ytics 0.02
set mytics 5
set xtics 0.1
set mxtics 5
set cbtics 0.0005
set xlabel "B in [m]"
set ylabel "H in [m]"

set multiplot layout 1,3
#set view map
#set dgrid3d 50,50 box 0.015,0.005
#set style data lines
#set pm3d interpolate 1,1

#splot 'inter.csv' using 1:2:($3 == NaN ? 0.0 : $3) with pm3d
plot 'inter_max.csv' using ($3 == NaN ? 0.0 : $1):2:3 with points pointtype 20 pointsize 1.8 palette z notitle

plot 'inter_min.csv' using ($3 == NaN ? 0.0 : $1):2:3 with points pointtype 20 pointsize 1.8 palette z notitle

set zrange[0.0005:0.005]

set view 75,200

splot 'inter_max.csv' using ($3 == NaN ? 0.0 : $1):2:3 with points pointtype 1 pointsize 1.0 palette z notitle, \
      'inter_min.csv' using ($3 == NaN ? 0.0 : $1):2:3 with points pointtype 1 pointsize 1.0 palette z notitle


pause -1
