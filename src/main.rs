use ndarray::ArrayView3;  
use std::io::{self, Write};
use ffmpeg_next::{self as ffmpeg};
use rayon::prelude::*;

/// Prints an RGB array to the terminal
/// 
/// # Arguments
/// - `rgb_array` - The RGB array to print
/// - `frame_buffer` - The buffer to write the frame to
/// - `trim_stride` - Whether to trim the excess stride from the right side of the frame
/// 
/// # Returns
/// - None
fn print_rgb_array(rgb_array: &ArrayView3<u8>, frame_buffer: &mut Vec<u8>, trim_stride: bool) {

    // Clear the frame buffer from the previous frame
    frame_buffer.clear();

    // Get the dimensions of the RGB array
    let (h, w, _) = rgb_array.dim();

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
    frame_buffer.extend_from_slice(to_origin);
     // Only print even rows due to the stacked vertical pixels sharing one character cell
     // Use Rayon to parallelize the loop
     let row_chunks: Vec<_> = (0..h).step_by(2).collect();
     let chunk_results: Vec<Vec<u8>> = row_chunks
        .into_par_iter()
        .map(|y| {
            let mut chunk_buffer: Vec<u8> = Vec::new();
            for x in 0..w {
                
                // Extract the RGB values from the array
                // Foreground
                let fg_r = rgb_array[[y, x, 0]];
                let fg_g = rgb_array[[y, x, 1]];
                let fg_b = rgb_array[[y, x, 2]];
                // Background
                let bg_r = rgb_array[[y + 1, x, 0]];
                let bg_g = rgb_array[[y + 1, x, 1]];
                let bg_b = rgb_array[[y + 1, x, 2]];

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
                chunk_buffer.extend_from_slice(fg_ansi);
                chunk_buffer.extend_from_slice(&[fg_r / 100 + b'0', (fg_r % 100) / 10 + b'0', fg_r % 10 + b'0']);
                chunk_buffer.extend_from_slice(ansi_sep);
                chunk_buffer.extend_from_slice(&[fg_g / 100 + b'0', (fg_g % 100) / 10 + b'0', fg_g % 10 + b'0']);
                chunk_buffer.extend_from_slice(ansi_sep);
                chunk_buffer.extend_from_slice(&[fg_b / 100 + b'0', (fg_b % 100) / 10 + b'0', fg_b % 10 + b'0']);
                chunk_buffer.extend_from_slice(ansi_end);

                // Generate the background ANSI escape sequence
                chunk_buffer.extend_from_slice(bg_ansi);
                chunk_buffer.extend_from_slice(&[bg_r / 100 + b'0', (bg_r % 100) / 10 + b'0', bg_r % 10 + b'0']);
                chunk_buffer.extend_from_slice(ansi_sep);
                chunk_buffer.extend_from_slice(&[bg_g / 100 + b'0', (bg_g % 100) / 10 + b'0', bg_g % 10 + b'0']);
                chunk_buffer.extend_from_slice(ansi_sep);
                chunk_buffer.extend_from_slice(&[bg_b / 100 + b'0', (bg_b % 100) / 10 + b'0', bg_b % 10 + b'0']);
                chunk_buffer.extend_from_slice(ansi_end);

                // Append the block character
                chunk_buffer.extend_from_slice(print_char);
        }
        // Reset formatting and move to the next line
        chunk_buffer.extend_from_slice(ansi_reset);
        chunk_buffer
    })
    .collect();

    for chunk in chunk_results {
        frame_buffer.extend_from_slice(&chunk);
    }

    io::stdout().lock().write_all(&frame_buffer).unwrap();
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

    // Initialize ffmpeg
    ffmpeg::init().expect("Failed to initialize ffmpeg");

    // Open the video file to get the input context
    let mut ictx = ffmpeg::format::input(&video_path).expect("Failed to open input file");

    // Get the index of the correct (video) stream in the input video file
    let video_stream_index = ictx
        .streams()
        .best(ffmpeg::media::Type::Video).expect("Failed to get best video stream")
        .index();

    // Get the context from the video stream
    let decoder_context = ffmpeg::codec::context::Context::from_parameters(
        ictx
            .stream(video_stream_index).expect("Failed to get video stream")
            .parameters(),
        ).expect("Failed to create decoder");
    
    // Create a video decoder from the decoder context
    let mut decoder = decoder_context.decoder().video().expect("Failed to create video decoder");

    // Create a context to convert the video to RGB
    let mut video_to_rgb = ffmpeg::software::scaling::context::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::util::format::Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        ffmpeg::software::scaling::flag::Flags::FAST_BILINEAR
    ).expect("Failed to create video to RGB context");
    
    // Track if the needed variables are initialized
    let mut hemera_is_initialized = false;
    
    // Initialize the video frame and RGB frame outside of the loop
    let mut input_frame = ffmpeg::util::frame::Video::empty();
    let mut rgb_frame = ffmpeg::util::frame::Video::empty();
    let mut rgb_view: ArrayView3<u8>;
    let mut pixel_data: &[u8];
    
    // Create a buffer to store the printable frame data outside of the loop
    let mut frame_buffer: Vec<u8> = Vec::new();

    // Initialize dimensions outside of the loops
    let mut h: usize = 0;
    let mut w: usize;
    let mut stride_w: usize;
    let mut stride: usize;
    let mut byte_estimate: usize;

    // Track the current frame
    let mut frame_index = 0;

    // Loop through the packets in the input context
    for (stream, packet) in ictx.packets() {
        // If the packet is a video packet, send it to the decoder
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet).expect("Failed to send packet to decoder");

            // If the function has not initialized, initialize the variables
            if !hemera_is_initialized {
                // Get the actual height and width of the video
                h = decoder.height() as usize;
                w = decoder.width() as usize;
                // Calculate the stride of the RGB frame
                // (Since we dont yet know the stride of the RGB frame, we estimate it to be:
                // width of the frame * three pixels (RGB) + 4 bytes of padding per row)
                stride_w = (w + 4) * 3;

                // // Create an empty video frame to store the input frame
                // let mut input_frame = ffmpeg::util::frame::Video::empty();
                byte_estimate = h * stride_w * 20;
                frame_buffer = Vec::with_capacity(byte_estimate);
                hemera_is_initialized = true;
            }
        
            // While the decoder has frames to process, process them
            while decoder.receive_frame(&mut input_frame).is_ok() {
                if frame_index % 2 == 0 {
                    // Convert the video frame to an RGB frame
                    video_to_rgb.run(&input_frame, &mut rgb_frame).expect( "Failed to convert video frame to RGB");

                    // Now that we have the RGB frame, we can view the pixel data and the actual stride
                    pixel_data = rgb_frame.data(0);
                    stride = rgb_frame.stride(0) / 3;

                    // Create a view of the pixel data instead of copying it (ArrayView3 vs Array3)
                    rgb_view = ArrayView3::from_shape((h, stride, 3), pixel_data).expect("Failed to view pixel data");

                    // Print the RGB view to the terminal
                    print_rgb_array(&rgb_view, &mut frame_buffer, true);
                }
                // Increment the frame index
                frame_index += 1;

            }
        }
    }

}
