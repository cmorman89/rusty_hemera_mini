use ndarray::{Array3, ArrayView3, Zip};
use std::io::{self, Write};
use std::mem;
use ffmpeg_next::{self as ffmpeg};

/// Prints an RGB array to the terminal
/// 
/// # Arguments
/// - `rgb_array` - The RGB array to print
/// - `frame_buffer` - The buffer to write the frame to
/// - `trim_stride` - Whether to trim the excess stride from the right side of the frame
/// 
/// # Returns
/// - None
fn print_rgb_array(rgb_backbuffer: &ArrayView3<u8>, byte_buffer: &mut Vec<u8>, trim_stride: bool) {

    // Clear the frame buffer from the previous frame
    byte_buffer.clear();

    // Get the dimensions of the RGB array
    let (h, w, _) = rgb_backbuffer.dim();

    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // TO-DO: Fix something weird with the video on the right column
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    let w = if trim_stride { w - 32 } else { w };

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
    let ansi_reset: &[u8; 5] = b"\x1b[0m\n";

    // Start printing:
    // Move the cursor to the top left corner
    byte_buffer.extend_from_slice(to_origin);
     // Only print even rows due to the stacked vertical pixels sharing one character cell
    for y in (0..h).step_by(2) {
        for x in 0..w {
            // Extract the RGB values from the array
            // Foreground
            fg_r = rgb_backbuffer[[y, x, 0]];
            fg_g = rgb_backbuffer[[y, x, 1]];
            fg_b = rgb_backbuffer[[y, x, 2]];
            // Background
            bg_r = rgb_backbuffer[[y + 1, x, 0]];
            bg_g = rgb_backbuffer[[y + 1, x, 1]];
            bg_b = rgb_backbuffer[[y + 1, x, 2]];

            // Generate the foreground ANSI escape sequence
            //
            // - The format is: 
            //   - Foreground: \x1b[38;2;{r};{g};{b}m
            //   - Background: \x1b[48;2;{r};{g};{b}m
            //
            // - The mathmatical operations are used to convert the u8 to bytes without using a string
            //   (A string would require a heap allocation for every pixel color *channel* -- too slow)
            //
            // - The math was moved inline to avoid the overhead of a function call, again for every pixel
            //   color *channel* (3 per pixel, 6 per loop iteration, or 7,200,000 calls per second for a 
            //   400*200 video at 30fps. Ouch.)
            byte_buffer.extend_from_slice(fg_ansi);
            byte_buffer.extend_from_slice(&[fg_r / 100 + b'0', (fg_r % 100) / 10 + b'0', fg_r % 10 + b'0']);
            byte_buffer.extend_from_slice(ansi_sep);
            byte_buffer.extend_from_slice(&[fg_g / 100 + b'0', (fg_g % 100) / 10 + b'0', fg_g % 10 + b'0']);
            byte_buffer.extend_from_slice(ansi_sep);
            byte_buffer.extend_from_slice(&[fg_b / 100 + b'0', (fg_b % 100) / 10 + b'0', fg_b % 10 + b'0']);
            byte_buffer.extend_from_slice(ansi_end);

            // Generate the background ANSI escape sequence
            byte_buffer.extend_from_slice(bg_ansi);
           byte_buffer.extend_from_slice(&[bg_r / 100 + b'0', (bg_r % 100) / 10 + b'0', bg_r % 10 + b'0']);
            byte_buffer.extend_from_slice(ansi_sep);
            byte_buffer.extend_from_slice(&[bg_g / 100 + b'0', (bg_g % 100) / 10 + b'0', bg_g % 10 + b'0']);
            byte_buffer.extend_from_slice(ansi_sep);
            byte_buffer.extend_from_slice(&[bg_b / 100 + b'0', (bg_b % 100) / 10 + b'0', bg_b % 10 + b'0']);
            byte_buffer.extend_from_slice(ansi_end);

            // Append the block character
            byte_buffer.extend_from_slice(print_char);
        }
        // Reset formatting and move to the next line
        byte_buffer.extend_from_slice(ansi_reset);
    }
    io::stdout().lock().write_all(&byte_buffer).unwrap();
}

fn generate_byte_buffer(h: usize, w: usize) -> Vec<u8> {
    let mut byte_buffer: Vec<u8> = Vec::with_capacity(h * w * 20);
    byte_buffer
}

fn generate_framebuffers(h: usize, w: usize) -> (ndarray::Array3<u8>, ndarray::Array3<u8>, ndarray::Array3<u8>) {
    let backbuffer = Array3::<u8>::zeros((h, w, 3));
    let frontbuffer = Array3::<u8>::zeros((h, w, 3));
    let delta_framebuffer = Array3::<u8>::zeros((h, w, 3));
    (backbuffer, frontbuffer, delta_framebuffer)
}

fn swap_framebuffer_pointers(backbuffer: &mut Array3<u8>, frontbuffer: &mut Array3<u8>) {
    mem::swap(backbuffer, frontbuffer);
}

fn generate_delta_frame(front_frame: &ArrayView3<u8>, back_frame: &ArrayView3<u8>, delta_frame: &mut Array3<u8>) {
    // print!(
    //     "Front Frame:
    //     height: {}
    //     width: {}

    //     Back Frame:
    //     height: {}
    //     width: {}

    //     Delta Frame:
    //     height: {}
    //     width: {}
    //     ",
    //     front_frame.shape()[0],
    //     front_frame.shape()[1],
    //     back_frame.shape()[0],
    //     back_frame.shape()[1],
    //     delta_frame.shape()[0],
    //     delta_frame.shape()[1]

    // );
    Zip::from(delta_frame)
        .and(front_frame)
        .and(back_frame)
        .for_each(|delta_pixel, front_pixel, back_pixel| {
            if back_pixel != front_pixel {
                *delta_pixel = *back_pixel;
                // println!("\nC: CHANGED Pixel Data:");
                // println!("     -  Front: {} ", front_pixel);
                // println!("     -  Back:  {} ", back_pixel);
                // println!("     -> Delta: {} ", delta_pixel);
                // std::process::exit(0);
            }
            else {
                *delta_pixel = 0;
                // println!("\nU: UNCHANGED Pixel Data:");
                // println!("     -  Front: {} ", front_pixel);
                // println!("     -  Back:  {} ", back_pixel);
                // println!("     -> Delta: {} ", delta_pixel);
            }
        });
}

fn rasterize_delta_frame(delta_frame: &Array3<u8>, byte_buffer: &mut Vec<u8>) {
    // Steps:
    // 1. Clear the byte buffer
    // 2. Move the cursor to the top left corner
    // 3. Loop through the delta frame
    // 4. Skip pixels with a value of 0
    // 5. If printing pixel found, issue cursor relocate and add to byte buffer
    // 6. Track ANSI color state to avoid redundant ANSI sequences
    // 7. Continue until zeroed pixels are found
    // 8. Repeat until the end of the frame
    // 9. Issue a reset ANSI sequence to clear the color state 
    print!("Delta Frame:");
    let (h, w, _) = delta_frame.dim();
    print!("R:\n {:?}", &delta_frame.slice(ndarray::s![.., .., 0]));
    print!("G:\n {:?}", &delta_frame.slice(ndarray::s![.., .., 1]));
    print!("B:\n {:?}", &delta_frame.slice(ndarray::s![.., .., 2]));
}

fn print_frame(byte_buffer: &Vec<u8>) {
    io::stdout().lock().write_all(&byte_buffer).unwrap();
}

fn play_file(file_path: &str) {
    // Initialize ffmpeg
    initialize_ffmpeg();

    // Open the video file to get the input context and the video stream index
    let mut ictx: ffmpeg_next::format::context::Input = open_file(file_path);
    let video_stream_index = get_video_stream_index(&ictx);

    // Get the video decoder
    let mut decoder = get_decoder(&ictx);

    // Create video to RGB context and empty ffmpeg frames to hold each
    let mut rgb_conversion_context = get_video_to_rgb_context(&decoder);
    let mut ffmpeg_frame = generate_ffmpeg_frame();
    let mut rgb_frame = generate_ffmpeg_frame();

    // Get the dimensions of the video
    let ffmpeg_h = decoder.height() as usize;
    let ffmpeg_w = decoder.width() as usize;
    let mut ffmpeg_s: usize = (&ffmpeg_w + 4);

    // Generate the framebuffers
    let (backbuffer, frontbuffer, mut delta_framebuffer) = generate_framebuffers(ffmpeg_h, ffmpeg_s);
    let mut rgb_pixel_data: &[u8];

    // Create the byte buffer to store the final output
    let mut byte_buffer = generate_byte_buffer(ffmpeg_h, ffmpeg_w);
    let mut frame_index = 0;

    // Loop through the packets in the input context
    for (stream, packet) in ictx.packets() {
        // If the packet is a video packet, send it to the decoder
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet).expect("Failed to send packet to decoder");

            // // If the function has not initialized, initialize the variables
            // if !hemera_is_initialized {
            //     // Get the actual height and width of the video
            //     h = decoder.height() as usize;
            //     w = decoder.width() as usize;
            //     // Calculate the stride of the RGB frame
            //     // (Since we dont yet know the stride of the RGB frame, we estimate it to be:
            //     // width of the frame * three pixels (RGB) + 4 bytes of padding per row)
                // stride_w = (w + 4) * 3;

                // // // Create an empty video frame to store the input frame
                // // let mut input_frame = ffmpeg::util::frame::Video::empty();
                // byte_estimate = h * stride_w * 20;
                // frame_buffer = Vec::with_capacity(byte_estimate);
                // hemera_is_initialized = true;
            // }
        
            // While the decoder has frames to process, process them
            while decoder.receive_frame(&mut ffmpeg_frame).is_ok() {
                if frame_index % 2 == 0 {
                    // Convert the video frame to an RGB frame
                    rgb_conversion_context.run(&ffmpeg_frame, &mut rgb_frame).expect( "Failed to convert video frame to RGB");
                    // Create a view of the pixel data instead of copying it (ArrayView3 vs Array3)
                    let backbuffer = ArrayView3::from_shape((ffmpeg_h, ffmpeg_s, 3), rgb_frame.data(0))
                        .expect(
                            &format!(
                                "Failed to view pixel data.
                                "

                                // =========================
                                // ffmpeg dimensions:
                                // ffmpeg_h: {}
                                // ffmpeg_w: {}

                                // =========================
                                // Calculated stride = (ffmpeg_w + pad) * 3:
                                // Assumed pad: 4
                                // ffmpeg_s (calc): {}

                                // =========================
                                // rgb pixel dimensions:
                                // rgb_pixel_data.len(): {}
                                // rgb_pixel_data.len() / h: {}
                                // rgb_pixel_data.len() / h / 3: {}
                                // (rgb_pixel_data.len() / h / 3) - ffmpeg_w: {}

                                // =========================
                                // ACTUAL PER-CHANNEL STRIDE PAD (Calc): {}
                                // =========================

                                // =========================
                                // rgb frame dimensions:
                                // rgb_frame_h (mesaured): {}
                                // rgb_frame_w (mesaured): {}
                                // rgb_frame_stride (measured): {}
                                
                                // =========================
                                // dimensions used for ArrayView3:
                                // ArrayView3: (ffmpeg_h ({}), ffmpeg_s ({}), 3)
                                // Total AV3 size: {}
                                // Total rgb data size: {}

                                // =========================
                                // Total Element Difference
                                // Difference: {}
                                // ",
                                // ffmpeg_h,
                                // ffmpeg_w,
                                // ffmpeg_s,
                                // rgb_pixel_data.len(),
                                // rgb_pixel_data.len() / ffmpeg_h,
                                // rgb_pixel_data.len() / ffmpeg_h / 3,
                                // (rgb_pixel_data.len() / ffmpeg_h / 3) - ffmpeg_w,
                                // (rgb_pixel_data.len() / ffmpeg_h / 3) - ffmpeg_w,
                                // rgb_frame_h,
                                // rgb_frame_w,
                                // rgb_frame.stride(0),
                                // ffmpeg_h,
                                // ffmpeg_s,
                                // ffmpeg_h * ffmpeg_s,
                                // rgb_pixel_data.len(),
                                // (ffmpeg_h * ffmpeg_s) as isize - rgb_pixel_data.len() as isize
                        ));
                        let frontbuffer_zero = vec![0 as u8; ffmpeg_h * ffmpeg_s * 3];
                        let frontbuffer = ArrayView3::from_shape((ffmpeg_h, ffmpeg_s, 3), &frontbuffer_zero).expect("frontbuffer err");

                    // Print the RGB view to the terminal
                    // =========================
                    // WORKING IF YOU USE PRINT_RGB_ARRAY
                    // =========================
                    // print_rgb_array(&backbuffer, &mut byte_buffer, true);

                    // =========================
                    // WORK IN PROGRESS - ONLY PRINTS SIZES SO FAR
                    // =========================
                    generate_delta_frame(&frontbuffer, &backbuffer, &mut delta_framebuffer);
                    rasterize_delta_frame(&delta_framebuffer, &mut byte_buffer);
                }
                // Increment the frame index
                frame_index += 1;
                if frame_index == 2 {
                    std::process::exit(0);
                }

            }
        }
    }

}

fn initialize_ffmpeg() {
    match ffmpeg::init() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Failed to initialize ffmpeg: {}", e);
            std::process::exit(1);
        }
    }
}

fn open_file(file_path: &str) -> ffmpeg::format::context::Input {
    match ffmpeg::format::input(file_path) {
        Ok(ictx) => {
            ictx
        },
        Err(e) => {
            eprintln!("Failed to open input file: {}", e);
            std::process::exit(1);
        }
    }
}

fn get_video_stream_index(ictx: &ffmpeg::format::context::Input) -> usize {
    match ictx.streams().best(ffmpeg::media::Type::Video) {
        Some(video_stream) => {
            video_stream.index()
        },
        None => {
            eprintln!("Failed to get best video stream");
            std::process::exit(1);
        }
    }
}

fn get_decoder_context(ictx: &ffmpeg::format::context::Input, video_stream_index: usize) -> ffmpeg_next::codec::Context {
    let decoder_context = ffmpeg::codec::context::Context::from_parameters(
        ictx
            .stream(video_stream_index).expect("Failed to get video stream")
            .parameters(),
        ).expect("Failed to create decoder");
    decoder_context
}

fn get_decoder(ictx: &ffmpeg_next::format::context::Input) -> ffmpeg_next::codec::decoder::Video {

    let decoder_context = get_decoder_context(&ictx, get_video_stream_index(&ictx));
    match decoder_context.decoder().video() {
        Ok(decoder) => {
            decoder
        },
        Err(e) => {
            eprintln!("Failed to create video decoder: {}", e);
            std::process::exit(1);
        }
    }
}


fn get_video_to_rgb_context(decoder: &ffmpeg_next::codec::decoder::Video) -> ffmpeg::software::scaling::context::Context {
    let src_format = decoder.format();
    let src_w = decoder.width();
    let src_h = decoder.height();
    let dst_format = ffmpeg::util::format::Pixel::RGB24;
    let dst_w = decoder.width();
    let dst_h = decoder.height();
    let flags = ffmpeg::software::scaling::flag::Flags::FAST_BILINEAR;
    match ffmpeg::software::scaling::context::Context::get(src_format, src_w, src_h, dst_format, dst_w, dst_h, flags) {
        Ok(video_to_rgb) => {
            video_to_rgb
        },
        Err(e) => {
            eprintln!("Failed to create video to RGB context: {}", e);
            std::process::exit(1);
        }
    }
}

fn generate_ffmpeg_frame() -> ffmpeg::util::frame::Video {
    ffmpeg::util::frame::Video::empty()
}
fn main() {

    // HOW TO USE:
    // 1. Change the video_path to the path of the video you want to play
    // 2. Change to the project directory if you are not already there
    // 3. Run the program with `cargo run --release`
    // 4. Enjoy the video!
    //
    // Tips:
    // - Be sure to zoom out of the terminal to see the full video!
    // - Use Ctrl+C to stop the video
    // - Try to limit the input video to 400x200 @ 30fps for the best results
    // 
    // Dependencies:
    // - Rust Cargo Packages:
    //   - ffmpeg-next
    //   - ndarray
    // - System Dependencies:
    //   - ffmpeg-devel (or equivalent)
    
    // Path to the video file
    let video_path = "asset/cp2077.mp4";

    play_file(video_path);




    // ======================================================
    // // Initialize ffmpeg
    // ffmpeg::init().expect("Failed to initialize ffmpeg");

    // // Open the video file to get the input context
    // let mut ictx = ffmpeg::format::input(&video_path).expect("Failed to open input file");

    // // Get the index of the correct (video) stream in the input video file
    // let video_stream_index = ictx
    //     .streams()
    //     .best(ffmpeg::media::Type::Video).expect("Failed to get best video stream")
    //     .index();

    // // Get the context from the video stream
    // let decoder_context: ffmpeg_next::codec::Context = ffmpeg::codec::context::Context::from_parameters(
    //     ictx
    //         .stream(video_stream_index).expect("Failed to get video stream")
    //         .parameters(),
    //     ).expect("Failed to create decoder");
    
    // // Create a video decoder from the decoder context
    // let mut decoder = decoder_context.decoder().video().expect("Failed to create video decoder");

    // Create a context to convert the video to RGB
    // let mut video_to_rgb = ffmpeg::software::scaling::context::Context::get(
    //     decoder.format(),
    //     decoder.width(),
    //     decoder.height(),
    //     ffmpeg::util::format::Pixel::RGB24,
    //     decoder.width(),
    //     decoder.height(),
    //     ffmpeg::software::scaling::flag::Flags::FAST_BILINEAR
    // ).expect("Failed to create video to RGB context");
    
    // // Track if the needed variables are initialized
    // let mut hemera_is_initialized = false;
    
    // // Initialize the video frame and RGB frame outside of the loop
    // let mut input_frame = ffmpeg::util::frame::Video::empty();
    // let mut rgb_frame = ffmpeg::util::frame::Video::empty();
    // let mut rgb_view: ArrayView3<u8>;
    // let mut pixel_data: &[u8];
    
    // // Create a buffer to store the printable frame data outside of the loop
    // let mut frame_buffer: Vec<u8> = Vec::new();

    // // Initialize dimensions outside of the loops
    // let mut h: usize = 0;
    // let mut w: usize;
    // let mut stride_w: usize;
    // let mut stride: usize;
    // let mut byte_estimate: usize;

    // Track the current frame
    // let mut frame_index = 0;

    // // Loop through the packets in the input context
    // for (stream, packet) in ictx.packets() {
    //     // If the packet is a video packet, send it to the decoder
    //     // if stream.index() == video_stream_index {
    //     //     decoder.send_packet(&packet).expect("Failed to send packet to decoder");

    //     //     // If the function has not initialized, initialize the variables
    //     //     if !hemera_is_initialized {
    //     //         // Get the actual height and width of the video
    //     //         h = decoder.height() as usize;
    //     //         w = decoder.width() as usize;
    //     //         // Calculate the stride of the RGB frame
    //     //         // (Since we dont yet know the stride of the RGB frame, we estimate it to be:
    //     //         // width of the frame * three pixels (RGB) + 4 bytes of padding per row)
    //     //         stride_w = (w + 4) * 3;

    //     //         // // Create an empty video frame to store the input frame
    //     //         // let mut input_frame = ffmpeg::util::frame::Video::empty();
    //     //         byte_estimate = h * stride_w * 20;
    //     //         frame_buffer = Vec::with_capacity(byte_estimate);
    //     //         hemera_is_initialized = true;
    //     //     }
        
    //     //     // // While the decoder has frames to process, process them
    //     //     // while decoder.receive_frame(&mut input_frame).is_ok() {
    //     //     //     if frame_index % 2 == 0 {
    //     //     //         // Convert the video frame to an RGB frame
    //     //     //         video_to_rgb.run(&input_frame, &mut rgb_frame).expect( "Failed to convert video frame to RGB");

    //     //     //         // Now that we have the RGB frame, we can view the pixel data and the actual stride
    //     //     //         pixel_data = rgb_frame.data(0);
    //     //     //         stride = rgb_frame.stride(0) / 3;

    //     //     //         // Create a view of the pixel data instead of copying it (ArrayView3 vs Array3)
    //     //     //         rgb_view = ArrayView3::from_shape((h, stride, 3), pixel_data).expect("Failed to view pixel data");

    //     //     //         // Print the RGB view to the terminal
    //     //     //         print_rgb_array(&rgb_view, &mut frame_buffer, true);
    //     //     //     }
    //     //     //     // Increment the frame index
    //     //     //     frame_index += 1;

    //     //     // }
    //     // }
    // }

}
