use crate::constants::*;
use dsp;
use rustfft::{num_complex::Complex, FftPlanner};
use std::fs::File;

#[tauri::command]
pub async fn get_time_onefft(
    path: &str,
    app_handle: tauri::AppHandle,
) -> Result<(Vec<f32>, Vec<f32>), &str> {
    let mut time_data = vec![];
    let mut freq_data = vec![];

    let p = app_handle
        .path_resolver()
        .resolve_resource(ASSETS_PATH)
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let filepath = p + "/" + path;

    let thread = tauri::async_runtime::spawn(async move {
        let file_in = File::open(filepath).unwrap();
        let (_head, w) = wav_io::read_from_file(file_in).unwrap();
        let mut buffer = vec![];
        let len = w.len();
        for s in w.clone() {
            let x = s;
            buffer.push(Complex { re: x, im: 0.0f32 })
        }
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(len);

        fft.process(&mut buffer);
        return (
            w,
            buffer[0..len / 2]
                .iter()
                .map(|x| x.norm())
                .collect::<Vec<f32>>(),
        );
    });

    if let Ok(r) = thread.await {
        (time_data, freq_data) = r;
    }

    Ok((time_data, freq_data))
}

#[tauri::command]
pub async fn get_stft_data(
    path: &str,
    app_handle: tauri::AppHandle,
) -> Result<(Vec<f32>, Vec<Vec<f32>>), &str> {
    let mut vstft: Vec<Vec<f32>> = vec![];
    let mut time_data = vec![];

    let p = app_handle
        .path_resolver()
        .resolve_resource(ASSETS_PATH)
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let filepath = p + "/" + path;

    let thread = tauri::async_runtime::spawn(async move {
        let file_in = File::open(filepath).unwrap();
        let (head, w) = wav_io::read_from_file(file_in).unwrap();
        return w;
    });

    if let Ok(r) = thread.await {
        time_data = r;
    }

    // if let Ok(mut wav) = Wav::from_path(p + "/" + path) {
    //     let itr: Vec<f32> = wav.read().unwrap().to_vec();
    //     let mut buffer = vec![];
    //     let len = itr.len();

    //     for s in itr {
    //         let x = s;
    //         time_data.push(x.clone());
    //         buffer.push(Complex { re: x, im: 0.0f32 })
    //     }

    //     let fftsize = 2048;
    //     let vstft = stft(buffer.clone(), fftsize, fftsize / 2);

    //     return Ok((time_data, vstft));
    // } else {
    //     return Err("bad path");
    // }

    Ok((time_data, vstft))
}

pub fn stft(mut buffer: Vec<Complex<f32>>, size: usize, hop: usize) -> Vec<Vec<f32>> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(size);

    let win = dsp::window::hamming(size);
    let frame = vec![1.0; size];
    let mut output = vec![0.0; size];
    win.apply(&frame, &mut output);

    let l = buffer.len();
    let num_slices = l / (size + hop);
    let mut spectra: Vec<Vec<f32>> = vec![];
    let mut last_spectrum = vec![Complex { re: 0.0, im: 0.0 }; size];
    for slice in 0..num_slices {
        let mut x = vec![Complex { re: 0.0, im: 0.0 }; size];
        for (i, samp) in buffer[slice * size + hop * slice..(slice + 1) * size + hop * slice]
            .iter()
            .enumerate()
        {
            x[i] = (samp * win.samples[i] + last_spectrum[i]) / 2.0;
        }

        // last_spectrum = x.clone();

        fft.process(&mut x);

        let mut v = vec![];
        for i in x[0..size / 2].iter() {
            v.push(i.norm());
        }
        spectra.push(v);
    }

    spectra
}

pub fn mfft(mut signal: Vec<f32>) -> Vec<f32> {
    let len = signal.len();
    let mut buffer = vec![];
    for s in signal.iter() {
        buffer.push(Complex { re: *s, im: 0.0f32 })
    }
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(len);

    fft.process(&mut buffer);
    buffer[0..len / 2]
        .iter()
        .map(|x| x.norm())
        .collect::<Vec<f32>>()
}
