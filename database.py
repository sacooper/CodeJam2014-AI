from Image import open as openImage
import os
import sys
import csv

def makeCSV(fname):
    img = openImage(fname)
    (width, height) = img.size

    pixels = img.load()
    output = ''
    for x in range(0, width):
        for y in range(0, height):
            output += str(pixels[x, y]) + ' '

    with open((os.path.splitext(fname)[0] + ('.csv')), 'a') as file:
        file.write(output + 'n')

if (len(sys.argv) < 2):
    print 'enter a directory'
    sys.exit()

for root, dirs, files in os.walk(sys.argv[1]):
    for fname in files:
        if fname.endswith('.gif'):
            makeCSV(os.path.join(root, fname))
