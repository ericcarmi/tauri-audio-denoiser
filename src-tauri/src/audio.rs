use crate::constants::*;
use crate::fourier::mfft;
use crate::types::*;
use anyhow;
use cpal::FromSample;
use cpal::{self};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    SizedSample,
};
use tauri::State;
use wavers::Wav;

pub fn get_wav_samples(path: &str) -> Vec<f32> {
    let mut wav: Wav<f32> = Wav::from_path(path).unwrap();
    wav.read().unwrap().to_vec()
}

#[tauri::command]
pub fn setup_stream(
    tx: tauri::async_runtime::Sender<Vec<f32>>,
) -> Result<(cpal::Stream, tauri::async_runtime::Sender<Message>), anyhow::Error>
where
{
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::I8 => make_stream::<i8>(&device, &config.into(), tx),
        cpal::SampleFormat::I16 => make_stream::<i16>(&device, &config.into(), tx),
        cpal::SampleFormat::I32 => make_stream::<i32>(&device, &config.into(), tx),
        cpal::SampleFormat::I64 => make_stream::<i64>(&device, &config.into(), tx),
        cpal::SampleFormat::U8 => make_stream::<u8>(&device, &config.into(), tx),
        cpal::SampleFormat::U16 => make_stream::<u16>(&device, &config.into(), tx),
        cpal::SampleFormat::U32 => make_stream::<u32>(&device, &config.into(), tx),
        cpal::SampleFormat::U64 => make_stream::<u64>(&device, &config.into(), tx),
        cpal::SampleFormat::F32 => make_stream::<f32>(&device, &config.into(), tx),
        cpal::SampleFormat::F64 => make_stream::<f64>(&device, &config.into(), tx),
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
    tx_ui: tauri::async_runtime::Sender<Vec<f32>>,
) -> Result<(cpal::Stream, tauri::async_runtime::Sender<Message>), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let num_channels = config.channels as usize;
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let (tx, mut rx) = tauri::async_runtime::channel::<Message>(1);
    let mut process_filterbank = FilterBank::new();

    // let mut rb = dasp_ring_buffer::Bounded::from(vec![0.0; 1]);
    // rb.push(0.0);
    // rb.push(0.0);
    // rb.push(0.0);
    // rb.push(0.0);

    let file_samples = get_wav_samples(TEST_FILE_PATH);
    let mut time = 0;
    let num_file_samples = file_samples.len();
    let mut clean = false;

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            // check for messages sent to receiver...update things
            if let Ok(msg) = rx.try_recv() {
                if let Some(bp) = msg.bp1 {
                    process_filterbank.bp1.update_coeffs(bp);
                }
                if let Some(bp) = msg.bp2 {
                    process_filterbank.bp2.update_coeffs(bp);
                }
                if let Some(bp) = msg.bp3 {
                    process_filterbank.bp3.update_coeffs(bp);
                }
                if let Some(bp) = msg.bp4 {
                    process_filterbank.bp4.update_coeffs(bp);
                }
                if let Some(bp) = msg.bp5 {
                    process_filterbank.bp5.update_coeffs(bp);
                }
                if let Some(t) = msg.time {
                    time = (num_file_samples as f32 * t) as usize;
                }
                if let Some(c) = msg.clean {
                    clean = c;
                }
            }
            // println!("{:?}", process_filterbank);

            // vec for fft, will make another for processed spectrum?
            let mut spectrum = vec![];
            if clean {
                // ...each frame has 2 samples
                for frame in output.chunks_mut(num_channels) {
                    if time >= num_file_samples {
                        break;
                    }
                    let sample = file_samples[time];
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
                    let sample = file_samples[time];
                    let f1 = process_filterbank.bp1.process(sample.clone());
                    let f2 = process_filterbank.bp2.process(sample.clone());
                    let f3 = process_filterbank.bp3.process(sample.clone());
                    let f4 = process_filterbank.bp4.process(sample.clone());
                    let f5 = process_filterbank.bp5.process(sample.clone());
                    let filtered = (f1 + f2 + f3 + f4 + f5) / 5.0;
                    // let filtered = f2;
                    // let filtered = (process_filterbank.bp1.process(sample)
                    //     + process_filterbank.bp2.process(sample)
                    //     + process_filterbank.bp3.process(sample)
                    //     + process_filterbank.bp4.process(sample)
                    //     + process_filterbank.bp5.process(sample))
                    //     / 5.0;
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
            let r = tx_ui.try_send(mfft(spectrum.clone()));
            // println!("{:?}", r);
        },
        err_fn,
        None,
    )?;

    Ok((stream, tx))
}

#[tauri::command]
pub fn update_filters(
    bp1: Option<IIR2>,
    bp2: Option<IIR2>,
    bp3: Option<IIR2>,
    bp4: Option<IIR2>,
    bp5: Option<IIR2>,
    streamsend: State<MStreamSend>,
) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            time: None,
            clean: None,
            bp1,
            bp2,
            bp3,
            bp4,
            bp5,
        });
}

#[tauri::command]
pub fn update_time(t: f32, streamsend: State<MStreamSend>) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            time: Some(t),
            clean: None,
            bp1: None,
            bp2: None,
            bp3: None,
            bp4: None,
            bp5: None,
        });
}

#[tauri::command]
pub fn update_clean(clean: bool, streamsend: State<MStreamSend>) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            time: None,
            clean: Some(clean),
            bp1: None,
            bp2: None,
            bp3: None,
            bp4: None,
            bp5: None,
        });
}
