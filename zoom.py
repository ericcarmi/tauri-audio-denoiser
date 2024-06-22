# friggin zoom
'''
    pixel space is always 0..num pixels - 1
    t1...t2 changes
    
'''

from pylab import *


# resolution / number of pixels wide
N = 950
# x1, x2 are in units of num_time_samples? num of samples, not pixels
# but the spacing depends on number of pixels on screen
x1 = 0
x2 = 100000
y = linspace(x1,x2,N)
dz = 0.25
# changing zoom level changes the bounds

# also need to account for hover position (between 0 and 1)
# and the velocity multiplier and scroll/zoom direction
x1 = x1 + dz
x2 = x2 - dz

y2 = linspace(x1,x2,N)

def zoom(direction, start, end, scroll):
    # the previous values get used, inputs appear at output
    # start and end change first based on zoom ammount
    x1 = start + dz * direction
    x2 = end - dz * direction
    # scroll amount depends on zoom level
    # need to set position of scroll
    # min scroll movement is 1 pixel
    # 1 pixel moved on scrollbar is how many samples?
