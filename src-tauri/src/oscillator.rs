/* This example expose parameter to pass generator of sample.
Good starting point for integration of cpal into your application.
*/

use ringbuf::{HeapRb, Rb};
use std::sync::{mpsc, Mutex};

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

pub struct MSender(pub Mutex<tauri::async_runtime::Sender<bool>>);

pub struct AppState(pub Mutex<AppStruct>);

pub struct AppStruct {
    pub stream: MStream,
    pub msender: MSender,
}

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
) -> Result<(cpal::Stream, tauri::async_runtime::Sender<bool>), anyhow::Error>
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
) -> Result<(cpal::Stream, tauri::async_runtime::Sender<bool>), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let num_channels = config.channels as usize;
    let mut oscillator = Oscillator {
        waveform: Waveform::Sine,
        sample_rate: config.sample_rate.0 as f32,
        current_sample_index: 0.0,
        frequency_hz: 440.0,
    };
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let time_at_start = std::time::Instant::now();
    println!("Time at start: {:?}", time_at_start);

    let (newtx, mut rx) = tauri::async_runtime::channel::<bool>(1);

    // let tx = mtx.0.lock().unwrap();

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            // for 0-1s play sine, 1-2s play square, 2-3s play saw, 3-4s play triangle_wave
            let time_since_start = std::time::Instant::now()
                .duration_since(time_at_start)
                .as_secs_f32();
            if time_since_start < 1.0 {
                oscillator.set_waveform(Waveform::Sine);
            } else if time_since_start < 2.0 {
                oscillator.set_waveform(Waveform::Triangle);
            } else if time_since_start < 3.0 {
                oscillator.set_waveform(Waveform::Square);
            } else if time_since_start < 4.0 {
                oscillator.set_waveform(Waveform::Saw);
            } else {
                oscillator.set_waveform(Waveform::Sine);
            }
            // process_frame(output, &mut oscillator, num_channels)
            // process_frame2(output, &mut oscillator, num_channels)

            if let Err(p) = rx.try_recv() {
                println!("{:?}", p);
                // if !p {
                //     return false;
                // }
            }
            for frame in output.chunks_mut(num_channels) {
                let f = oscillator.tick();
                let value: T = T::from_sample(f);

                // filter_bank.input_history.push(f);

                // copy the same value to all channels
                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        },
        err_fn,
        None,
    )?;

    Ok((stream, newtx))
}

#[tauri::command]
pub fn update_filters(app_state: State<AppState>) {
    let r = app_state
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .blocking_send(false);

    // let (tx, _) = tauri::async_runtime::channel::<bool>(1);
    // let tx = sender.0.lock().unwrap();
    // let r = tx.blocking_send(false);
    println!("{:?}", r);
}

fn process_frame<SampleType>(
    output: &mut [SampleType],
    oscillator: &mut Oscillator,
    num_channels: usize,
) where
    SampleType: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(num_channels) {
        let value: SampleType = SampleType::from_sample(oscillator.tick());

        // copy the same value to all channels
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

#[tauri::command]
fn process_frame2<SampleType>(
    output: &mut [SampleType],
    oscillator: &mut Oscillator,
    num_channels: usize,
) where
    SampleType: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(num_channels) {
        let f = oscillator.tick();
        let value: SampleType = SampleType::from_sample(f);

        // filter_bank.input_history.push(f);

        // copy the same value to all channels
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

pub struct FilterBank {
    pub coeffs: Vec<f32>,
    pub input_history: HeapRb<f32>,
}

impl FilterBank {
    pub fn new() -> Self {
        let rb = HeapRb::<f32>::new(2);

        Self {
            coeffs: vec![0.5, 0.5],
            input_history: rb,
        }
    }
}

impl std::fmt::Debug for FilterBank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("{:?}", self.coeffs);

        Ok(())
    }
}
