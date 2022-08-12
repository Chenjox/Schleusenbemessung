set datafile separator ','
set multiplot

stats 'result.csv' using 1:2 nooutput

q = STATS_max_y
qpos = STATS_index_max_y

stats 'result.csv' using 1:3 nooutput

k = STATS_max_y

set xrange[0:*]

set lmargin screen 0.3

#### Kammerwasserspiegel

set ytics 1.0
set yrange [0:k]
set ylabel 'Kammerwasserspiegel in [m]'

plot 'result.csv' using 1:3 with lines linecolor 1 notitle

####

set yrange [0:q+5]
set ytics offset -8, 0
set ylabel 'Durchfluss in [m^3/s]' offset -8, 0
plot 'result.csv' using 1:2 with lines linecolor 2 notitle

set arrow from qpos, graph(0,0) to qpos, graph(1,1) nohead

#### Plot 3

stats 'result.csv' using 5 nooutput

s = STATS_max

set yrange [0:s+0.1*s]
set ytics 0.01 offset -16, 0
set ylabel 'Wasserspiegelneigung in [-]' offset -16, 0
plot 'result.csv' using 1:5 with lines linecolor 3 notitle

#plot 'result.csv' using 1:2 with lines axes x1y1 title 'Durchfluss in [m^3/s]', \
#     'result.csv' using 1:3 with lines axes x1y2 title 'Kammerwasserspiegel in [m]'

pause -1
