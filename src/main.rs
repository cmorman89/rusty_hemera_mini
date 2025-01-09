use ndarray::Array3;
use std::io::{self, Write};
use ffmpeg_next::{self as ffmpeg, frame};

/// Prints an RGB array to the terminal
/// 
/// # Arguments
/// - `rgb_array` - The RGB array to print
/// - `frame_buffer` - The buffer to write the frame to
/// 
/// # Returns
/// - None
fn print_rgb_array(rgb_array: &Array3<u8>, frame_buffer: &mut Vec<u8>, trim_stride: bool) {

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
    // The stride adds 4 bytes to the end of the each row of the video frame
    // This is due to the padding added by the ffmpeg software scaler
    // Slicing the ndarray will cause memory reallocation and slow down the rendering
    // Instead, we simply do not iterate over the last 4 columns of the frame
    let w = if trim_stride { w - 32 } else { w };
    // Only print even rows due to the stacked vertical pixels sharing one character cell
    for y in (0..h).step_by(2) {
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
    handle.write_all(&frame_buffer).unwrap();
}



/// Converts a u8 value to a 3 element array of ASCII bytes
/// Avoids overhead of heap allocation for string conversion
/// 
/// # Arguments
/// - `u8_val` - The u8 value to convert
/// 
/// # Returns
/// - A 3 element array of ASCII bytes
fn u8_to_bytes(u8_val: u8) -> [u8; 3] {
    
    let mut bytes: [u8; 3] = [0; 3];
    bytes[0] = u8_val / 100 + b'0';
    bytes[1] = (u8_val % 100) / 10 + b'0';
    bytes[2] = u8_val % 10 + b'0';
    bytes
}


fn main() {
    // Initialize ffmpeg
    ffmpeg::init().expect("Failed to initialize ffmpeg");

    // Open the video file and get the input context
    let video_path = "asset/small_2.mp4";
    let mut ictx = ffmpeg::format::input(&video_path).expect("Failed to open input file");

    // Get the index of the video stream in the video file
    let video_stream_index = ictx
        .streams()
        .best(ffmpeg::media::Type::Video).expect("Failed to get best video stream")
        .index();

    // Get the video stream parameters using the video stream index;
    // and create a decoder from the parameters
    let decoder_context = ffmpeg::codec::context::Context::from_parameters(
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
        ffmpeg::software::scaling::flag::Flags::BILINEAR
    ).expect("Failed to create video to RGB context");
    let mut frame_index = 0;
    let mut frame_buffer: Vec<u8> = Vec::new();
    let mut frame_buffer_is_initialized = false;
    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            // If the packet is a video packet, send it to the decoder
            decoder.send_packet(&packet).expect("Failed to send packet to decoder");
            let h = decoder.height() as usize;
            let w = decoder.width() as usize;
            let stride_w = (w + 4) * 3;
            let mut input_frame = ffmpeg::util::frame::Video::empty();
            let byte_estimate: usize = h * stride_w * 20;
            if !frame_buffer_is_initialized {
                frame_buffer = Vec::with_capacity(byte_estimate);
                frame_buffer_is_initialized = true;
            }
        
            // While the decoder has frames to process, process them
            while decoder.receive_frame(&mut input_frame).is_ok() {
                // Create an empty RGB frame
                let mut rgb_frame = ffmpeg::util::frame::Video::empty();
                // Convert the video frame to an RGB frame
                video_to_rgb.run(&input_frame, &mut rgb_frame).expect( "Failed to convert video frame to RGB");
                // Get the pixel data from the RGB frame
                // let w = rgb_frame.stride(0);
                let pixel_data = rgb_frame.data(0);
                // Turn the pixel data into a vector
                let w = rgb_frame.stride(0);
                let rgb_ndarray = Array3::from_shape_vec((h, w / 3, 3), pixel_data.to_vec()).expect("Failed to create ndarray");
                
                frame_index += 1;
                print_rgb_array(&rgb_ndarray, &mut frame_buffer, true);

            }
        }
    }

}
