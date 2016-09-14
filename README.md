# rust-avc

Source code for an autonomous vehicle based on the Raspberry Pi model 3 and the Rust programming language. This is the basis for my entry at the annual Sparkfun AVC competition in Boulder, CO and is always a work in progress.

I wrote this README primarily for myself so that I can quickly set up the software again in the event that my SD card gets damaged (quite a likely outcome if the vehicle crashes!).

# Electronics

The following electronics parts are used in my vehicle:

- 1 x Raspberry Pi 3
- 1 x Raspberry Pi 7" Display: TBD
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
burn the image onto a 16 GB (or larger) class 10 micro SD card. 

For me, the steps were as follows (on a Mac) but you will likely need to change the disk number. It's best to follow the instructions linked to above.

```
sudo dd bs=1m if=2016-05-27-raspbian-jessie.img of=/dev/rdisk5
sudo diskutil eject /dev/rdisk5
```

Insert the SD card into the Raspberry Pi and connect the power. Once the pi has booted up, open a terminal window and run the following commands:

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

Next, follow these instructions to build from source and install. This `make` step here takes a long time. I didn't time it but I would allow an hour or so for this step. 

```
wget https://github.com/Itseez/opencv/archive/3.1.0.zip
unzip 3.1.0.zip
cd opencv-3.1.0/
cmake -G "Unix Makefiles"
make
sudo make install
```

## Install Rust

```
curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly
```

# Operations

## Cross compiling

NOTE: I don't have this working yet.

```
rustup target add arm-unknown-linux-gnueabihf
cargo build --target=arm-unknown-linux-gnueabihf
```

## Connecting to the pi via ethernet

Use a regular ethernet cable to connect a laptop to the Pi (you'll need a USB-Ethernet adapter if you're using a laptop that doesn't have an ethernet port).

```ssh pi@raspberrypi.local```

The default password is 'raspberry'.

For testing video, this is required:

```
export DISPLAY=":0.0"
```


## Setting up udev rules

When connecting USB devices to the Pi they are assigned filenames such as /dev/ttyUSB0, /dev/ttyUSB1 and so on. After a reboot there is no guarantee that the names will be assigned in the same order so we need a way to assign our own names to each device e.g. /dev/ttyGPS and /dev/ttyCompass. We can use udev rules to accomplish this.

Connect the first device to the Pi and use `ls -l /dev/tty*` to find the device name (you can run this before and after connecting the device and play spot the difference).

Once you know the device name, in the case /dev/ttyUSB0, use `udevadm` to show the unique serial id of the device:

udevadm info --attribute-walk /dev/ttyUSB0 | grep -i serial

In my case, I created the file `/etc/udev/rules.d/gforce.rules` containing the following:

```
SUBSYSTEM=="tty", ATTRS{serial}=="A105BOB5", SYMLINK+="imu"
SUBSYSTEM=="tty", ATTRS{serial}=="AL00ERTT", SYMLINK+="gps"
SUBSYSTEM=="tty", ATTRS{serial}=="AI0483D0", SYMLINK+="qik"
```

I then used `sudo reboot` to reboot the Pi and then I was able to refer to the serial devices as `/dev/imu`, `/dev/gpu`, and `/dev/qik`.
