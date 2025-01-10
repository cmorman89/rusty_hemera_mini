# Rusty Hemera (Mini)

## Purpose:

This is a sub-project of the NyxEngine repo. It is a minimalistic version of the terminal rasterization module, `HemeraTermFx`, used for testing and debugging as I convert some core components of the NyxEngine to Rust.

GIFs and videos are being used as input material for testing -- hence the lack of any video game assets.

### Note on code quality:

I am learning Rust as I go, so this code may not be idiomatic Rust -- or even correct Rust. But, I've always found the best way to learn is to bite off more than you can chew...and chew like hell until it makes sense. 🤯

This repo is:
- A learning tool
- A testing ground
- PoC/viability testing
- Rapid prototyping

> [!WARNING]
> This repo is not:
>- A demonstration of best practices
>- A demonstration of idiomatic Rust
>- An example of good GitHub housekeeping
>- Inherently useful to anyone but me -- yet

## Progress:

- 1-9-2025:
> I am bouncing back and forth between this, two client websites, and a few other projects. I'd like to spend more time on this, but for now I am happy with the progress. Videos are relatively smooth, considering the lack of any major optimization to the amount of characters needing to be printed to the terminal.

- 1-7-2025:
> So far, I am quite happy: [Check out the demos (based on a slower version)!](https://www.youtube.com/playlist?list=PLvkXEUKaigSyHm_Q2-Cmmdko0IKtBOLpU)


## Basic Usage:
1. Clone the repo
    > ```bash
    > git clone https://github.com/cmorman89/rusty_hemera_mini
    > ```
2. Change to the project directory
    > ```bash
    > cd rusty_hemera_mini
    > ```
3. Install the dependencies
    > Install Rust via RustUp (Linux): 
    > ```bash
    > curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    > ```

    > Install Rust via RustUp (Windows):
    > ```powershell
    > Invoke-WebRequest https://win.rustup.rs -OutFile rustup-init.exe; .\rustup-init.exe
    > ```

    > Install ffmpeg-devel (or equivalent) via your package manager
    > - I will provide better instructions for this in the future

3. Change the video_path in `main.rs` to the path of the video you want to play
    > ```rust
    > // Path to the video file
    > let video_path = "asset/small_2.mp4";
    > ```

4. Run the program
    > ```bash
    > cargo run --release
    > ```
5. Enjoy the video!

> [!NOTE]
> - Be sure to zoom out of the terminal to see the full video!
> - Use Ctrl+C to stop the video
> - Try to limit the input video to 400x200 @ 30fps for the best results

## Dependencies:

### Rust Cargo Packages:
  - ffmpeg-next
  - ndarray

### System Dependencies:
  - ffmpeg-devel (or equivalent)
  > [!WARNING]
  > This was a bit of a pain to install on Fedora. I had to install the RPM Fusion repos and then install ffmpeg-devel with flags set to allow erasing existing packages due to version issues. I am not sure if this is the best way to do it, but it worked for me.
  >
  > I'd Google it instead of following my advice on this one.

## To Do Ideas:
- Frame-by-frame change detection:
    - Delta buffer to only update changed cells
    - Run buffers for ANSI escape caching
    - Consider quantiziation methods to limit small variations in color
- Parallelize the rendering process
    - Use Rayon for parallelization
    - Use SIMD for pixel processing
    - Split the image into chunks for parallel processing?
    - Split the ndarray into chunks for parallel processing?
- ~Check if ndarray is the right data structure for holding RGB values~
    - Seem to be the best option for now, as tracking a Vec index was visibly slower
- Look at SIMD for pixel processing of u8 -> bytes
    - Does ndarray automatically use SIMD? If not, how to enable it?
- Look at mapping u8 -> bytes
    - Currently moved to inline arithmetic
    - Maybe use a macro, though I have heard they are complex to make in Rust
    - Whole-ndarray operations to convert to bytes in one go?
- Learn error handling in Rust.expect(😭)