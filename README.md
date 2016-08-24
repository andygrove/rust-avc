# rust-avc

Source code for an autonomous vehicle based on the Raspberry Pi model 3 and the Rust programming language. This is the basis for my entry at the annual Sparkfun AVC competition in Boulder, CO and is always a work in progress.

# Configuring the Raspberry Pi

## Installing the base operating system

Download the Raspian Jessie image from https://www.raspberrypi.org/downloads/raspbian/ (I used the 2016-05-27 version) and follow the instructions on that page to 
burn the image onto a 16 GB (or larger) class 10 micro SD card. Insert the SD card into the Raspberry Pi and connect the power. Once the pi has booted up, open a terminal window and run the following commands:

```
sudo apt-get update
sudo apt-get upgrade
sudo rpi-update
sudo reboot
```

## Install Git

```
sudo apt-get install git
```

## Install OpenCV

First, install the following dependencies:

```
sudo apt-get install build-essential cmake pkg-config
sudo apt-get install libjpeg-dev libtiff5-dev libjasper-dev libpng12-dev
sudo apt-get install libavcodec-dev libavformat-dev libswscale-dev libv4l-dev libxvidcore-dev libx264-dev libgtk2.0-dev libatlas-base-dev gfortran python2.7-dev python3-dev
```

 and then follow these instructions to build from source and install.

```
wget https://github.com/Itseez/opencv/archive/3.1.0.zip
unzip 3.1.0.zip
cd opencv-3.1.0/
cmake -G "Unix Makefiles"
make
sudo make install
```

## Install Rust

Follow instructions at https://rustup.rs/

# OLDER NOTES


Optional - install raspicam lib

https://sourceforge.net/projects/raspicam/files/?

  111 unzip Downloads/raspicam-0.1.3.zip
  112 cd raspicam-0.1.3/
  113 mkdir build
  114 cd build
  115 cmake ..
make
sudo make install[ ]