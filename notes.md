# Ideas, notes, musings, code examples, etc.

> !WARNING
> Probably incohorent ramblings and springboard ideas. Just a place to dump thoughts that are probably helpful or at least remind me of something helpful.

---

## Delta comparison Ideas
- [ ] Process inline into byte buffer or as a matrix and then into byte buffer?
    > The inline processing is faster, but the matrix processing is more readable and maintainable
    > The matrix processing is also more flexible for future changesl

- [ ] Use different digit mappings to represent different quantiziation levels
    > This would allow for more demanding quantiziation levels to be used for more complex videos as the tables are precomputed
    > Dynamically swap the tables based on the quantiziation level desired
    > Dynamically adjust the quantiziation level based on loop timing vs target timing (fps)

- [ ] Further, create separate R, G, B mappings for different quantiziation levels
    > Human vision does not perceive all colors equally. It is most sensitive to changes in green, then red, then blue.
    > Different tables allows different steps between each channel to be used
    > The benefit is that we can make complex quantization levels without needing to calculate the values on the fly
    > The granularity allows a more accurate representation of the video when the system is bogging down
    > The downside is that it requires more memory to store the tables -- but the u8 space is small
    >
    >
    > ---
    >
    > Example perceptual quantization function:
    > ```rust
    > fn perceptual_quantization(value: u8, sensitivity_curve: &[f32; 256]) -> u8 {
    > let mut quantized_value = 0;
    > let step = 1.0 / (sensitivity_curve.len() as f32);
    > while quantized_value < sensitivity_curve.len() {
    >     if value as f32 <= sensitivity_curve[quantized_value] {
    >         return (quantized_value as u8) * (256 / sensitivity_curve.len() as u8);
    >     }
    >     quantized_value += 1;
    > }
    > 255
    > }
    > 
    > // Example sensitivity curves for R, G, B (hypothetical)
    > static RED_CURVE: [f32; 256] = [/* precomputed perceptual values */];
    > static GREEN_CURVE: [f32; 256] = [/* precomputed perceptual values */];
    > static BLUE_CURVE: [f32; 256] = [/* precomputed perceptual values */];
    > 
    > // Precompute tables based on perceptual curves
    > static R_PERCEPTUAL_TABLE: [u8; 256] = generate_perceptual_table(&RED_CURVE);
    > static G_PERCEPTUAL_TABLE: [u8; 256] = generate_perceptual_table(&GREEN_CURVE);
    > static B_PERCEPTUAL_TABLE: [u8; 256] = generate_perceptual_table(&BLUE_CURVE);
    > ```
    >
    > ---
    >
    > Making 8-bit through 4-bit linear tables:
    > ```rust
    > /// Generate a quantization table for a specified number of levels (not necessarily 2^n).
    > fn generate_non_2n_table(levels: usize) -> [u8; 256] {
    >     let mut table = [0u8; 256];
    >     let step = 256 / levels; // Mapping values from 0 to 255 into `levels` levels.
    >     for i in 0..256 {
    >         // Use flooring or rounding to map the value to one of the levels
    >         table[i] = (i / step) as u8 * step as u8;
    >     }
    >     table
    > }
    > 
    > const R_7BIT: [u8; 256] = generate_non_2n_table(128); // 7-bit levels for red (128 levels)
    > const G_6BIT: [u8; 256] = generate_non_2n_table(64);  // 6-bit levels for green (64 levels)
    > const B_5BIT: [u8; 256] = generate_non_2n_table(32);  // 5-bit levels for blue (32 levels)
    > 
    > /// Function to apply quantization to RGB channels using custom bit depths for each channel.
    > fn quantize_rgb(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    >     (
    >         R_7BIT[r as usize],  // 7-bit quantization for red
    >         G_6BIT[g as usize],  // 6-bit quantization for green
    >         B_5BIT[b as usize],  // 5-bit quantization for blue
    >     )
    > }
    > 
    > // Example usage
    > let red = 123;
    > let green = 200;
    > let blue = 77;
    > 
    > let (qr, qg, qb) = quantize_rgb(red, green, blue);
    > println!("Quantized RGB: ({}, {}, {})", qr, qg, qb);


## Rasterization ideas
- [ ] Make a lookup table of u8 -> b'u8' for the color channel values
    > Since it is only the u8 space, it takes very little memory to store the mappings
    > This would replace the per channel arithmetic operation: 
    >
    > `byte_buffer.extend_from_slice(&[fg_r / 100 + b'0', (fg_r % 100) / 10 + b'0', fg_r % 10 + b'0']);`
    >
    > Map the u8 value to the byte value:
    > ```rust
    > static DIGIT_MAPPINGS: [[u8; 3]; 256] = {
    > let mut mappings = [[b'0', b'0', b'0']; 256];
    > let mut i = 0;
    > while i < 256 {
    >     mappings[i] = [
    >         i / 100 + b'0',
    >         (i % 100) / 10 + b'0',
    >         i % 10 + b'0',
    >     ];
    >     i += 1;
    > }
    > mappings
    > };
    > ```

- [ ] Convert the delta buffer to bytes for as a matrix operation
    > This would allow for a single operation to convert the u8 values to the byte values
    > Use the mapping to convert the u8 value to the byte value:
    > ```rust
    > let shape = (4, 4, 4);
    > let input_array: Array3<u8> = Array3::from_shape_fn(shape, |(i, j, k)| ((i + j + k) % 256) as u8);
    > let mut output_array: Array4<u8> = Array4::zeros((shape.0, shape.1, shape.2, 3));
    >
    > azip!((val in &input_array, mut out_lane in output_array.lanes_mut(ndarray::Axis(3))) {
    >     let mapping = DIGIT_MAPPINGS[val as usize];
    >     out_lane[0] = mapping[0];
    >     out_lane[1] = mapping[1];
    >     out_lane[2] = mapping[2];
    > });
    > ```

- [ ] Conversely, use the map to append directly to the byte buffer (better for performance)
    > This would allow for a single operation to convert the u8 values to the byte values
    > Use the mapping to convert the u8 value to the byte value:
    > ```rust
    > let shape = (4, 4, 4);
    > let input_array: Array3<u8> = Array3::from_shape_fn(shape, |(i, j, k)| ((i + j + k) % 256) as u8);
    > let mut byte_buffer = Vec::with_capacity(shape.0 * shape.1 * shape.2 * 3);
    >
    > input_array.iter().for_each(|&val| {
    >     let mapping = DIGIT_MAPPINGS[val as usize];
    >     byte_buffer.extend_from_slice(&mapping);
    > });
    > ```