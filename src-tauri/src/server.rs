use crate::constants::*;
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
    println!("{:?}", r);

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
    println!("set fft {:?}", r);

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

#[derive(Serialize, Deserialize, Debug)]
struct Thing {}
