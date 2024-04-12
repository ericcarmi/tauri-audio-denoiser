use std::fs::File;

use crate::constants::*;
use crate::fourier::mfft;
use crate::sdft::SDFT;
use crate::types::*;
use anyhow;
use cpal::FromSample;
use cpal::{self};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    SizedSample,
};
use dasp_ring_buffer::Fixed;
use rustfft::num_complex::Complex;
use tauri::AppHandle;

pub fn get_resource_wav_samples(path: &str, app_handle: AppHandle) -> Vec<f32> {
    let p = app_handle
        .path_resolver()
        .resolve_resource(path)
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let file_in = File::open(p).unwrap();
    let (_head, samples) = wav_io::read_from_file(file_in).unwrap();
    samples
}

pub fn get_wav_samples(path: &str) -> Vec<f32> {
    let file_in = File::open(path).unwrap();
    let (_head, samples) = wav_io::read_from_file(file_in).unwrap();

    samples
}

#[tauri::command]
pub fn setup_stream(
    tx: tauri::async_runtime::Sender<Vec<f32>>,
    app_handle: AppHandle,
    file_path: Option<String>,
) -> Result<(cpal::Stream, tauri::async_runtime::Sender<Message>), anyhow::Error>
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
    tx_ui: tauri::async_runtime::Sender<Vec<f32>>,
    app_handle: AppHandle,
    file_path: Option<String>,
) -> Result<(cpal::Stream, tauri::async_runtime::Sender<Message>), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let num_channels = config.channels as usize;
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let (tx, mut rx) = tauri::async_runtime::channel::<Message>(1);
    let mut process_filterbank = FilterBank::new();

    let file_samples;
    if let Some(f) = file_path {
        file_samples = get_wav_samples(f.as_str());
    } else {
        file_samples = get_resource_wav_samples(TEST_FILE_PATH, app_handle);
    }
    let mut time = 0;
    let num_file_samples = file_samples.len();
    let mut clean = false;
    let dft_size = 256;
    let mut sdft = SDFT::new(dft_size);
    let mut noise_spectrum = process_filterbank.parallel_transfer(dft_size);

    let mut noise_gain = 0.0;
    let mut output_gain = 1.0;
    let mut pre_smooth_gain = 0.5;
    let mut post_smooth_gain = 0.5;

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            // check for messages sent to receiver...update things
            if let Ok(msg) = rx.try_recv() {
                msg.receive(
                    dft_size,
                    num_file_samples,
                    &mut process_filterbank,
                    &mut noise_spectrum,
                    &mut time,
                    &mut sdft,
                    &mut clean,
                    &mut output_gain,
                    &mut noise_gain,
                    &mut pre_smooth_gain,
                    &mut post_smooth_gain,
                )
            }

            // vec for fft, will make another for processed spectrum?
            let mut spectrum: Vec<f32> = vec![];
            if clean {
                // ...each frame has 2 samples
                for frame in output.chunks_mut(num_channels) {
                    if time >= num_file_samples {
                        break;
                    }
                    let sample = file_samples[time] * output_gain;
                    let v: T = T::from_sample(sample);
                    spectrum.push(sample);

                    // copying to all channels for now
                    for out_sample in frame.iter_mut() {
                        *out_sample = v;
                    }
                    time += 1;
                }
            } else {
                for frame in output.chunks_mut(num_channels) {
                    if time >= num_file_samples {
                        break;
                    }
                    let sample = file_samples[time] * output_gain;
                    let filtered = sdft.spectral_subtraction(
                        sample,
                        &noise_spectrum,
                        noise_gain,
                        pre_smooth_gain,
                        post_smooth_gain,
                    );

                    let v: T = T::from_sample(filtered);
                    spectrum.push(filtered);

                    // copying to all channels for now
                    for out_sample in frame.iter_mut() {
                        *out_sample = v;
                    }
                    time += 1;
                }
            }

            // send a chunk of the fft here
            // let _r = tx_ui.try_send(mfft(spectrum.clone()));
            // let _r = tx_ui.try_send(freq_filter.clone());
            let _r = tx_ui.try_send(sdft.norm_vec()[0..sdft.size / 2].to_vec());

            // println!("{:?}", r);
        },
        err_fn,
        None,
    )?;

    Ok((stream, tx))
}
