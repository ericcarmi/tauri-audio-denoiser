use crate::{
    constants::*,
    types::{Bpf, FilterBank},
};
use redis::AsyncCommands;
use rustfft::{num_complex::Complex, FftPlanner};
use tauri::{AppHandle, State};

use redis::Commands;
use wavers::Wav;

use serde::{Deserialize, Serialize};

fn fetch_integer() -> redis::RedisResult<isize> {
    // connect to redis
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;
    // throw away the result, just make sure it does not fail
    // let _: () = con.set("my_key", 42)?;
    // read back the key and return it.  Because the return value
    // from the function is a result for integer this will automatically
    // convert into one.
    con.get("my_fey")
}

#[tauri::command]
pub fn get_integer() -> Result<isize, String> {
    let r = fetch_integer();
    // println!("{:?}", r);

    if r.is_ok() {
        Ok(r.unwrap())
    } else {
        let s = r.err().unwrap();
        Err(s.to_string())
    }
}

fn fetch_fft(file_name: String) -> redis::RedisResult<String> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;
    let r = con.get(file_name + "-fft");

    r
}

#[tauri::command]
pub async fn get_file_fft(file_name: &str, app_handle: AppHandle) -> Result<Vec<f32>, String> {
    let p = app_handle
        .path_resolver()
        .resolve_resource(ASSETS_PATH)
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let filepath = p + "/" + file_name;
    let handle = tauri::async_runtime::spawn(async move {
        let r = fetch_fft(filepath);
        return r;
    });

    let r = handle.await;

    if r.is_ok() {
        // Ok(r.unwrap().unwrap())
        let de: Vec<f32> = serde_json::from_str(r.unwrap().unwrap().as_str()).unwrap();
        Ok(de)
    } else {
        let s = r.err().unwrap();
        Err(s.to_string())
    }
}

#[tauri::command]
pub async fn set_file_fft(file_name: &str, app_handle: AppHandle) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resolve_resource(ASSETS_PATH)
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let filepath = p + "/" + file_name;

    let r = redis_set_file_fft(filepath.as_str()).await;
    // println!("set fft {:?}", r);

    Ok(())
}

async fn redis_set_file_fft(file_name: &str) -> redis::RedisResult<()> {
    let p = file_name.to_string() + "-fft";
    let filepath = file_name.to_string();

    let thread = tauri::async_runtime::spawn(async move {
        let w: Vec<f32> = Wav::from_path(filepath).unwrap().read().unwrap().to_vec();
        let mut vfft: Vec<f32> = vec![];
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

    // if thread has error, problem fft or reading file
    // no error means go ahead and set the data in redis
    let r = thread.await;
    let data = r.unwrap().1;

    // let data: String = vec![1, 0].iter().map(|x| x.to_string() + ",").collect();
    // println!("{:?}", r);

    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_multiplexed_async_connection().await?;

    // let x = con.set(p, data).await;
    let x = con.set(p, serde_json::to_string(&data).unwrap()).await;
    x
}

async fn redis_save_global_state(bpfs: Vec<Bpf>) -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_multiplexed_async_connection().await?;
    // println!("{:?}", bpfs);

    for (i, bpf) in bpfs.iter().enumerate() {
        let a: Result<(), redis::RedisError> = con.set(format!("gain-{}", i + 1), bpf.gain).await;
        if a.is_err() {
            return a;
        }
        let b: Result<(), redis::RedisError> = con.set(format!("freq-{}", i + 1), bpf.freq).await;
        if b.is_err() {
            return b;
        }
        let c: Result<(), redis::RedisError> = con.set(format!("Q-{}", i + 1), bpf.Q).await;
        if c.is_err() {
            return c;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn save_global_state(bpfs: Vec<Bpf>) {
    let r = redis_save_global_state(bpfs).await;
    // println!("{:?}", r);
}

async fn redis_get_global_state() -> redis::RedisResult<Vec<Bpf>> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_multiplexed_async_connection().await?;

    let mut bpfs = vec![];
    for i in 0..5 {
        let gain: Result<f32, redis::RedisError> = con.get(format!("gain-{}", i + 1)).await;
        if gain.is_err() {
            // this isn't good but weird since this needs to turn bpf and redis result
            return Ok(bpfs);
        }
        let freq: Result<f32, redis::RedisError> = con.get(format!("freq-{}", i + 1)).await;
        if freq.is_err() {
            return Ok(bpfs);
        }
        let Q: Result<f32, redis::RedisError> = con.get(format!("Q-{}", i + 1)).await;
        if Q.is_err() {
            return Ok(bpfs);
        }
        bpfs.push(Bpf {
            gain: gain.unwrap(),
            freq: freq.unwrap(),
            Q: Q.unwrap(),
        })
    }

    Ok(bpfs)
}

#[tauri::command]
pub async fn get_global_state() -> Result<Vec<Bpf>, String> {
    if let Ok(bpfs) = redis_get_global_state().await {
        Ok(bpfs)
    } else {
        Err("failed to get global state".to_string())
    }
}

async fn redis_save_bpf_gain(gain: f32, index: usize) -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_multiplexed_async_connection().await?;
    let a: Result<(), redis::RedisError> = con.set(format!("gain-{}", index), gain).await;
    return a;
}

#[tauri::command]
pub async fn save_bpf_gain(gain: f32, index: usize) {
    let r = redis_save_bpf_gain(gain, index).await;
    // println!("{:?}", r);
}

async fn redis_save_bpf_freq(freq: f32, index: usize) -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_multiplexed_async_connection().await?;
    let a: Result<(), redis::RedisError> = con.set(format!("freq-{}", index), freq).await;
    return a;
}

#[tauri::command]
pub async fn save_bpf_freq(freq: f32, index: usize) {
    let r = redis_save_bpf_freq(freq, index).await;
    // println!("{:?}", r);
}

async fn redis_save_bpf_Q(Q: f32, index: usize) -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_multiplexed_async_connection().await?;
    let a: Result<(), redis::RedisError> = con.set(format!("Q-{}", index), Q).await;
    return a;
}

#[tauri::command]
pub async fn save_bpf_Q(Q: f32, index: usize) {
    let r = redis_save_bpf_Q(Q, index).await;
    // println!("{:?}", r);
}
