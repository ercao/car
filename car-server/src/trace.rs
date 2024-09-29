use car_utils::command::Navigate;
use opencv::core::{self, Mat, MatExprTraitConst, MatTraitConst, Point, Scalar, Vector};
use opencv::imgproc;

pub fn follow_line(frame: &mut Mat) -> Navigate {
  // 将输入帧转换为灰度图像
  let mut gray = Mat::default();
  imgproc::cvt_color(frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0).unwrap();

  // 对灰度图像进行二值化处理
  let mut thresh = Mat::default();
  imgproc::threshold(&gray, &mut thresh, 0.0, 255.0, imgproc::THRESH_BINARY_INV + imgproc::THRESH_OTSU).unwrap();

  // 创建遮掩层
  let mut mask = Mat::zeros(thresh.rows(), thresh.cols(), thresh.typ()).unwrap().to_mat().unwrap();
  let (height, width) = (frame.rows(), frame.cols());
  let roi_vertices = Vector::from_slice(&[
    Point::new(0, height),
    Point::new((width as f64 * 0.45) as i32, (height as f64 * 0.6) as i32),
    Point::new((width as f64 * 0.55) as i32, (height as f64 * 0.6) as i32),
    Point::new(width, height),
  ]);

  imgproc::fill_poly(&mut mask, &roi_vertices, Scalar::all(255.0), imgproc::LINE_8, 0, core::Point::new(0, 0)).unwrap();

  // 位运算，只保留感兴趣区域
  let mut masked_image = Mat::default();
  core::bitwise_and(&thresh, &mask, &mut masked_image, &core::no_array()).unwrap();

  // 查找轮廓
  let mut contours = Vector::<Mat>::new();
  imgproc::find_contours(
    &masked_image,
    &mut contours,
    imgproc::RETR_EXTERNAL,
    imgproc::CHAIN_APPROX_SIMPLE,
    Point::new(0, 0),
  )
  .unwrap();

  if !contours.is_empty() {
    // 选出面积最大的轮廓
    let max_contour = contours.iter().max_by_key(|c| imgproc::contour_area(c, false).unwrap() as i32).unwrap();
    let moments = imgproc::moments(&max_contour, false).unwrap();

    if moments.m00 > 0.0 {
      let cx = (moments.m10 / moments.m00) as i32;
      let cy = (moments.m01 / moments.m00) as i32;

      // 去掉显示质心的代码
      imgproc::circle(frame, Point::new(cx, cy), 5, Scalar::new(0.0, 0.0, 255.0, 0.0), -1, imgproc::LINE_8, 0).unwrap();
      let center = width as f64 / 2.0;
      let deviation_threshold = width as f64 * 0.1;

      if (cx as f64 - center).abs() <= deviation_threshold {
        return Navigate::Forward;
      } else if cx < center as i32 {
        return Navigate::Right;
      } else {
        return Navigate::Left;
      }
    }
  }

  // 没识别到
  Navigate::BackWard
}
