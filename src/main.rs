// extern crate image;
// extern crate ndarray;

use image::{Rgba, AnimationDecoder, Frame, ImageBuffer};
use ndarray::Array3;
use std::thread;
use std::time;
use std::collections::VecDeque;
use std::fs::File;
use image::codecs::gif::GifDecoder;
use std::io::BufReader;

fn generate_rgb_escape(r: &u8, g: &u8, b: &u8, is_foreground: bool) -> String {
    if is_foreground {
        format!("\x1b[38;2;{};{};{}m", r, g, b)
    } else {
        format!("\x1b[48;2;{};{};{}m", r, g, b)
    }
}

fn frame_sleep(fps: u32, frame_start: time::Instant) {
    let frame_duration = time::Duration::from_secs(1) / fps;
    let elapsed_time = frame_start.elapsed();
    if elapsed_time < frame_duration {
        thread::sleep(frame_duration - elapsed_time);
    }
    // thread::sleep(time::Duration::from_millis(duration_ms));
}


fn print_rgb_array(rgb_array: &Array3<u8>) {
    let (h, w, _) = rgb_array.dim();
    let mut fg_r: u8;
    let mut fg_g: u8;
    let mut fg_b: u8;
    let mut bg_r: u8;
    let mut bg_g: u8;
    let mut bg_b: u8;
    let mut fg_ansi: String = String::new();
    let mut bg_ansi: String = String::new();
    let print_char:String = "▀".to_string();
    let mut frame_buffer: String = String::new();
    for y in 0..h {
        let mut row_buffer: String = String::new();
        if y % 2 == 0 && y < h - 2 {
            for x in 0..w {
                fg_r = rgb_array[[y, x, 0]];
                fg_g = rgb_array[[y, x, 1]];
                fg_b = rgb_array[[y, x, 2]];
                bg_r = rgb_array[[y + 1, x, 0]];
                bg_g = rgb_array[[y + 1, x, 1]];
                bg_b = rgb_array[[y + 1, x, 2]];
                fg_ansi = generate_rgb_escape(&fg_r, &fg_g, &fg_b, true);
                bg_ansi = generate_rgb_escape(&bg_r, &bg_g, &bg_b, false);
                row_buffer.push_str(&fg_ansi);
                row_buffer.push_str(&bg_ansi);
                row_buffer.push_str(&print_char);
            }
            row_buffer.push_str("\x1b[0m\n");
            frame_buffer.push_str(&row_buffer);
            // println!("{}\x1b[0m", row_buffer);
        }
    }
    print!("\x1b[0;0H{}", frame_buffer);
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
    
    let gif_path: &str = "/home/charles/projects/rust-practice/rusty_hemera_mini/src/deshawn.gif";
    let frame_deque = gif_to_deque(gif_path);
    let fps: u32 = 10;
    let time_between_frames = 1 / fps;
    loop {
        for frame in frame_deque.iter() {
            // print!("\x1b[0;0H");
            let start_time = time::Instant::now();
            print_rgb_array(&frame);
            frame_sleep(60, start_time);
        }
    }
}
