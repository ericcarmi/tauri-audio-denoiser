#[cfg(test)]
mod tests {
    use super::*;
    // use crate::sdft::*;
    use dasp_ring_buffer as ringbuff;
    use rustfft::num_complex::Complex;

    #[test]
    fn sdft() {
        let N = 10;
        let mut x = ringbuff::Fixed::from(vec![Complex { re: 0.0, im: 0.0 }; N]);
        for i in 0..N {
            x.push(Complex {
                re: i as f32,
                im: 0.0,
            });
        }
        for i in 0..N {
            println!("{:?}", x.get(0));
            x.push(Complex { re: 1.0, im: 0.0 });
        }
        for i in 0..N {
            println!("{:?}", x.get(0));
        }
    }
}
