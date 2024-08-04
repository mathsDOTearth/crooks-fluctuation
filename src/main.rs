// Rust Rayon multithreaded Crooks Fluctuation Theorem simulation
// by maths.earth 2024
// https://en.wikipedia.org/wiki/Crooks_fluctuation_theorem

mod unirand;

use image::ImageBuffer;
use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;
use std::f64::consts::PI;
use unirand::RNG;

// Constants for image dimensions
const WIDTH: usize = 1024;
const HEIGHT: usize = 768;

// Function to compute the Crooks fluctuation theorem
fn crooks_fluctuation_theorem(terms: u32, coefficient: f64, exponent: f64, time: f64) -> f64 {
    let mut sum = 0.0;
    for i in 1..=terms {
        let term = (2.0 * PI * i as f64 + time).sin() / (2.0 * PI * i as f64 + time).cosh();
        sum += (coefficient * term).powf(exponent);
    }
    sum
}

fn main() {
    // Create a new window
    let mut window = Window::new(
        "Crooks Fluctuation Theorem Simulation",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let terms = 100;
    let coefficient = 2.0;
    let exponent = 3.0;
    let mut time = 0.0;
    let time_step = 0.05;
    let scale_factor = 1e3; // Adjusted scale factor for better variability

    // Main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut image = ImageBuffer::new(WIDTH as u32, HEIGHT as u32);

        // Compute the colour values for each pixel in parallel
        image.enumerate_pixels_mut().par_bridge().for_each(|(x, y, pixel)| {
            let value = crooks_fluctuation_theorem(terms, coefficient, exponent, time + (x as f64) / 100.0 + (y as f64) / 100.0) * scale_factor;

            // Use custom RNG for random factors and convert them to f64
            let random_factor_r = RNG.with(|rng| rng.borrow_mut().generate() as f64);
            let random_factor_g = RNG.with(|rng| rng.borrow_mut().generate() as f64);
            let random_factor_b = RNG.with(|rng| rng.borrow_mut().generate() as f64);
            let normalized_value = value.sin() * 0.5 + 0.5;

            // Enhanced colour mapping with different random factors for each colour channel
            let red = (normalized_value * random_factor_r * 255.0) as u8;
            let green = ((1.0 - normalized_value) * random_factor_g * 255.0) as u8;
            let blue = ((0.5 - (normalized_value - 0.5).abs()) * 2.0 * random_factor_b * 255.0) as u8;

            // Set pixel data
            *pixel = image::Rgb([red, green, blue]);
        });

        // Create a buffer to display the image in the window
        let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
        for (x, y, pixel) in image.enumerate_pixels() {
            let red = pixel[0] as u32;
            let green = pixel[1] as u32;
            let blue = pixel[2] as u32;
            let colour = (red << 16) | (green << 8) | blue;
            buffer[y as usize * WIDTH + x as usize] = colour;
        }

        // Update the window with the new image
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        time += time_step;
    }
}
