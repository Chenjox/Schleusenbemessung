set datafile separator ','
set terminal postscript eps
set term epslatex size 5,3
set output 'inter.eps'
set multiplot

stats 'result.csv' using 2:4 nooutput

q = STATS_max_y
qpos = STATS_index_max_y

stats 'result.csv' using 1:3 nooutput

k = STATS_max_y
maxt = STATS_max_x

set xrange[0:maxt/60.0]

set lmargin screen 0.3

#### Kammerwasserspiegel

set ytics 1.0
set yrange [0:k]
set ylabel 'Kammerwasserspiegel in [m]'

plot 'result.csv' using ($2/60.0):3 with lines linecolor 1 notitle

####

set yrange [0:q+5]
set ytics offset -8, 0
set ylabel 'Durchfluss in [m^3/s]' offset -8, 0
plot 'result.csv' using ($2/60.0):4 with lines linecolor 2 notitle

unset xtics

unset ytics
unset ylabel
unset border
set yrange [0:q+5]

filtervalues(filtercol,str,col) = (stringcolumn(filtercol) eq str) ? column(col) : NaN

plot 'events.csv' using (filtervalues(2,"SG",1)/60.0):(q+5) with impulses linecolor "orange" notitle

plot 'events.csv' using (filtervalues(2,"VG",1)/60.0):(q+5) with impulses linecolor "brown" notitle

plot 'events.csv' using (filtervalues(2,"SU",1)/60.0):(q+5) with impulses linecolor "blue" notitle

plot 'events.csv' using (filtervalues(2,"VU",1)/60.0):(q+5) with impulses linecolor "navy" notitle


pause -1
