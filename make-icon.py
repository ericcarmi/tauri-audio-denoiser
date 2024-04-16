from pylab import imshow, show, imsave
import numpy as np
import sys

try:
    dv = float(sys.argv[1])
except:
    dv = 0.994

H = 1024
W = H

img = np.zeros((H, W))

l = .98
h = .99

v = 1
for i in range(W):

    signal = np.sin(2*np.pi*i*5/W)**2
    noise = np.random.uniform(0, 1, H)
    # x = (signal*(1-v) + noise*v)
    # img[:, i] = x
    # v*=dv
    if i < 2*W // 5:
        img[:, i] = noise + signal/20
    elif i < 4*W // 5:
        x = (signal*(1-v) + noise*v)
        v *= dv
        img[:, i] = x
    else:
        x = (signal*(1-v) + noise*v)
        img[:, i] = x

img = img / max(abs(img.flatten())) * 255
imsave("denoiser.png", img, cmap='gray')
