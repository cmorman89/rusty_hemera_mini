// extern crate image;
// extern crate ndarray;

use image::{GenericImageView, Rgba, RgbaImage, GifDecoder, DynamicImage};
use ndarray::{Array3, Array2, s};
use std::thread;
use std::time;
use std::collections::VecDeque;

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
        if y % 2 == 0 && y < h - 2 {
            for x in 0..w {
                let fg_r = rgb_array[[y, x, 0]];
                let fg_g = rgb_array[[y, x, 1]];
                let fg_b = rgb_array[[y, x, 2]];
                let bg_r = rgb_array[[y + 1, x, 0]];
                let bg_g = rgb_array[[y + 1, x, 1]];
                let bg_b = rgb_array[[y + 1, x, 2]];
                print!("{}", generate_rgb_escape(fg_r, fg_g, fg_b, true));
                print!("{}", generate_rgb_escape(bg_r, bg_g, bg_b, false));
                print!("▀");
            }

            println!("\x1b[0m");
        }
    }
}

fn gif_to_deque(image_path: &str) -> VecDeque<Array3<u8>> {
    let gif = image::open(image_path).expect("\n===========\nIMAGE NOT FOUND!\n===========\n");
    let decoder = GifDecoder::new(&gif).expect("\n===========\nGIF DECODER FAILED TO PARSE GIF!\n===========\n");

    let mut frame_deque: VecDeque<Array3<u8>> = VecDeque::new();

    // Extract frames from the GIF
    for frame in decoder.into_frames() {
        let frame = frame.expect("\n===========\nFAILED TO PARSE FRAME!\n===========\n");
        let buffer = frame.into_buffer();

        let (gif_width, gif_height) = buffer.dimensions();
        let mut gif_rgb_array: Array3<u8> = Array3::zeros((gif_height as usize, gif_width as usize, 3));

        // Get the RGB values of the GIF frame
        for (x, y, pixel) in img.to_rgba8().enumerate_pixels() {
            // For now, we are ignoring the alpha channel
            let (r, g, b, _): (u8, u8, u8, u8) = pixel.0.into();
            gif_rgb_array[[y as usize, x as usize, 0]] = r;
            gif_rgb_array[[y as usize, x as usize, 1]] = g;
            gif_rgb_array[[y as usize, x as usize, 2]] = b;

        }
        
        // Add the RGB array to the deque
        frame_deque.push_back(gif_rgb_array);
    }

    frames_deque
}

fn main() {
    let image_path = "/home/charles/projects/rust-practice/rusty_hemera_mini/src/parrot.png"; // Replace with your image file path

    // Get the RGB 3D array
    let rgb_array = image_to_rgb_array(image_path);
    // println!("RGB Array (3D):\n{:#?}", rgb_array);
    print_rgb_array(&rgb_array);
}