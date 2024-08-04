// unirand.rs

use std::cell::RefCell;

const LEN_U: usize = 98;

// Marsaglia's Universal Random Number Generator (RNG) structure
pub struct MarsagliaUniRng {
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
    pub static RNG: RefCell<MarsagliaUniRng> = RefCell::new({
        let mut rng = MarsagliaUniRng::new();
        rng.initialise(12345); // Initialise with a seed value
        rng
    });
}
