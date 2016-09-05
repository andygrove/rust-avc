FROM debian:jessie

RUN apt-get update
RUN apt-get install -y build-essential
RUN apt-get install -y wget vim
RUN wget https://github.com/Itseez/opencv/archive/3.1.0.zip
RUN apt-get install -y zip
RUN unzip 3.1.0.zip
RUN apt-get install -y cmake
RUN cd opencv-3.1.0 ; cmake -G "Unix Makefiles" ; make ; make install


