set datafile separator ','
#set terminal postscript
#set output '| ps2pdf - output.pdf'
set palette defined (0 "blue", 1 "red")

set grid xtics ytics

set xrange[1.0:2.3]
set yrange[0.3:0.6]
set cbrange[0.0005:0.005]

set ytics 0.02
set xtics 0.1
set xlabel "B in [m]"
set ylabel "H in [m]"

plot 'min.csv' using 1:2:3 palette z

pause -1
