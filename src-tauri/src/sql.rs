use crate::{
    settings::{Colors, PlotScale, Settings, Theme},
    types::{AudioParams, StereoChoice, StereoParams, UIFilterBank, UIParams, BPF},
};
use rusqlite::{Connection, Result};
use tauri::AppHandle;
pub const DB_FILE_NAME: &'static str = "/db.sqlite";

pub fn query_theme_name(s: String) -> Result<Theme, rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let mut stmt = conn.prepare("SELECT theme FROM settings")?;
    let mut rows = stmt.query([])?;
    let mut theme: Theme = Theme::RGB;

    while let Some(row) = rows.next()? {
        theme = row.get(0)?;
    }
    Ok(theme)
}

#[tauri::command]
pub fn sql_theme_name(app_handle: AppHandle) -> Result<Theme, String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = query_theme_name(p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn query_theme(s: String, theme: Theme) -> Result<Colors, rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let mut stmt =
        conn.prepare(format!("SELECT * FROM themes WHERE name='{}'", theme.as_str()).as_str())?;

    let colors_iter = stmt.query_map([], |row| {
        Ok(Colors {
            rotary_tick: row.get(2)?,
            rotary_hover: row.get(3)?,
            slider_hover: row.get(4)?,
            slider_border: row.get(5)?,
            slider_active: row.get(6)?,
            slider_indicator: row.get(7)?,
            plot_main: row.get(8)?,
            plot_single_filter: row.get(9)?,
            plot_total_curve: row.get(10)?,
            plot_filter_hover: row.get(11)?,
        })
    })?;

    for colors in colors_iter {
        if let Ok(c) = colors {
            return Ok(c);
        }
    }
    return Err(rusqlite::Error::InvalidQuery);
}

#[tauri::command]
pub fn sql_theme(theme: Theme, app_handle: AppHandle) -> Result<Colors, String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = query_theme(p, theme);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn query_settings(s: String) -> Result<Settings, rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let mut stmt = conn.prepare("SELECT * FROM SETTINGS")?;
    let settings_iter = stmt.query_map([], |row| {
        Ok(Settings {
            id: row.get(0)?,
            plot_scale: row.get(1)?,
            theme: row.get(2)?,
            draw_freq_axis: row.get(3)?,
            draw_fft_amp_axis: row.get(4)?,
            draw_filter_amp_axis: row.get(5)?,
        })
    })?;

    for settings in settings_iter {
        if let Ok(sett) = settings {
            return Ok(sett);
        }
    }
    return Err(rusqlite::Error::InvalidQuery);
}

pub fn query_ui_params(
    stereo_choice: StereoChoice,
    s: String,
) -> Result<UIParams, rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let chan = stereo_choice.as_str().to_lowercase();
    let mut stmt =
        conn.prepare(format!("SELECT * FROM UI_PARAMS WHERE stereo_choice='{}'", chan).as_str())?;
    let control_iter = stmt.query_map([], |row| {
        Ok(UIParams {
            // id: row.get(0)?,
            clean: row.get(2)?,
            left_mute: row.get(3)?,
            right_mute: row.get(4)?,
            output_gain: row.get(5)?,
            noise_gain: row.get(6)?,
            pre_smooth_gain: row.get(7)?,
            post_smooth_gain: row.get(8)?,
            ..Default::default()
        })
    })?;

    for control in control_iter {
        if let Ok(sett) = control {
            return Ok(sett);
        }
    }
    return Err(rusqlite::Error::InvalidQuery);
}

#[tauri::command]
pub fn sql_ui_params(
    stereo_choice: StereoChoice,
    app_handle: AppHandle,
) -> Result<UIParams, String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = query_ui_params(stereo_choice, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

#[tauri::command]
pub fn sql_settings(app_handle: AppHandle) -> Result<Settings, String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = query_settings(p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn query_filter_bank(
    stereo_choice: StereoChoice,
    s: String,
) -> Result<UIFilterBank, rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let chan = stereo_choice.as_str().to_lowercase();
    let mut stmt =
        conn.prepare(format!("SELECT * FROM FILTERBANK WHERE stereo_choice='{}'", chan).as_str())?;
    let control_iter = stmt.query_map([], |row| {
        let offset = 1;
        Ok(UIFilterBank {
            // id: row.get(0)?,
            bp1: BPF {
                gain: row.get(offset + 1)?,
                freq: row.get(offset + 2)?,
                Q: row.get(offset + 3)?,
            },
            bp2: BPF {
                gain: row.get(offset + 4)?,
                freq: row.get(offset + 5)?,
                Q: row.get(offset + 6)?,
            },
            bp3: BPF {
                gain: row.get(offset + 7)?,
                freq: row.get(offset + 8)?,
                Q: row.get(offset + 9)?,
            },
            bp4: BPF {
                gain: row.get(offset + 10)?,
                freq: row.get(offset + 11)?,
                Q: row.get(offset + 12)?,
            },
            bp5: BPF {
                gain: row.get(offset + 13)?,
                freq: row.get(offset + 14)?,
                Q: row.get(offset + 15)?,
            },
        })
    })?;

    for control in control_iter {
        if let Ok(sett) = control {
            return Ok(sett);
        }
    }
    return Err(rusqlite::Error::InvalidQuery);
}

#[tauri::command]
pub fn sql_filter_bank(
    stereo_choice: StereoChoice,
    app_handle: AppHandle,
) -> Result<UIFilterBank, String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = query_filter_bank(stereo_choice, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn update_ui_params(
    stereo_choice: StereoChoice,
    ui_params: UIParams,
    s: String,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!("UPDATE UI_PARAMS SET clean={}, left_mute={}, right_mute={}, output_gain={}, noise_gain={}, pre_smooth_gain={}, post_smooth_gain={}  WHERE stereo_choice='{}';
        UPDATE FILTERBANK SET 
        bpf_gain_1 = {}, bpf_freq_1 = {}, bpf_q_1 = {},  
        bpf_gain_2 = {}, bpf_freq_2 = {}, bpf_q_2 = {},  
        bpf_gain_3 = {}, bpf_freq_3 = {}, bpf_q_3 = {},  
        bpf_gain_4 = {}, bpf_freq_4 = {}, bpf_q_4 = {},  
        bpf_gain_5 = {}, bpf_freq_5 = {}, bpf_q_5 = {}  
        WHERE stereo_choice='{}';
        
        ",ui_params.clean, ui_params.left_mute, ui_params.right_mute, ui_params.output_gain, ui_params.noise_gain, ui_params.pre_smooth_gain, ui_params.post_smooth_gain, st, ui_params.filter_bank.bp1.gain, ui_params.filter_bank.bp1.freq, ui_params.filter_bank.bp1.Q,ui_params.filter_bank.bp2.gain, ui_params.filter_bank.bp2.freq, ui_params.filter_bank.bp2.Q, ui_params.filter_bank.bp3.gain, ui_params.filter_bank.bp3.freq, ui_params.filter_bank.bp3.Q,ui_params.filter_bank.bp4.gain, ui_params.filter_bank.bp4.freq, ui_params.filter_bank.bp4.Q,ui_params.filter_bank.bp5.gain, ui_params.filter_bank.bp5.freq, ui_params.filter_bank.bp5.Q, st);

    let stmt = conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_ui_params(
    stereo_choice: StereoChoice,
    ui_params: UIParams,
    app_handle: AppHandle,
) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = update_ui_params(stereo_choice, ui_params, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}
