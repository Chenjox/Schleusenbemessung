set datafile separator ','
set multiplot

stats 'result.csv' using 2:4 nooutput

q = STATS_max_y
qpos = STATS_index_max_y

stats 'result.csv' using 1:3 nooutput

k = STATS_max_y
maxt = STATS_max_x

set xrange[0:maxt]

set lmargin screen 0.3

#### Kammerwasserspiegel

set ytics 1.0
set yrange [0:k]
set ylabel 'Kammerwasserspiegel in [m]'

plot 'result.csv' using 2:3 with lines linecolor 1 notitle

####

set yrange [0:q+5]
set ytics offset -8, 0
set ylabel 'Durchfluss in [m^3/s]' offset -8, 0
plot 'result.csv' using 2:4 with lines linecolor 2 notitle

#set arrow from qpos, graph(0,0) to qpos, graph(1,1) nohead

#### Plot 3

stats 'result.csv' using 5 nooutput

so = STATS_max
su = STATS_min

set yrange [-0.2:0.2]
set ytics 0.01 offset -16, 0
set ylabel 'Durchflussänderung in [m^3/s^2]' offset -16, 0
plot 'result.csv' using 1:5 with lines linecolor 3 notitle

#plot 'result.csv' using 1:2 with lines axes x1y1 title 'Durchfluss in [m^3/s]', \
#     'result.csv' using 1:3 with lines axes x1y2 title 'Kammerwasserspiegel in [m]'


unset xtics

unset ytics
unset ylabel
unset border
set yrange [0:q+5]

filtervalues(filtercol,str,col) = (stringcolumn(filtercol) eq str) ? column(col) : NaN

plot 'events.csv' using (filtervalues(2,"SG",1)):(q+5) with impulses linecolor "orange" notitle

plot 'events.csv' using (filtervalues(2,"VG",1)):(q+5) with impulses linecolor "brown" notitle

plot 'events.csv' using (filtervalues(2,"SU",1)):(q+5) with impulses linecolor "blue" notitle

plot 'events.csv' using (filtervalues(2,"VU",1)):(q+5) with impulses linecolor "navy" notitle


pause -1
