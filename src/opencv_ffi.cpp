#include "opencv2/highgui.hpp"
#include "opencv2/imgproc.hpp"
#include <iostream>

using namespace std;
using namespace cv;

VideoCapture inputVideo;
VideoWriter outputVideo;

Mat frame;

extern "C" int32_t video_init(uint32_t camera, const char *filename) {
  if (!inputVideo.open(camera)) {
    return -1;
  }

// Acquire input size (640 x 480 with the Logitech C920)
  Size S = Size((int) inputVideo.get(CV_CAP_PROP_FRAME_WIDTH),
                  (int) inputVideo.get(CV_CAP_PROP_FRAME_HEIGHT));

  // start writing MP4V video file and report the speed as 24 FPS which should be close enough to make
  // playback happen in real-time based on experiments so far
  outputVideo.open(filename, VideoWriter::fourcc('M','P','4','V'), 24, S, true);
  if (!outputVideo.isOpened()) {
    cerr << "failed to open video output file" << endl;
    return -2;
  }

  return 0;
}

extern "C" int32_t video_capture() {
  inputVideo >> frame;
  if (frame.empty()) {
    return -1;
  }
  return 0;
}

extern "C" int32_t video_drawtext(uint32_t x, uint32_t y, const char *s, uint8_t r, uint8_t g, uint8_t b, uint8_t a) {
  putText(frame, s, cvPoint(x,y), FONT_HERSHEY_COMPLEX_SMALL, 0.6, cvScalar(b,g,r,a), 1, CV_AA);
  return 0;
}

extern "C" int32_t video_write() {
  outputVideo << frame;

  // this won't work for some reason when called from Rust
  //imshow( "opencv", frame );

  return 0;
}

extern "C" int32_t video_close() {
  outputVideo.release();
  return 0;
}
