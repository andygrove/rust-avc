# rust-avc

Source code for an autonomous vehicle based on the Raspberry Pi model 3 and the Rust programming language.

# Configuring the Raspberry Pi

## Installing the base operating system

- Download the Raspian Jessie image from https://www.raspberrypi.org/downloads/raspbian/ (I used the 2016-05-27 version)
- Follow the instructions for burning this onto an SD card: https://www.raspberrypi.org/documentation/installation/installing-images/README.md
- Insert the SD card into the Raspberry Pi and connect power. The Pi should boot up.
- Upgrade to the latest using the following commands

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

Next, download OpenCV 3.1.0 for Linux here: https://github.com/Itseez/opencv/archive/3.1.0.zip and then follow these instructions to build from source and install.

```
unzip ~/Downloads/opencv-3.1.0.zip
cd opencv-3.1.0/
wget -O opencv_contrib.zip https://github.com/Itseez/opencv_contrib/archive/3.0.0.z
cmake -G "Unix Makefiles"
make
sudo make install
```

## Install Rust

TBD

# OLDER NOTES


install raspicam lib

https://sourceforge.net/projects/raspicam/files/?

  111 unzip Downloads/raspicam-0.1.3.zip
  112 cd raspicam-0.1.3/
  113 mkdir build
  114 cd build
  115 cmake ..
make
sudo make install[ ]