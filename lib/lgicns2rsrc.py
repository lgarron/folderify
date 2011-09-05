#!/usr/bin/python

# Using Python 2 because sideways compatibility might be painful and time-consuming to get right.

# 
import sys, time, string, binascii

# If no arguments were given, print a helpful message
if len(sys.argv)!=3:
    print 'Usage: python lgicns2rsrc.py <file_in.icns> <file_out.rsrc>'
    sys.exit(0)

    
file_in = open(sys.argv[1], 'r')
file_out = open(sys.argv[2], 'w')

header = "\x64\x61\x74\x61\x20\x27\x69\x63\x6E\x73\x27\x20\x28\x2D\x31\x36\x34\x35\x35\x29\x20\x7B"
footer = "\x0A\x7D\x3B\x0A\x0A"

# print header, "-", len(header)
# print footer, "-", len(footer)

print "Converting", sys.argv[1], "to a .rsrc file at", sys.argv[2]

file_out.write(header)

while True:

	dataBin = file_in.read(16);
	if len(dataBin) == 0:
		break

	dataHex = binascii.hexlify(dataBin).upper()

	for i in range((((len(dataHex) - 2)/4) * 4), 0, -4):
		dataHex = dataHex[0:i] + "\x20" + dataHex[i:]

	# I'm not sure what the proper format is, but it seems to work.
	lineStart = "\x0A\x09\x24\x22"
	lineMiddle = "\x22\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20"
	lineEnd = ""
	line = lineStart + dataHex + lineMiddle + lineEnd

	file_out.write(line)

	# print binascii.hexlify(line).upper()

file_out.write(footer)

file_in.close()
file_out.close()
