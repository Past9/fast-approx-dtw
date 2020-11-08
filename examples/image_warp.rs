#![feature(min_const_generics)]

use fast_approx_dtw::{downsample_fns, loss_fns, DtwSolver};
use image::GenericImageView;
use std::thread;
use std::time::SystemTime;

const STACK_SIZE: usize = 128 * 1024 * 1024;
const IMG_HEIGHT: usize = 512;
const IMG_WIDTH: usize = 512;
const MAX_PATH_SIZE: usize = 1025;

fn main() {
  thread::Builder::new()
    .stack_size(STACK_SIZE)
    .spawn(run)
    .unwrap()
    .join()
    .unwrap();
}

fn run() {
  let left_img = load_image::<IMG_HEIGHT, IMG_WIDTH>("./examples/pentagon-left.gif");
  let right_img = load_image::<IMG_HEIGHT, IMG_WIDTH>("./examples/pentagon-right.gif");
  let mut warped_img = [[[0f32; 3]; IMG_WIDTH]; IMG_HEIGHT];
  let mut depth_img = [[0f32; IMG_WIDTH]; IMG_HEIGHT];

  let start = SystemTime::now();
  for y in 0..IMG_HEIGHT {
    let path = DtwSolver::<[f32; 3], IMG_WIDTH, MAX_PATH_SIZE>::new(
      &left_img[y],
      &right_img[y],
      downsample_fns::mean,
      loss_fns::euclidean::<3>,
    )
    .limit_downsamples(0)
    .solve();

    warped_img[y] = path.warp(left_img[y]);
    depth_img[y] = path.get_disparity();
  }
  println!(
    "Calculated paths in {:?} μs",
    SystemTime::now().duration_since(start).unwrap().as_nanos() as f32 / 1000.0
  );

  save_rgb_image::<IMG_HEIGHT, IMG_WIDTH>("./examples/output/pentagon-warped.bmp", warped_img);
  save_gray_image::<IMG_HEIGHT, IMG_WIDTH>("./examples/output/pentagon-depth.bmp", depth_img);
}

fn derive_signal<const N: usize>(sig: &[[f32; 3]; N]) -> [[f32; 3]; N] {
  let mut derivative = [[0f32; 3]; N];

  for t in 0..N - 1 {
    derivative[t][0] = sig[t + 1][0] - sig[t][0];
    derivative[t][1] = sig[t + 1][1] - sig[t][1];
    derivative[t][2] = sig[t + 1][2] - sig[t][2];
  }

  derivative
}

fn load_image<const H: usize, const W: usize>(filepath: &'static str) -> [[[f32; 3]; W]; H] {
  let mut img = [[[0f32; 3]; W]; H];
  let file_img = image::open(filepath).unwrap();

  println!(
    "Loaded image from {}, {}x{}",
    filepath,
    file_img.dimensions().0,
    file_img.dimensions().1
  );

  for y in 0..H {
    for x in 0..W {
      let pixel = file_img.get_pixel(x as u32, y as u32).0;
      img[y][x] = [
        pixel[0] as f32 / 255.0,
        pixel[1] as f32 / 255.0,
        pixel[2] as f32 / 255.0,
      ];
    }
  }

  img
}

fn save_gray_image<const H: usize, const W: usize>(filepath: &'static str, img: [[f32; W]; H]) {
  let mut rgb_img = [[[0f32; 3]; W]; H];

  let mut max = 0f32;
  for y in 0..H {
    for x in 0..W {
      if img[y][x] > max {
        max = img[y][x];
      }
    }
  }

  for y in 0..H {
    for x in 0..W {
      let pix = img[y][x] / max;
      rgb_img[y][x][0] = pix;
      rgb_img[y][x][1] = pix;
      rgb_img[y][x][2] = pix;
    }
  }

  save_rgb_image(filepath, rgb_img);
}

fn save_rgb_image<const H: usize, const W: usize>(filepath: &'static str, img: [[[f32; 3]; W]; H]) {
  let mut img_buf = image::ImageBuffer::new(W as u32, H as u32);

  for (img_x, img_y, pixel) in img_buf.enumerate_pixels_mut() {
    let x = img_x as usize;
    let y = img_y as usize;
    *pixel = image::Rgb([
      (img[y][x][0] * 255f32) as u8,
      (img[y][x][1] * 255f32) as u8,
      (img[y][x][2] * 255f32) as u8,
    ]);
  }

  img_buf.save(filepath).unwrap();

  println!("Saved image to {}", filepath);
}
