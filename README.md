# rust-avc

Source code for an autonomous vehicle based on the Raspberry Pi model 3 and the Rust programming language. This is the basis for my entry at the annual Sparkfun AVC competition in Boulder, CO and is always a work in progress.

I wrote this README primarily for myself so that I can quickly set up the software again in the event that my SD card gets damaged (quite a likely outcome if the vehicle crashes).

# Electronics

The following electronics parts are used in my vehicle:

- 1 x Raspberry Pi 3
- 1 x Touchscreen Display: TBD
- 1 x Logitech C920 Webcam: https://www.amazon.com/gp/product/B006JH8T3S
- 1 x GPS: https://www.sparkfun.com/products/8975
- 1 x IMU: https://www.sparkfun.com/products/10736
- 1 x Motor controller: https://www.pololu.com/product/1112
- 4 x Brushed DC Motors: https://www.pololu.com/product/1572
- 4 x USB-Serial Adapter: https://www.sparkfun.com/products/9873
- 5 x HC-SR04 Ultrasonic Sensors: TBD
- 1 x Octasonic: TBD

The total cost of the vehicle is somewhere around $600.

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

Next, follow these instructions to build from source and install. This part takes a LONG time.

```
wget https://github.com/Itseez/opencv/archive/3.1.0.zip
unzip 3.1.0.zip
cd opencv-3.1.0/
cmake -G "Unix Makefiles"
make
sudo make install
```

## Install Rust

Follow instructions at https://rustup.rs/ to install rustup and then:

```
rust install nightly
```

