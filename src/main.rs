// extern crate image;
// extern crate ndarray;
use image::{Rgba, AnimationDecoder, Frame, ImageBuffer};
use ndarray::{Array2, Array3};
use std::{thread, time, vec};
use std::collections::VecDeque;
use std::fs::File;
use image::codecs::gif::GifDecoder;
use std::io::{BufReader, self, Write};
use ffmpeg_next::{self as ffmpeg, frame};

fn frame_sleep(fps: &u32, frame_start: time::Instant) {
    /// Sleeps the thread to maintain a target frame rate
    /// 
    /// # Arguments
    /// - `fps` - The target frames per second
    /// - `frame_start` - The time the frame started
    /// 
    /// # Returns
    /// - None
    
    // Calculate the frame duration based on the target frames per second
    let frame_duration = time::Duration::from_secs(1) / fps.clone();
    // Calculate the elapsed time since the frame started
    let elapsed_time = frame_start.elapsed();
    // If the frame finished rendering faster than the target frame duration, sleep the thread
    if elapsed_time < frame_duration {
        thread::sleep(frame_duration - elapsed_time);
    }
}


fn print_rgb_array(rgb_array: &Array3<u8>, frame_buffer: &mut Vec<u8>) {
    /// Prints an RGB array to the terminal
    /// 
    /// # Arguments
    /// - `rgb_array` - The RGB array to print
    /// - `frame_buffer` - The buffer to write the frame to
    /// 
    /// # Returns
    /// - None
    
    // Clear the frame buffer from the previous frame
    frame_buffer.clear();

    // Get the stdout handle
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Preallocate the pixel values
    let mut fg_r: u8;
    let mut fg_g: u8;
    let mut fg_b: u8;
    let mut bg_r: u8;
    let mut bg_g: u8;
    let mut bg_b: u8;

    // ANSI escape sequences pre-computed for performance
    let to_origin = b"\x1b[0H";
    let fg_ansi: &[u8; 7] = b"\x1b[38;2;";
    let bg_ansi: &[u8; 7] = b"\x1b[48;2;";
    let ansi_sep: &[u8; 1] = b";";
    let ansi_end: &[u8; 1] = b"m";
    let print_char: &[u8] = "▀".as_bytes();

    // Start printing:
    // Move the cursor to the top left corner
    frame_buffer.extend_from_slice(to_origin);
    let (h, w, _) = rgb_array.dim();
    for y in 0..h {
        // Only print even rows due to the stacked vertical pixels sharing one character cell
        if y % 2 == 0 && y < h - 2 {
            for x in 0..w {
                // Extract the RGB values from the array
                // Foreground
                fg_r = rgb_array[[y, x, 0]];
                fg_g = rgb_array[[y, x, 1]];
                fg_b = rgb_array[[y, x, 2]];
                // Background
                bg_r = rgb_array[[y + 1, x, 0]];
                bg_g = rgb_array[[y + 1, x, 1]];
                bg_b = rgb_array[[y + 1, x, 2]];

                // Generate the foreground ANSI escape sequence
                frame_buffer.extend_from_slice(fg_ansi);
                frame_buffer.extend_from_slice(&u8_to_bytes(fg_r));
                frame_buffer.extend_from_slice(ansi_sep);
                frame_buffer.extend_from_slice(&u8_to_bytes(fg_g));
                frame_buffer.extend_from_slice(ansi_sep);
                frame_buffer.extend_from_slice(&u8_to_bytes(fg_b));
                frame_buffer.extend_from_slice(ansi_end);

                // Generate the background ANSI escape sequence
                frame_buffer.extend_from_slice(bg_ansi);
                frame_buffer.extend_from_slice(&u8_to_bytes(bg_r));
                frame_buffer.extend_from_slice(ansi_sep);
                frame_buffer.extend_from_slice(&u8_to_bytes(bg_g));
                frame_buffer.extend_from_slice(ansi_sep);
                frame_buffer.extend_from_slice(&u8_to_bytes(bg_b));
                frame_buffer.extend_from_slice(ansi_end);

                // Print the character
                frame_buffer.extend_from_slice(&print_char);
            }
            frame_buffer.extend_from_slice(b"\x1b[0m\n");
        }
    }
    handle.write_all(&frame_buffer).unwrap();
}

fn initialize_rasterizer() {
    /// Initializes the rasterizer by preallocating buffers, variables, and precalculating byte-
    /// representations of ANSI escape sequences.
    
    // Clear the screen
    print!("\x1b[2J");
    // Hide the cursor
    print!("\x1b[?25l");
}


fn u8_to_bytes(u8_val: u8) -> [u8; 3] {
    /// Converts a u8 value to a 3 element array of ASCII bytes
    /// Avoids overhead of heap allocation for string conversion
    /// 
    /// # Arguments
    /// - `u8_val` - The u8 value to convert
    /// 
    /// # Returns
    /// - A 3 element array of ASCII bytes
    
    let mut bytes: [u8; 3] = [0; 3];
    bytes[0] = u8_val / 100 + b'0';
    bytes[1] = (u8_val % 100) / 10 + b'0';
    bytes[2] = u8_val % 10 + b'0';
    bytes
}

fn gif_to_deque(image_path: &str) -> VecDeque<Array3<u8>> {
    /// Converts a GIF file to a deque of RGB arrays
    /// 
    /// # Arguments
    /// - `image_path` - The path to the GIF file
    /// 
    /// # Returns
    /// - A deque of RGB arrays
    
    // Create a deque to store the frames
    let mut frame_deque: VecDeque<Array3<u8>> = VecDeque::new();

    // Open the GIF file and get the frames
    let gif_file = BufReader::new(File::open(image_path).expect("\n===========\nIMAGE NOT FOUND!\n===========\n"));
    let decoder = GifDecoder::new(gif_file).unwrap();
    let frames = decoder.into_frames();
    let frames = frames.collect_frames().expect("\n===========\nFAILED TO COLLECT FRAMES!\n===========\n");
    
    // Extract frames from the GIF
    for frame in frames{
        let gif_rgb_array = process_frame(&frame);
        // Add the RGB array to the deque
        frame_deque.push_back(gif_rgb_array);
    }
    // Return the deque of RGB array frames
    frame_deque
}

fn process_frame(frame: &Frame) -> Array3<u8> {
    /// Processes a GIF frame into an RGB array
    /// 
    /// # Arguments
    /// - `frame` - The GIF frame to process
    /// 
    /// # Returns
    /// - An RGB array
    
    // Get the buffer from the frame
    let buffer: &ImageBuffer<Rgba<u8>, Vec<u8>> = &frame.buffer();
    // Get the dimensions of the buffer
    let (w, h) = buffer.dimensions();
    // Create an RGB array to store the pixel values
    let mut rgb_array: Array3<u8> = Array3::zeros((h as usize, w as usize, 3));
    for (x, y, pixel) in buffer.enumerate_pixels() {
        let (r, g, b, _): (u8, u8, u8, u8) = pixel.0.into();
        rgb_array[[y as usize, x as usize, 0]] = r;
        rgb_array[[y as usize, x as usize, 1]] = g;
        rgb_array[[y as usize, x as usize, 2]] = b;
    }
    // Return the RGB array
    rgb_array
}

fn main() {
    // Initialize ffmpeg
    ffmpeg::init().expect("Failed to initialize ffmpeg");

    // Open the video file and get the input context
    let video_path = "asset/planet-earth.mp4";
    let mut ictx = ffmpeg::format::input(&video_path).expect("Failed to open input file");

    // Get the index of the video stream in the video file
    let video_stream_index = ictx
        .streams()
        .best(ffmpeg::media::Type::Video).expect("Failed to get best video stream")
        .index();

    // Get the video stream parameters using the video stream index;
    // and create a decoder from the parameters
    let mut decoder_context = ffmpeg::codec::context::Context::from_parameters(
        ictx
            .stream(video_stream_index).expect("Failed to get video stream")
            .parameters(),
        ).expect("Failed to create decoder");
    let mut decoder = decoder_context.decoder().video().expect("Failed to create video decoder");

    let mut video_to_rgb = ffmpeg::software::scaling::context::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::util::format::Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        // Uncomment for bilinear scaling
        ffmpeg::software::scaling::flag::Flags::BILINEAR
        // Uncomment for nearest neighbor scaling
        // ffmpeg::software::scaling::flag::Flags::NEAREST,
    ).expect("Failed to create video to RGB context");

    let mut frame_index = 0;

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            // If the packet is a video packet, send it to the decoder
            decoder.send_packet(&packet).expect("Failed to send packet to decoder");
            let h = decoder.height() as usize;
            let w = (decoder.width() as usize + 4) * 3;
            let mut input_frame = ffmpeg::util::frame::Video::empty();
            let byte_estimate: usize = h * w * 20;
            let mut frame_buffer: Vec<u8> = Vec::with_capacity(byte_estimate);
        
            // While the decoder has frames to process, process them
            while decoder.receive_frame(&mut input_frame).is_ok() {
                // Create an empty RGB frame
                let mut rgb_frame = ffmpeg::util::frame::Video::empty();
                // Convert the video frame to an RGB frame
                video_to_rgb.run(&input_frame, &mut rgb_frame).expect( "Failed to convert video frame to RGB");
                // Get the pixel data from the RGB frame
                // let w = rgb_frame.stride(0);
                let stride = rgb_frame.stride(0);
                let pixel_data = rgb_frame.data(0);
                // Turn the pixel data into a vector
                println!("Stride: {}", stride);
                println!("Frame {}: Pixel data length: {} vs h ({}) * w ({}) = {}", &frame_index, pixel_data.len(), &h, &w, h * w);
                let w = rgb_frame.stride(0);
                let rgb_ndarray = Array3::from_shape_vec((h, w / 3, 3), pixel_data.to_vec()).expect("Failed to create ndarray");
                
                frame_index += 1;
                print_rgb_array(&rgb_ndarray, &mut frame_buffer);

            }
        }
    }


    // Path to the GIF file
    let gif_path: &str = "asset/planet-earth.gif";
    // Set the target frames per second
    let fps: u32 = 30;

    // Convert the GIF to a deque of RGB arrays
    let frame_deque = gif_to_deque(gif_path);
    let rgb_array = frame_deque.front().unwrap();
    let (h, w, _) = rgb_array.dim();
    let byte_estimate: usize = h * w * 20;
    let mut frame_buffer: Vec<u8> = Vec::with_capacity(byte_estimate);

    print!("\x1b[0;0H");
    loop {
        for frame in frame_deque.iter() {
            let start_time = time::Instant::now();
            print_rgb_array(&frame, &mut frame_buffer);
            frame_sleep(&fps, start_time);
        }
    }
}
