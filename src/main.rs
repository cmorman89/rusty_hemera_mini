// extern crate image;
// extern crate ndarray;

use image::{DynamicImage, Rgba, RgbaImage, GenericImageView, ImageDecoder, AnimationDecoder, Frame, ImageBuffer};
use ndarray::{Array3, Array2, s};
use std::thread;
use std::time;
use std::collections::VecDeque;
use std::fs::File;
use image::codecs::gif::{GifDecoder, GifEncoder};
use std::io::BufReader;

fn mirror_row(row: &[u8]) -> impl Iterator<Item = &u8> {
    row.iter().chain(row.iter().rev())
}

fn generate_rgb_escape(r: u8, g: u8, b: u8, is_foreground: bool) -> String {
    if is_foreground {
        format!("\x1b[38;2;{};{};{}m", r, g, b)
    } else {
        format!("\x1b[48;2;{};{};{}m", r, g, b)
    }
}

fn frame_sleep(duration_ms: u64) {
    thread::sleep(time::Duration::from_millis(duration_ms));
}

fn image_to_rgb_array(image_path: &str) -> Array3<u8> {
    // Open the image file
    let img = image::open(image_path).expect("Failed to open image");
    
    // Convert the image to RGBA (even if it's RGB, it will be handled as RGBA)
    let (width, height) = img.dimensions();
    let img_rgba = img.to_rgba8();

    // Create a 3D ndarray for the RGB values
    let mut rgb_array: Array3<u8> = Array3::zeros((height as usize, width as usize, 3));

    for (x, y, pixel) in img_rgba.enumerate_pixels() {
        let (r, g, b, _): (u8, u8, u8, u8) = pixel.0.into(); // We ignore the alpha channel here
        rgb_array[[y as usize, x as usize, 0]] = r;
        rgb_array[[y as usize, x as usize, 1]] = g;
        rgb_array[[y as usize, x as usize, 2]] = b;
    }

    rgb_array
}

fn print_rgb_array(rgb_array: &Array3<u8>) {
    let (h, w, _) = rgb_array.dim();
    for y in 0..h {
        let mut row_buffer: Vec<String> = Vec::new();
        if y % 2 == 0 && y < h - 2 {
            for x in 0..w {
                let fg_r: u8 = rgb_array[[y, x, 0]];
                let fg_g: u8 = rgb_array[[y, x, 1]];
                let fg_b: u8 = rgb_array[[y, x, 2]];
                let bg_r: u8 = rgb_array[[y + 1, x, 0]];
                let bg_g: u8 = rgb_array[[y + 1, x, 1]];
                let bg_b: u8 = rgb_array[[y + 1, x, 2]];
                let fg_ansi: String = generate_rgb_escape(fg_r, fg_g, fg_b, true);
                let bg_ansi: String = generate_rgb_escape(bg_r, bg_g, bg_b, false);
                let print_char:String = "▀".to_string();
                row_buffer.push(fg_ansi);
                row_buffer.push(bg_ansi);
                row_buffer.push(print_char);
                // print!("{}{}▀", generate_rgb_escape(fg_r, fg_g, fg_b, true), generate_rgb_escape(bg_r, bg_g, bg_b, false));
                // print!("▀");
            }

            println!("{}\x1b[0m", row_buffer.join(""));
        }
    }
}

fn gif_to_deque(image_path: &str) -> VecDeque<Array3<u8>> {
    let gif_file = BufReader::new(File::open(image_path).expect("\n===========\nIMAGE NOT FOUND!\n===========\n"));
    let mut decoder = GifDecoder::new(gif_file).unwrap();
    let frames = decoder.into_frames();
    let frames = frames.collect_frames().expect("\n===========\nFAILED TO COLLECT FRAMES!\n===========\n");

    let mut frame_deque: VecDeque<Array3<u8>> = VecDeque::new();

    // Extract frames from the GIF
    for frame in frames{
        let gif_rgb_array = process_frame(&frame);
        
        // Add the RGB array to the deque
        frame_deque.push_back(gif_rgb_array);
    }

    frame_deque
}

fn process_frame(frame: &Frame) -> Array3<u8> {
    let buffer: &ImageBuffer<Rgba<u8>, Vec<u8>> = &frame.clone().into_buffer();

    let (w, h) = buffer.dimensions();
    let mut rgb_array: Array3<u8> = Array3::zeros((h as usize, w as usize, 3));
    for (x, y, pixel) in buffer.enumerate_pixels() {
        let (r, g, b, _): (u8, u8, u8, u8) = pixel.0.into();
        rgb_array[[y as usize, x as usize, 0]] = r;
        rgb_array[[y as usize, x as usize, 1]] = g;
        rgb_array[[y as usize, x as usize, 2]] = b;
    }
    rgb_array
}

fn main() {
    // let image_path = "/home/charles/projects/rust-practice/rusty_hemera_mini/src/parrot.png";
    // let rgb_array = image_to_rgb_array(image_path);
    // print_rgb_array(&rgb_array);

    let gif_path: &str = "/home/charles/projects/rust-practice/rusty_hemera_mini/src/office.gif";
    let frame_deque = gif_to_deque(gif_path);
    let fps: u64 = 10;
    let time_between_frames = 1 / fps;
    loop {
        for frame in frame_deque.iter() {
            print!("\x1b[0;0H");
            print_rgb_array(&frame);
            // frame_sleep(100);
        }
    }
}