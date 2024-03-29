/* This example expose parameter to pass generator of sample.
Good starting point for integration of cpal into your application.
*/

use wavers::{Samples, Wav};

use anyhow;
use cpal::{self};

use crate::constants::*;
use crate::types::*;
use cpal::FromSample;
use cpal::{
    traits::{DeviceTrait, HostTrait},
    SizedSample,
};
use tauri::State;

pub fn get_wav_samples(path: &str) -> Vec<f32> {
    let mut wav: Wav<f32> = Wav::from_path(path).unwrap();
    let samples: Samples<f32> = wav.read().unwrap();

    samples.to_vec()
}

#[tauri::command]
pub fn stream_setup_for(
) -> Result<(cpal::Stream, tauri::async_runtime::Sender<FilterBank>), anyhow::Error>
where
{
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::I8 => make_stream::<i8>(&device, &config.into()),
        cpal::SampleFormat::I16 => make_stream::<i16>(&device, &config.into()),
        cpal::SampleFormat::I32 => make_stream::<i32>(&device, &config.into()),
        cpal::SampleFormat::I64 => make_stream::<i64>(&device, &config.into()),
        cpal::SampleFormat::U8 => make_stream::<u8>(&device, &config.into()),
        cpal::SampleFormat::U16 => make_stream::<u16>(&device, &config.into()),
        cpal::SampleFormat::U32 => make_stream::<u32>(&device, &config.into()),
        cpal::SampleFormat::U64 => make_stream::<u64>(&device, &config.into()),
        cpal::SampleFormat::F32 => make_stream::<f32>(&device, &config.into()),
        cpal::SampleFormat::F64 => make_stream::<f64>(&device, &config.into()),
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
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config : {:?}", config);

    Ok((host, device, config))
}

pub fn make_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
) -> Result<(cpal::Stream, tauri::async_runtime::Sender<FilterBank>), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let num_channels = config.channels as usize;
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let (newtx, mut rx) = tauri::async_runtime::channel::<FilterBank>(1);
    let mut process_filterbank = FilterBank::new();

    let mut rb = dasp_ring_buffer::Bounded::from(vec![0.0; 1]);
    rb.push(0.0);
    // rb.push(0.0);
    // rb.push(0.0);
    // rb.push(0.0);

    let file_samples = get_wav_samples(TEST_FILE_PATH);
    let mut time = 0;
    let num_file_samples = file_samples.len();

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            // check for messages sent to receiver...update things
            if let Ok(p) = rx.try_recv() {
                process_filterbank.coeffs = p.coeffs.clone();
            }

            // ...each frame has 2 samples
            for frame in output.chunks_mut(num_channels) {
                if time >= num_file_samples {
                    break;
                }

                let sample = file_samples[time];
                let filtered =
                    process_filterbank.coeffs[0] * sample - process_filterbank.coeffs[1] * rb[0];
                let v: T = T::from_sample(filtered);
                rb.push(sample);
                // println!("{:?}", sample);

                // copying to all channels for now
                for out_sample in frame.iter_mut() {
                    *out_sample = v;
                }
                time += 1;
            }
        },
        err_fn,
        None,
    )?;

    Ok((stream, newtx))
}

#[tauri::command]
pub fn update_filters(
    alpha: f32,
    streamsend: State<MStreamSend>,
    mfilter_bank: State<MFilterBank>,
) {
    let mut filt = mfilter_bank.0.lock().unwrap();

    filt.coeffs = vec![alpha, 1.0 - alpha];
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(filt.clone());
}
