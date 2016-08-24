#!/bin/bash
gcc opencv_ffi.cpp -l opencv_core -l opencv_videoio -l opencv_highgui -l opencv_imgproc -l opencv_imgcodecs -shared -o libopencv_ffi.so

