// Rust Rayon multithreaded Crooks Fluctuation Theorem simulation
// by maths.earth 2024
// https://en.wikipedia.org/wiki/Crooks_fluctuation_theorem

use image::ImageBuffer;
use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;
use std::f64::consts::PI;
use std::cell::RefCell;

// Constants for image dimensions and random number generator array length
const WIDTH: usize = 1024;
const HEIGHT: usize = 768;
const LEN_U: usize = 98;

// Marsaglia's Universal Random Number Generator (RNG) structure
struct MarsagliaUniRng {
    recent_values: [f32; LEN_U], // Array holding the recent random numbers
    correction: f32,             // Correction to avoid periodicity
    correction_delta: f32,       // Correction delta value
    correction_modulus: f32,     // Correction modulus
    current_index: usize,        // Current position in the random values array
    second_index: usize,
}

impl MarsagliaUniRng {
    // Constructor for the random number generator
    pub fn new() -> Self {
        Self {
            recent_values: [0.0; LEN_U],
            correction: 0.0,
            correction_delta: 0.0,
            correction_modulus: 0.0,
            current_index: 0,
            second_index: 0,
        }
    }

    // Generate a new random float value between 0 and 1
    pub fn generate(&mut self) -> f32 {
        let mut new_value = self.recent_values[self.current_index] - self.recent_values[self.second_index];
        if new_value < 0.0 {
            new_value += 1.0;
        }
        self.recent_values[self.current_index] = new_value;

        if self.current_index == 0 {
            self.current_index = 97;
        } else {
            self.current_index -= 1;
        }
        if self.second_index == 0 {
            self.second_index = 97;
        } else {
            self.second_index -= 1;
        }

        self.correction -= self.correction_delta;
        if self.correction < 0.0 {
            self.correction += self.correction_modulus;
        }

        new_value -= self.correction;
        if new_value < 0.0 {
            new_value += 1.0;
        }
        new_value
    }

    // Initialise the random values array using four seeds
    pub fn start(&mut self, seed1: i32, seed2: i32, seed3: i32, seed4: i32) {
        let mut i = seed1;
        let mut j = seed2;
        let mut k = seed3;
        let mut l = seed4;
        for ii in 1..=97 {
            let mut s = 0.0;
            let mut t = 0.5;
            for _jj in 1..=24 {
                let m = ((i * j % 179) * k) % 179;
                i = j;
                j = k;
                k = m;
                l = (53 * l + 1) % 169;
                if l * m % 64 >= 32 {
                    s += t;
                }
                t *= 0.5;
            }
            self.recent_values[ii] = s;
        }
        self.correction = 362436.0 / 16777216.0;
        self.correction_delta = 7654321.0 / 16777216.0;
        self.correction_modulus = 16777213.0 / 16777216.0;
        self.current_index = 97;
        self.second_index = 33;
    }

    // Validate and decompose a single seed into four seeds, then initialise the random values array
    pub fn initialise(&mut self, seed: i32) {
        if seed < 0 || seed > 900_000_000 {
            panic!("initialise: seed = {} -- out of range", seed);
        }

        let ij = seed / 30082;
        let kl = seed - (30082 * ij);
        let i = ((ij / 177) % 177) + 2;
        let j = (ij % 177) + 2;
        let k = ((kl / 169) % 178) + 1;
        let l = kl % 169;

        if i <= 0 || i > 178 {
            panic!("initialise: i = {} -- out of range", i);
        }
        if j <= 0 || j > 178 {
            panic!("initialise: j = {} -- out of range", j);
        }
        if k <= 0 || k > 178 {
            panic!("initialise: k = {} -- out of range", k);
        }
        if l < 0 || l > 168 {
            panic!("initialise: l = {} -- out of range", l);
        }
        if i == 1 && j == 1 && k == 1 {
            panic!("initialise: 1 1 1 not allowed for first 3 seeds");
        }

        self.start(i, j, k, l);
    }
}

// Thread-local storage for the random number generator
thread_local! {
    static RNG: RefCell<MarsagliaUniRng> = RefCell::new({
        let mut rng = MarsagliaUniRng::new();
        rng.initialise(12345); // Initialise with a seed value
        rng
    });
}

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
