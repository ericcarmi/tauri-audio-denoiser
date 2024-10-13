use crate::messages::{AudioUIMessage, UIAudioMessage};
use crate::{averaged_stft, types::*};
use crate::{constants::*, stft};
use anyhow;
use cpal::FromSample;
use cpal::{self};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    SizedSample,
};
use dsp::num_complex::Complex;
use samplerate::{convert, ConverterType};
use std::fs::File;
use std::path::PathBuf;
use tauri::{AppHandle, Window};

pub fn get_resource_wav_samples(path: &str, app_handle: AppHandle) -> (Vec<f32>, bool) {
    let p = app_handle
        .path_resolver()
        .resolve_resource(path)
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    println!("{:?}", p);
    let file_in = File::open(p).unwrap();
    let (head, samples) = wav_io::read_from_file(file_in).unwrap();
    let is_stereo = if head.channels == 1 { false } else { true };
    let device_sample_rate = device_sample_rate().unwrap();

    if device_sample_rate != cpal::SampleRate(head.sample_rate) {
        // sample rates should not be constant numbers
        let resampled = convert(44100, 48000, 1, ConverterType::SincBestQuality, &samples).unwrap();
        (resampled, is_stereo)
    } else {
        (samples, is_stereo)
    }
}

pub fn get_wav_samples(path: PathBuf) -> (Vec<f32>, bool) {
    // if let Ok(file_in) = File::open(path) {
    //     let (head, samples) = wav_io::read_from_file(file_in).unwrap();
    //     let is_stereo = if head.channels == 1 { false } else { true };
    //     (samples, is_stereo)
    // } else {
    let f = File::open(path).unwrap();
    let (head, samples) = wav_io::read_from_file(f).unwrap();
    let is_stereo = if head.channels == 1 { false } else { true };
    let device_sample_rate = device_sample_rate().unwrap();
    if device_sample_rate != cpal::SampleRate(head.sample_rate) {
        let resampled = convert(44100, 48000, 1, ConverterType::SincBestQuality, &samples).unwrap();
        (resampled, is_stereo)
    } else {
        (samples, is_stereo)
    }
    // }
}

#[tauri::command]
pub fn setup_stream(
    tx: tauri::async_runtime::Sender<AudioUIMessage>,
    app_handle: AppHandle,
    file_path: Option<String>,
    window: Window,
) -> Result<(cpal::Stream, tauri::async_runtime::Sender<UIAudioMessage>), anyhow::Error>
where
{
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::I8 => {
            make_stream::<i8>(&device, &config.into(), tx, app_handle, file_path, window)
        }
        cpal::SampleFormat::I16 => {
            make_stream::<i16>(&device, &config.into(), tx, app_handle, file_path, window)
        }
        cpal::SampleFormat::I32 => {
            make_stream::<i32>(&device, &config.into(), tx, app_handle, file_path, window)
        }
        cpal::SampleFormat::I64 => {
            make_stream::<i64>(&device, &config.into(), tx, app_handle, file_path, window)
        }
        cpal::SampleFormat::U8 => {
            make_stream::<u8>(&device, &config.into(), tx, app_handle, file_path, window)
        }
        cpal::SampleFormat::U16 => {
            make_stream::<u16>(&device, &config.into(), tx, app_handle, file_path, window)
        }
        cpal::SampleFormat::U32 => {
            make_stream::<u32>(&device, &config.into(), tx, app_handle, file_path, window)
        }
        cpal::SampleFormat::U64 => {
            make_stream::<u64>(&device, &config.into(), tx, app_handle, file_path, window)
        }
        cpal::SampleFormat::F32 => {
            make_stream::<f32>(&device, &config.into(), tx, app_handle, file_path, window)
        }
        cpal::SampleFormat::F64 => {
            make_stream::<f64>(&device, &config.into(), tx, app_handle, file_path, window)
        }
        sample_format => Err(anyhow::Error::msg(format!(
            "Unsupported sample format '{sample_format}'"
        ))),
    }
}

pub fn host_device_setup(
) -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;

    // let s = device.supported_output_configs();
    // if let Ok(c) = s {
    //     for i in c {
    //         println!("{:?}", i);
    //     }
    //     // let mut  = c;
    // }

    let conf = device.default_output_config()?;
    // let config = cpal::SupportedStreamConfig::new(
    //     conf.channels(),
    //     cpal::SampleRate(48000),
    //     *conf.buffer_size(),
    //     conf.sample_format(),
    // );

    Ok((host, device, conf))
}

pub fn device_sample_rate() -> Result<cpal::SampleRate, anyhow::Error> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    let conf = device.default_output_config()?;
    Ok(conf.sample_rate())
}

pub fn make_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    tx_ui: tauri::async_runtime::Sender<AudioUIMessage>,
    app_handle: AppHandle,
    file_path: Option<String>,
    window: Window,
) -> Result<(cpal::Stream, tauri::async_runtime::Sender<UIAudioMessage>), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let num_channels = config.channels as usize;
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let (tx, mut rx) = tauri::async_runtime::channel::<UIAudioMessage>(1);

    // variables that stream will use, including params
    let file_samples;
    let is_stereo;
    if let Some(f) = file_path.clone() {
        let p = app_handle.path_resolver().resource_dir().unwrap().join(f);
        (file_samples, is_stereo) = get_wav_samples(p);
    } else {
        let p = app_handle
            .path_resolver()
            .resource_dir()
            .unwrap()
            .join("assets")
            .join(TEST_FILE);
        (file_samples, is_stereo) = get_resource_wav_samples(TEST_FILE, app_handle.clone());
    }
    let mut stereo_params = StereoParams::new();
    stereo_params.is_stereo = is_stereo;
    let _ = tx_ui.try_send(AudioUIMessage {
        is_stereo: Some(is_stereo),
        ..Default::default()
    });
    stereo_params.num_file_samples = file_samples.len();
    let _ = window
        .clone()
        .emit("update_sampling_rate", config.sample_rate.0);

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            if let Ok(msg) = rx.try_recv() {
                msg.receive(&mut stereo_params);
                if msg.fingerprint.is_some() {
                    println!("fingerprint time");

                    // call the function from here...then what
                    // it sends the spectrum and changes the the eq params -- return these from fxn
                    // send a chunk of the data, along with stereo_choice
                    // get spectrum of the noise
                    // LMS min error to get eq params
                    // calculate_fingerprint(file_samples)
                }
            }

            if !stereo_params.is_stereo {
                let mut spectrum: Vec<f32> = vec![];
                if stereo_params.clean {
                    // ...each frame has 2 samples
                    for frame in output.chunks_mut(num_channels) {
                        if stereo_params.is_looping
                            && stereo_params.time
                                > stereo_params.loop_start_time + stereo_params.loop_length
                        {
                            stereo_params.time = stereo_params.loop_start_time;
                        }
                        if stereo_params.time >= stereo_params.num_file_samples {
                            break;
                        }
                        let sample = file_samples[stereo_params.time]
                            * stereo_params.left.ui_params.output_gain;
                        let v: T = T::from_sample(sample);
                        spectrum.push(sample);

                        // copy mono input to both output channels
                        for out_sample in frame.iter_mut() {
                            *out_sample = v;
                        }
                        stereo_params.time += 1;
                    }
                } else {
                    for frame in output.chunks_mut(num_channels) {
                        if stereo_params.is_looping
                            && stereo_params.time
                                > stereo_params.loop_start_time + stereo_params.loop_length
                        {
                            stereo_params.time = stereo_params.loop_start_time;
                        }
                        if stereo_params.time >= stereo_params.num_file_samples {
                            break;
                        }
                        let sample = file_samples[stereo_params.time]
                            * stereo_params.left.ui_params.output_gain;
                        let filtered = stereo_params.left.sdft.spectral_subtraction(
                            sample,
                            &stereo_params.left.noise_spectrum,
                            stereo_params.left.ui_params.noise_gain,
                            stereo_params.left.ui_params.pre_smooth_gain,
                            stereo_params.left.ui_params.post_smooth_gain,
                        );

                        let v: T = T::from_sample(filtered);
                        spectrum.push(filtered);

                        // copying to all channels for now
                        for out_sample in frame.iter_mut() {
                            *out_sample = v;
                        }
                        stereo_params.time += 1;
                    }
                }

                let _ = window.emit(
                    AudioUIMessage::name(),
                    AudioUIMessage {
                        spectrum: Some(
                            stereo_params.left.sdft.norm_vec()[0..stereo_params.left.sdft.size / 2]
                                .to_vec(),
                        ),
                        time: Some(stereo_params.time as f32),
                        ..Default::default()
                    },
                );
            }
            // PROCESS STEREO
            else {
                let mut left_spectrum: Vec<f32> = vec![];
                let mut right_spectrum: Vec<f32> = vec![];
                if stereo_params.clean {
                    // ...each frame has 2 samples
                    for frame in output.chunks_mut(num_channels) {
                        if stereo_params.is_looping
                            && stereo_params.time
                                > stereo_params.loop_start_time + stereo_params.loop_length
                        {
                            stereo_params.time = stereo_params.loop_start_time;
                        }
                        if stereo_params.time + 2 >= stereo_params.num_file_samples {
                            break;
                        }
                        if !stereo_params.left.ui_params.left_mute {
                            let left_sample = file_samples[stereo_params.time]
                                * stereo_params.left.ui_params.output_gain;
                            let left_samp: T = T::from_sample(left_sample);
                            left_spectrum.push(left_sample);
                            let fr = frame.get_mut(0).unwrap();
                            *fr = left_samp;
                        }

                        if !stereo_params.right.ui_params.right_mute {
                            let right_sample = file_samples[stereo_params.time + 1]
                                * stereo_params.right.ui_params.output_gain;
                            let right_samp: T = T::from_sample(right_sample);
                            right_spectrum.push(right_sample);

                            let fr = frame.get_mut(1).unwrap();
                            *fr = right_samp;
                        }
                        stereo_params.time += 2;
                    }
                } else {
                    for frame in output.chunks_mut(num_channels) {
                        if stereo_params.is_looping
                            && stereo_params.time
                                > stereo_params.loop_start_time + stereo_params.loop_length
                        {
                            stereo_params.time = stereo_params.loop_start_time;
                        }
                        if stereo_params.time + 2 >= stereo_params.num_file_samples {
                            break;
                        }

                        if !stereo_params.left.ui_params.left_mute {
                            let left_sample = file_samples[stereo_params.time]
                                * stereo_params.left.ui_params.output_gain;
                            let left_filtered = stereo_params.left.sdft.spectral_subtraction(
                                left_sample,
                                &stereo_params.left.noise_spectrum,
                                stereo_params.left.ui_params.noise_gain,
                                stereo_params.left.ui_params.pre_smooth_gain,
                                stereo_params.left.ui_params.post_smooth_gain,
                            );

                            let left_samp: T = T::from_sample(left_filtered);
                            left_spectrum.push(left_filtered);
                            let fr = frame.get_mut(0).unwrap();
                            *fr = left_samp;
                        }

                        if !stereo_params.right.ui_params.right_mute {
                            let right_sample = file_samples[stereo_params.time + 1]
                                * stereo_params.right.ui_params.output_gain;
                            let right_filtered = stereo_params.right.sdft.spectral_subtraction(
                                right_sample,
                                &stereo_params.right.noise_spectrum,
                                stereo_params.right.ui_params.noise_gain,
                                stereo_params.right.ui_params.pre_smooth_gain,
                                stereo_params.right.ui_params.post_smooth_gain,
                            );
                            let right_samp: T = T::from_sample(right_filtered);
                            right_spectrum.push(right_filtered);
                            let fr = frame.get_mut(1).unwrap();
                            *fr = right_samp;
                        }
                        stereo_params.time += 2;
                    }
                }

                let _ = window.emit(
                    AudioUIMessage::name(),
                    AudioUIMessage {
                        spectrum: Some(
                            stereo_params.left.sdft.norm_vec()[0..stereo_params.left.sdft.size / 2]
                                .to_vec(),
                        ),
                        time: Some(stereo_params.time as f32),
                        ..Default::default()
                    },
                );
            }
        },
        err_fn,
        None,
    )?;

    Ok((stream, tx))
}

pub fn calculate_fingerprint(file_path: PathBuf, start: usize, len: usize) {
    println!(" ready to get fingerprint");
    let (file_samples, is_stereo) = get_wav_samples(file_path);
    let mut buf = vec![];
    for samp in file_samples[start..start + len].iter() {
        buf.push(Complex { re: *samp, im: 0.0 });
    }
    let fft_size = 512;
    let spectrum = averaged_stft(buf, fft_size, fft_size);
    // now do LMS to match filterbank to spectrum
    // this is one of those places where it would make sense to go between IIR2 and BPF, where BPF is a parameterized version, more intuitive control...interesting for ML maybe

    // does the spectrum have to be converted? well that's the goal, is to represent it with some filter bank, a sum of filters
    // matching one filter at a time? that would require some sense of a "partial-best-score", it would hit a local minimum, then it moves to a different region with a new filter...not sure if that will work

    // otherwise it's just randomly adjusting things and trying to move along gradient...this traditional method doesn't look at the big picture

    let mut filters = Filters::new();

    let score = 10000.0;
    let mu = 0.04;
    // maybe do num_iterations...but no idea how many it will take
    while score < 10.0 {
        // this is what i'm missing...where does the direction come from?need a gradient, a direction to go in...not sure
        for filter in filters.bank {
            // filter.b0 += mu * grad;
        }

        let s = filters.parallel_transfer(fft_size);
        // score gets updated like this?
        // score = sum(spectrum - s)
        // and then recalculate gradient?
    }
}
