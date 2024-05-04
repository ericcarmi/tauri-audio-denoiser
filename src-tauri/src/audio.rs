use crate::constants::*;
use crate::messages::{AudioUIMessage, UIAudioMessage};
use crate::types::*;
use anyhow;
use cpal::traits::StreamTrait;
use cpal::FromSample;
use cpal::{self};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    SizedSample,
};
use std::fs::File;
use tauri::{AppHandle, State, Window};

pub fn get_resource_wav_samples(path: &str, app_handle: AppHandle) -> (Vec<f32>, bool) {
    let p = app_handle
        .path_resolver()
        .resolve_resource(path)
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let file_in = File::open(p).unwrap();
    let (head, samples) = wav_io::read_from_file(file_in).unwrap();
    let is_stereo = if head.channels == 1 { false } else { true };
    (samples, is_stereo)
}

pub fn get_wav_samples(path: &str, app_handle: AppHandle) -> (Vec<f32>, bool) {
    if let Ok(file_in) = File::open(path) {
        let (head, samples) = wav_io::read_from_file(file_in).unwrap();
        let is_stereo = if head.channels == 1 { false } else { true };
        (samples, is_stereo)
    } else {
        let p = app_handle
            .path_resolver()
            .resolve_resource("assets/".to_owned() + path)
            .expect("failed to resolve resource")
            .into_os_string()
            .into_string()
            .unwrap();

        let f = File::open(p).unwrap();
        let (head, samples) = wav_io::read_from_file(f).unwrap();
        let is_stereo = if head.channels == 1 { false } else { true };
        (samples, is_stereo)
    }
}

#[tauri::command]
pub fn setup_stream(
    tx: tauri::async_runtime::Sender<AudioUIMessage>,
    app_handle: AppHandle,
    file_path: Option<String>,
) -> Result<(cpal::Stream, tauri::async_runtime::Sender<UIAudioMessage>), anyhow::Error>
where
{
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::I8 => {
            make_stream::<i8>(&device, &config.into(), tx, app_handle, file_path)
        }
        cpal::SampleFormat::I16 => {
            make_stream::<i16>(&device, &config.into(), tx, app_handle, file_path)
        }
        cpal::SampleFormat::I32 => {
            make_stream::<i32>(&device, &config.into(), tx, app_handle, file_path)
        }
        cpal::SampleFormat::I64 => {
            make_stream::<i64>(&device, &config.into(), tx, app_handle, file_path)
        }
        cpal::SampleFormat::U8 => {
            make_stream::<u8>(&device, &config.into(), tx, app_handle, file_path)
        }
        cpal::SampleFormat::U16 => {
            make_stream::<u16>(&device, &config.into(), tx, app_handle, file_path)
        }
        cpal::SampleFormat::U32 => {
            make_stream::<u32>(&device, &config.into(), tx, app_handle, file_path)
        }
        cpal::SampleFormat::U64 => {
            make_stream::<u64>(&device, &config.into(), tx, app_handle, file_path)
        }
        cpal::SampleFormat::F32 => {
            make_stream::<f32>(&device, &config.into(), tx, app_handle, file_path)
        }
        cpal::SampleFormat::F64 => {
            make_stream::<f64>(&device, &config.into(), tx, app_handle, file_path)
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
    // println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    // println!("Default output config : {:?}", config);

    Ok((host, device, config))
}

pub fn make_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    tx_ui: tauri::async_runtime::Sender<AudioUIMessage>,
    app_handle: AppHandle,
    file_path: Option<String>,
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
        (file_samples, is_stereo) = get_wav_samples(f.as_str(), app_handle.clone());
    } else {
        (file_samples, is_stereo) = get_resource_wav_samples(TEST_FILE_PATH, app_handle.clone());
    }
    let mut stereo_params = StereoParams::new();
    stereo_params.is_stereo = is_stereo;
    let _ = tx_ui.try_send(AudioUIMessage {
        is_stereo: Some(is_stereo),
        ..Default::default()
    });
    stereo_params.num_file_samples = file_samples.len();

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            if let Ok(msg) = rx.try_recv() {
                msg.receive(&mut stereo_params)
            }

            if !stereo_params.is_stereo {
                let mut spectrum: Vec<f32> = vec![];
                if stereo_params.clean {
                    // ...each frame has 2 samples
                    for frame in output.chunks_mut(num_channels) {
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

                // send a chunk of the fft here
                let _r = tx_ui.try_send(AudioUIMessage {
                    spectrum: Some(
                        stereo_params.left.sdft.norm_vec()[0..stereo_params.left.sdft.size / 2]
                            .to_vec(),
                    ),
                    ..Default::default()
                });
            }
            // PROCESS STEREO
            else {
                let mut left_spectrum: Vec<f32> = vec![];
                let mut right_spectrum: Vec<f32> = vec![];
                if stereo_params.clean {
                    // ...each frame has 2 samples
                    for frame in output.chunks_mut(num_channels) {
                        if stereo_params.time >= stereo_params.num_file_samples {
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
                        if stereo_params.time >= stereo_params.num_file_samples {
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

                // send a chunk of the fft here
                let _r = tx_ui.try_send(AudioUIMessage {
                    spectrum: Some(
                        stereo_params.left.sdft.norm_vec()[0..stereo_params.left.sdft.size / 2]
                            .to_vec(),
                    ),
                    ..Default::default()
                });
            }
        },
        err_fn,
        None,
    )?;

    Ok((stream, tx))
}

#[tauri::command]
pub async fn process_export(
    streamsend: State<'_, MStreamSend>,
    app_handle: AppHandle,
    file_path: Option<String>,
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
    if let Some(f) = file_path {
        (file_samples, is_stereo) = get_wav_samples(f.as_str(), app_handle.clone());
    } else {
        (file_samples, is_stereo) = get_resource_wav_samples(TEST_FILE_PATH, app_handle.clone());
    }
    let mut stereo_params = StereoParams::new();
    stereo_params.is_stereo = is_stereo;
    stereo_params.num_file_samples = file_samples.len();

    let _r = window.emit("update_processing_percentage", 0.0);
    // println!("{:?}", r);

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
            .resolve_resource(ASSETS_PATH)
            .expect("failed to resolve resource")
            .into_os_string()
            .into_string()
            .unwrap();

        let header = wav_io::new_stereo_header();
        if let Ok(mut file) = File::create(p + "/" + "output.wav") {
            let _r = wav_io::write_to_file(&mut file, &header, &samples);
        };
        Ok(())
    } else {
        Err("failed to write to file".to_string())
    }
}
