/* This example expose parameter to pass generator of sample.
Good starting point for integration of cpal into your application.
*/

// use dasp_ring_buffer::{Bounded, Fixed};
use hound::{WavReader, WavSamples};
use std::fs::File;
use std::io::BufReader;
use std::sync::{mpsc, Mutex};

use wavers::{read, Samples, Wav};

use anyhow;
use cpal::{self, Stream};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SizedSample,
};
use cpal::{FromSample, Sample};
use tauri::State;

pub struct MStream(pub Mutex<Stream>);
pub struct MFilterBank(pub Mutex<FilterBank>);

unsafe impl Sync for MStream {}
unsafe impl Send for MStream {}

pub struct MSender(pub Mutex<tauri::async_runtime::Sender<FilterBank>>);
// pub struct MSender2(pub Mutex<tauri::async_runtime::Sender<bool>>);

pub struct AppState(pub Mutex<AppStruct>);

pub struct AppStruct {
    pub stream: MStream,
    pub msender: MSender,
}
// let mut reader: WavReader<BufReader<File>> = hound::WavReader::open(file_path).unwrap();

const TEST_FILE_PATH: &str = "assets/test-file.wav";

pub enum Waveform {
    Sine,
    Square,
    Saw,
    Triangle,
}

pub struct Oscillator {
    pub sample_rate: f32,
    pub waveform: Waveform,
    pub current_sample_index: f32,
    pub frequency_hz: f32,
}

impl Oscillator {
    fn advance_sample(&mut self) {
        self.current_sample_index = (self.current_sample_index + 1.0) % self.sample_rate;
    }

    fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    fn calculate_sine_output_from_freq(&self, freq: f32) -> f32 {
        let two_pi = 2.0 * std::f32::consts::PI;
        (self.current_sample_index * freq * two_pi / self.sample_rate).sin()
    }

    fn is_multiple_of_freq_above_nyquist(&self, multiple: f32) -> bool {
        self.frequency_hz * multiple > self.sample_rate / 2.0
    }

    fn sine_wave(&mut self) -> f32 {
        self.advance_sample();
        self.calculate_sine_output_from_freq(self.frequency_hz)
    }

    fn generative_waveform(&mut self, harmonic_index_increment: i32, gain_exponent: f32) -> f32 {
        self.advance_sample();
        let mut output = 0.0;
        let mut i = 1;
        while !self.is_multiple_of_freq_above_nyquist(i as f32) {
            let gain = 1.0 / (i as f32).powf(gain_exponent);
            output += gain * self.calculate_sine_output_from_freq(self.frequency_hz * i as f32);
            i += harmonic_index_increment;
        }
        output
    }

    fn square_wave(&mut self) -> f32 {
        self.generative_waveform(2, 1.0)
    }

    fn saw_wave(&mut self) -> f32 {
        self.generative_waveform(1, 1.0)
    }

    fn triangle_wave(&mut self) -> f32 {
        self.generative_waveform(2, 2.0)
    }

    fn tick(&mut self) -> f32 {
        match self.waveform {
            Waveform::Sine => self.sine_wave(),
            Waveform::Square => self.square_wave(),
            Waveform::Saw => self.saw_wave(),
            Waveform::Triangle => self.triangle_wave(),
        }
    }
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
    // let mut oscillator = Oscillator {
    //     waveform: Waveform::Triangle,
    //     sample_rate: config.sample_rate.0 as f32,
    //     current_sample_index: 0.0,
    //     frequency_hz: 440.0,
    // };
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
    println!("{:?}", num_file_samples);

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
pub fn update_filters(alpha: f32, app_state: State<AppState>, mfilter_bank: State<MFilterBank>) {
    let mut filt = mfilter_bank.0.lock().unwrap();

    filt.coeffs = vec![alpha, 1.0 - alpha];
    let _ = app_state
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(filt.clone());
}

#[derive(Clone)]
pub struct FilterBank {
    pub coeffs: Vec<f32>,
    // pub input_history: Bounded<Vec<f32>>,
}

impl FilterBank {
    pub fn new() -> Self {
        // let rb = dasp_ring_buffer::Bounded::from(vec![0.0; 3]);

        Self {
            coeffs: vec![0.5, 0.5],
            // input_history: rb,
        }
    }
}

// use options with everything...a little annoying but then use None when passing to ignore most sub-structs
#[derive(Clone)]
pub struct Message {
    pub filter_bank: Option<FilterBank>,
    // pub wav_file: Option<WavReader<BufReader<File>>>,
    pub file_samples: Option<Vec<f32>>,
}

pub fn get_wav_samples(path: &str) -> Vec<f32> {
    let mut wav: Wav<f32> = Wav::from_path(path).unwrap();
    let samples: Samples<f32> = wav.read().unwrap();

    samples.to_vec()
}
