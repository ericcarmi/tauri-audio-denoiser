use cpal::traits::StreamTrait;
use samplerate::{convert, ConverterType};
use std::{fs::File, path::PathBuf, str::FromStr};
use tauri::{AppHandle, State, Window};

use crate::{
    audio::device_sample_rate,
    constants::{from_log, DOWN_RATE},
    sql::{query_filter_bank, query_ui_params},
    types::{MStreamSend, StereoChoice, StereoParams},
};
#[tauri::command]
pub async fn get_time_data(
    path: &str,
    app_handle: tauri::AppHandle,
) -> Result<Vec<f32>, &'static str> {
    let mut time_data = vec![];

    let device_sample_rate = device_sample_rate().unwrap();

    let filepath = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("~AppData\\Local\\denoiser should exist")
        .join(path);
    // path really being used as file_name

    let thread = tauri::async_runtime::spawn(async move {
        if let Ok(f) = File::open(filepath) {
            let (head, samples) = wav_io::read_from_file(f).unwrap();

            // check sample rate, if it doesn't match device_rate, convert
            if device_sample_rate != cpal::SampleRate(head.sample_rate) {
                println!("resampling begin");

                let resampled = convert(
                    head.sample_rate,
                    device_sample_rate.0,
                    1,
                    ConverterType::SincBestQuality,
                    &samples,
                )
                .unwrap();
                println!("resampling end");
                return resampled
                    .iter()
                    .step_by(DOWN_RATE)
                    .cloned()
                    .collect::<Vec<f32>>();
            } else {
                return samples
                    .iter()
                    .step_by(DOWN_RATE)
                    .cloned()
                    .collect::<Vec<f32>>();
            }
        }
        vec![]
    });

    if let Ok(r) = thread.await {
        time_data = r;
    }
    // } else {
    //     let filepath = path.to_owned();

    //     let thread = tauri::async_runtime::spawn(async move {
    //         if let Ok(f) = File::open(PathBuf::from(filepath)) {
    //             let (_head, samples) = wav_io::read_from_file(f).unwrap();

    //             return samples
    //                 .iter()
    //                 .step_by(DOWN_RATE)
    //                 .cloned()
    //                 .collect::<Vec<f32>>();
    //         }
    //         vec![]
    //     });

    //     if let Ok(r) = thread.await {
    //         time_data = r;
    //     }
    // }
    if time_data.is_empty() {
        return Err("failed to get time data");
    }

    Ok(time_data)
}

#[tauri::command]
pub async fn process_export(
    streamsend: State<'_, MStreamSend>,
    app_handle: AppHandle,
    file_path: String,
    stereo_choice: StereoChoice,
    window: Window,
) -> Result<(), String> {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .stream
        .0
        .lock()
        .unwrap()
        .pause();

    // file samples are not an audio param, stream is remade when file is changed so this stays
    let file_samples;
    let is_stereo;
    let head;
    // need to update this
    let p = PathBuf::from_str(file_path.as_str()).expect("bad path");
    // (file_samples, is_stereo) = get_wav_samples(p);
    let f = File::open(p).unwrap();
    (head, file_samples) = wav_io::read_from_file(f).unwrap();
    is_stereo = if head.channels == 1 { false } else { true };

    let db_path = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    // query db

    let mut stereo_params = StereoParams::new();
    stereo_params.is_stereo = is_stereo;
    stereo_params.num_file_samples = file_samples.len();

    match stereo_choice {
        StereoChoice::Both => {
            let filter_bank = query_filter_bank(stereo_choice, &db_path);
            let ui_params = query_ui_params(stereo_choice, &db_path);
            if filter_bank.is_err() || ui_params.is_err() {
                // return something
            }
            let fb = filter_bank.unwrap();
            stereo_params.left.filters = fb.clone().into();
            stereo_params.right.filters = fb.into();
            stereo_params.left.noise_spectrum = stereo_params.left.filters.parallel_transfer(256);
            stereo_params.right.noise_spectrum = stereo_params.right.filters.parallel_transfer(256);
            let p = ui_params.unwrap();
            stereo_params.left.ui_params.noise_gain = from_log(p.noise_gain);
            stereo_params.left.ui_params.output_gain = from_log(p.output_gain);
            stereo_params.left.ui_params.pre_smooth_gain = p.pre_smooth_gain;
            stereo_params.left.ui_params.post_smooth_gain = p.post_smooth_gain;
            stereo_params.right.ui_params.noise_gain = from_log(p.noise_gain);
            stereo_params.right.ui_params.output_gain = from_log(p.output_gain);
            stereo_params.right.ui_params.pre_smooth_gain = p.pre_smooth_gain;
            stereo_params.right.ui_params.post_smooth_gain = p.post_smooth_gain;
        }
        _ => {
            let left_bank = query_filter_bank(StereoChoice::Left, &db_path);
            let right_bank = query_filter_bank(StereoChoice::Right, &db_path);
            let left_ui_params = query_ui_params(StereoChoice::Left, &db_path);
            let right_ui_params = query_ui_params(StereoChoice::Right, &db_path);
            if left_bank.is_err()
                || left_ui_params.is_err()
                || right_bank.is_err()
                || right_ui_params.is_err()
            {
                // return something
            }
            stereo_params.left.filters = left_bank.unwrap().into();
            stereo_params.right.filters = right_bank.unwrap().into();
            stereo_params.left.noise_spectrum = stereo_params.left.filters.parallel_transfer(256);
            stereo_params.right.noise_spectrum = stereo_params.right.filters.parallel_transfer(256);
            let lu = left_ui_params.unwrap();
            let ru = right_ui_params.unwrap();

            stereo_params.left.ui_params.noise_gain = from_log(lu.noise_gain);
            stereo_params.left.ui_params.output_gain = from_log(lu.output_gain);
            stereo_params.left.ui_params.pre_smooth_gain = lu.pre_smooth_gain;
            stereo_params.left.ui_params.post_smooth_gain = lu.post_smooth_gain;
            stereo_params.right.ui_params.noise_gain = from_log(ru.noise_gain);
            stereo_params.right.ui_params.output_gain = from_log(ru.output_gain);
            stereo_params.right.ui_params.pre_smooth_gain = ru.pre_smooth_gain;
            stereo_params.right.ui_params.post_smooth_gain = ru.post_smooth_gain;
        }
    }
    // println!("{:?}", stereo_params);

    let _ = window.emit("update_processing_percentage", 0.0);

    let num_samples = stereo_params.num_file_samples;

    let thread = tauri::async_runtime::spawn(async move {
        let mut samples = vec![];
        if !stereo_params.is_stereo {
            for time in 0..num_samples {
                let sample = file_samples[time] * stereo_params.left.ui_params.output_gain;
                let filtered = stereo_params.left.sdft.spectral_subtraction(
                    sample,
                    &stereo_params.left.noise_spectrum,
                    stereo_params.left.ui_params.noise_gain,
                    stereo_params.left.ui_params.pre_smooth_gain,
                    stereo_params.left.ui_params.post_smooth_gain,
                );
                samples.push(filtered);
                samples.push(filtered);
            }
        }
        // PROCESS STEREO
        else {
            for time in 0..num_samples / 2 - 1 {
                if time % 4410 == 0 {
                    let _r = window.emit(
                        "update_processing_percentage",
                        time as f32 / num_samples as f32 * 2.0 * 100.0,
                    );
                }

                let left_sample = file_samples[2 * time] * stereo_params.left.ui_params.output_gain;
                let left_filtered = stereo_params.left.sdft.spectral_subtraction(
                    left_sample,
                    &stereo_params.left.noise_spectrum,
                    stereo_params.left.ui_params.noise_gain,
                    stereo_params.left.ui_params.pre_smooth_gain,
                    stereo_params.left.ui_params.post_smooth_gain,
                );

                samples.push(left_filtered);

                let right_sample =
                    file_samples[2 * time + 1] * stereo_params.right.ui_params.output_gain;
                let right_filtered = stereo_params.right.sdft.spectral_subtraction(
                    right_sample,
                    &stereo_params.right.noise_spectrum,
                    stereo_params.right.ui_params.noise_gain,
                    stereo_params.right.ui_params.pre_smooth_gain,
                    stereo_params.right.ui_params.post_smooth_gain,
                );
                samples.push(right_filtered);
            }
        };
        samples
    });

    if let Ok(r) = thread.await {
        let samples: Vec<f32> = r;
        if samples.is_empty() {
            return Err("empty samples, failed to write to file".to_string());
        }
        let p = app_handle
            .path_resolver()
            .app_local_data_dir()
            .expect("~AppData\\Local\\denoiser should exist");

        let header = wav_io::new_stereo_header();
        if let Ok(mut file) = File::create(p.join("denoised.wav")) {
            let _r = wav_io::write_to_file(&mut file, &header, &samples);
        };
        Ok(())
    } else {
        Err("failed to write to file".to_string())
    }
}
