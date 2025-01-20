pub mod audio;
pub mod constants;
pub mod errors;
pub mod file_io;
pub mod fourier;
pub mod messages;
pub mod sdft;
pub mod settings;
pub mod sql;
pub mod types;

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::*;
    use constants::from_log;
    // use crate::sdft::*;
    use dasp_ring_buffer as ringbuff;
    use rustfft::num_complex::Complex;
    use sql::{query_filter_bank, query_ui_params};
    use types::*;

    #[test]
    fn sdft() {
        let n = 10;
        let mut x = ringbuff::Fixed::from(vec![Complex { re: 0.0, im: 0.0 }; n]);
        for i in 0..n {
            x.push(Complex {
                re: i as f32,
                im: 0.0,
            });
        }
        for _i in 0..n {
            println!("{:?}", x.get(0));
            x.push(Complex { re: 1.0, im: 0.0 });
        }
        for _i in 0..n {
            println!("{:?}", x.get(0));
        }
    }

    // need to make sure values are not NaN...that's happening somehow with IIR2
    #[test]
    fn make_freq_response() {
        let mut params = StereoParams::new();
        let p = PathBuf::from_str("C:\\Users\\eric\\denoiser\\src-tauri\\target\\debug")
            .expect("failed to make path from str")
            .join("db.sqlite");
        let left_ui_params = query_ui_params(StereoChoice::Both, &p);
        let lu = left_ui_params.unwrap();
        let filter_bank = query_filter_bank(StereoChoice::Both, &p);
        let fb = filter_bank.unwrap();
        let filters: Filters = fb.into();

        println!("{:?}", fb.bank);
        println!("{:?}", filters);

        params.left.ui_params.noise_gain = from_log(lu.noise_gain);
        params.left.ui_params.output_gain = from_log(lu.output_gain);
        params.left.ui_params.pre_smooth_gain = lu.pre_smooth_gain;
        params.left.ui_params.post_smooth_gain = lu.post_smooth_gain;
        // println!("{:?}", params);
    }
}
