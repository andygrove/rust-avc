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

//  int fourcc = static_cast<int>(inputVideo.get(CV_CAP_PROP_FOURCC)); 
 // cout << "fourcc = " << fourcc << endl;

  Size S = Size((int) inputVideo.get(CV_CAP_PROP_FRAME_WIDTH),    // Acquire input size
                  (int) inputVideo.get(CV_CAP_PROP_FRAME_HEIGHT));

  outputVideo.open(filename, VideoWriter::fourcc('M','P','4','V'), inputVideo.get(CV_CAP_PROP_FPS), S, true);
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

extern "C" int32_t video_drawtext(uint32_t x, uint32_t y, const char *s) {
  putText(frame, s, cvPoint(x,y), FONT_HERSHEY_COMPLEX_SMALL, 0.8, cvScalar(200,200,250), 1, CV_AA);
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
