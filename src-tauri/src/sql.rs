use std::path::PathBuf;

use crate::{
    constants::NUM_FILTERS,
    errors::DenoiserResult,
    settings::{ComponentColors, Settings, Theme},
    types::{StereoChoice, UIFilters, UIParams, BPF},
};
use rusqlite::Connection;
use tauri::AppHandle;
// pub const DB_FILE_NAME: &'static str = "/db.sqlite";

/// pass in PathBuf from app handle with local data dir and .join
pub fn open_connection(p: &PathBuf) -> DenoiserResult<Connection> {
    let conn = Connection::open(p)?;
    Ok(conn)
}

pub fn query_theme_name(p: &PathBuf) -> DenoiserResult<Theme> {
    let conn = open_connection(p)?;
    let mut stmt = conn.prepare("SELECT theme FROM settings")?;
    let mut rows = stmt.query([])?;
    let mut theme: Theme = Theme::RGB;

    while let Some(row) = rows.next()? {
        theme = row.get(0)?;
    }
    Ok(theme)
}

#[tauri::command]
pub fn sql_theme_name(app_handle: AppHandle) -> DenoiserResult<Theme> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = query_theme_name(&p)?;
    Ok(q)
}

pub fn query_theme(p: PathBuf, theme: Theme) -> DenoiserResult<ComponentColors> {
    // let conn = Connection::open(s + DB_FILE_NAME)?;
    let conn = open_connection(&p)?;
    let mut stmt = conn.prepare(
        format!(
            "SELECT * FROM COMPONENTCOLORS WHERE name='{}'",
            theme.as_str()
        )
        .as_str(),
    )?;

    let colors_iter = stmt.query_map([], |row| {
        Ok(ComponentColors {
            rotary_tick: row.get(2)?,
            rotary_hover: row.get(3)?,
            slider_background: row.get(4)?,
            slider_border: row.get(5)?,
            slider_indicator: row.get(6)?,
            slider_hover: row.get(7)?,
            slider_active: row.get(8)?,
            plot_main: row.get(9)?,
            plot_single_filter: row.get(10)?,
            plot_total_curve: row.get(11)?,
            plot_filter_hover: row.get(12)?,
            app_background: row.get(13)?,
            app_text: row.get(14)?,
            button_background: row.get(15)?,
            button_text: row.get(16)?,
        })
    })?;

    for colors in colors_iter {
        if let Ok(c) = colors {
            return Ok(c);
        }
    }
    return Err(rusqlite::Error::InvalidQuery.into());
}

#[tauri::command]
pub fn sql_theme(theme: Theme, app_handle: AppHandle) -> DenoiserResult<ComponentColors> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = query_theme(p, theme)?;

    Ok(q)
}

pub fn query_settings(p: PathBuf) -> DenoiserResult<Settings> {
    let conn = open_connection(&p)?;
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
    return Err(rusqlite::Error::InvalidQuery.into());
}

pub fn query_ui_params(stereo_choice: StereoChoice, p: &PathBuf) -> DenoiserResult<UIParams> {
    let conn = open_connection(&p)?;
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
    return Err(rusqlite::Error::InvalidQuery.into());
}

#[tauri::command]
pub fn sql_ui_params(
    stereo_choice: StereoChoice,
    app_handle: AppHandle,
) -> DenoiserResult<UIParams> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = query_ui_params(stereo_choice, &p)?;

    Ok(q)
}

#[tauri::command]
pub fn sql_settings(app_handle: AppHandle) -> DenoiserResult<Settings> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = query_settings(p)?;

    Ok(q)
}

pub fn query_filter_bank(stereo_choice: StereoChoice, p: &PathBuf) -> DenoiserResult<UIFilters> {
    let conn = open_connection(p)?;
    let chan = stereo_choice.as_str().to_lowercase();
    let mut stmt =
        conn.prepare(format!("SELECT * FROM FILTERBANK WHERE stereo_choice='{}'", chan).as_str())?;
    let control_iter = stmt.query_map([], |row| {
        let offset = 2;
        let mut ui_filters = UIFilters::new();
        for i in 0..NUM_FILTERS {
            ui_filters.bank[i] = BPF {
                gain: row.get(offset + 3 * i)?,
                freq: row.get(offset + 3 * i + 1)?,
                Q: row.get(offset + 3 * i + 2)?,
            };
        }
        Ok(ui_filters)
    })?;

    for control in control_iter {
        if let Ok(sett) = control {
            return Ok(sett);
        }
    }
    return Err(rusqlite::Error::InvalidQuery.into());
}

#[tauri::command]
pub fn sql_filter_bank(
    stereo_choice: StereoChoice,
    app_handle: AppHandle,
) -> DenoiserResult<UIFilters> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = query_filter_bank(stereo_choice, &p)?;

    Ok(q)
}

/// update params AND filter bank, filter bank is part of ui params but treated separately because of how it isn't iterated over and has its own table
pub fn update_ui_params(
    stereo_choice: StereoChoice,
    ui_params: UIParams,
    p: PathBuf,
) -> DenoiserResult<()> {
    let conn = open_connection(&p)?;
    let st = stereo_choice.as_str().to_lowercase();

    let mut filter_string = "".to_string();
    let bank = ui_params.filters.bank;
    for i in 0..NUM_FILTERS {
        filter_string += format!(
            " bpf_gain_{} = {}, bpf_freq_{} = {}, bpf_Q_{} = {},",
            i, bank[i].gain, i, bank[i].freq, i, bank[i].Q
        )
        .as_str()
    }
    filter_string.pop();

    let mut q = format!("UPDATE UI_PARAMS SET clean={}, left_mute={}, right_mute={}, output_gain={}, noise_gain={}, pre_smooth_gain={}, post_smooth_gain={}  WHERE stereo_choice='{}';
        UPDATE FILTERBANK SET          
        ",ui_params.clean, ui_params.left_mute, ui_params.right_mute, ui_params.output_gain, ui_params.noise_gain, ui_params.pre_smooth_gain, ui_params.post_smooth_gain, st);

    let end_string = format!(" WHERE stereo_choice='{}'", st);
    q += filter_string.as_str();
    q += end_string.as_str();

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_ui_params(
    stereo_choice: StereoChoice,
    ui_params: UIParams,
    app_handle: AppHandle,
) -> DenoiserResult<()> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = update_ui_params(stereo_choice, ui_params, p)?;

    Ok(q)
}

pub fn update_filter_bank(
    stereo_choice: StereoChoice,
    bpf: BPF,
    index: usize,
    p: PathBuf,
) -> DenoiserResult<()> {
    let conn = open_connection(&p)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE FILTERBANK SET 
        bpf_gain_{} = {}, bpf_freq_{} = {}, bpf_q_{} = {}  
        WHERE stereo_choice='{}';
        ",
        index, bpf.gain, index, bpf.freq, index, bpf.Q, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_filter_bank(
    stereo_choice: StereoChoice,
    bpf: BPF,
    index: usize,
    app_handle: AppHandle,
) -> DenoiserResult<()> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = update_filter_bank(stereo_choice, bpf, index, p)?;

    Ok(q)
}

pub fn update_output_gain(
    stereo_choice: StereoChoice,
    output_gain: f32,
    p: PathBuf,
) -> DenoiserResult<()> {
    let conn = open_connection(&p)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET output_gain={} WHERE stereo_choice='{}';
        ",
        output_gain, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_output_gain(
    stereo_choice: StereoChoice,
    output_gain: f32,
    app_handle: AppHandle,
) -> DenoiserResult<()> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = update_output_gain(stereo_choice, output_gain, p)?;

    Ok(q)
}

pub fn update_noise_gain(
    stereo_choice: StereoChoice,
    noise_gain: f32,
    p: PathBuf,
) -> DenoiserResult<()> {
    let conn = open_connection(&p)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET noise_gain={} WHERE stereo_choice='{}';
        ",
        noise_gain, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_noise_gain(
    stereo_choice: StereoChoice,
    noise_gain: f32,
    app_handle: AppHandle,
) -> DenoiserResult<()> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = update_noise_gain(stereo_choice, noise_gain, p)?;

    Ok(q)
}

pub fn update_pre_smooth_gain(
    stereo_choice: StereoChoice,
    pre_smooth_gain: f32,
    p: PathBuf,
) -> DenoiserResult<()> {
    let conn = open_connection(&p)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET pre_smooth_gain={} WHERE stereo_choice='{}';
        ",
        pre_smooth_gain, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_pre_smooth_gain(
    stereo_choice: StereoChoice,
    pre_smooth_gain: f32,
    app_handle: AppHandle,
) -> DenoiserResult<()> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = update_pre_smooth_gain(stereo_choice, pre_smooth_gain, p)?;

    Ok(q)
}

pub fn update_post_smooth_gain(
    stereo_choice: StereoChoice,
    post_smooth_gain: f32,
    p: PathBuf,
) -> DenoiserResult<()> {
    let conn = open_connection(&p)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET post_smooth_gain={} WHERE stereo_choice='{}';
        ",
        post_smooth_gain, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_post_smooth_gain(
    stereo_choice: StereoChoice,
    post_smooth_gain: f32,
    app_handle: AppHandle,
) -> DenoiserResult<()> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = update_post_smooth_gain(stereo_choice, post_smooth_gain, p)?;

    Ok(q)
}

pub fn update_clean(stereo_choice: StereoChoice, clean: bool, p: PathBuf) -> DenoiserResult<()> {
    let conn = open_connection(&p)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET clean={} WHERE stereo_choice='{}';
        ",
        clean, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_clean(
    stereo_choice: StereoChoice,
    clean: bool,
    app_handle: AppHandle,
) -> DenoiserResult<()> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = update_clean(stereo_choice, clean, p)?;

    Ok(q)
}

pub fn update_left_mute(
    stereo_choice: StereoChoice,
    left_mute: bool,
    p: PathBuf,
) -> DenoiserResult<()> {
    let conn = open_connection(&p)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET left_mute={} WHERE stereo_choice='{}';
        ",
        left_mute, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_left_mute(
    stereo_choice: StereoChoice,
    left_mute: bool,
    app_handle: AppHandle,
) -> DenoiserResult<()> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = update_left_mute(stereo_choice, left_mute, p)?;

    Ok(q)
}

pub fn update_right_mute(
    stereo_choice: StereoChoice,
    right_mute: bool,
    p: PathBuf,
) -> DenoiserResult<()> {
    let conn = open_connection(&p)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET right_mute={} WHERE stereo_choice='{}';
        ",
        right_mute, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_right_mute(
    stereo_choice: StereoChoice,
    right_mute: bool,
    app_handle: AppHandle,
) -> DenoiserResult<()> {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("app local data dir should exist")
        .join("db.sqlite");

    let q = update_right_mute(stereo_choice, right_mute, p)?;
    Ok(q)
}

pub fn create_db(p: PathBuf) -> DenoiserResult<()> {
    let conn = open_connection(&p)?;

    let red = "#ff0000";
    let green = "#00ff00";
    let blue = "#0000ff";
    let cyan = "#00ffff";
    let yellow = "#ffff00";
    let magenta = "#ff00ff";
    let purple = "#800080";
    let orange = "#ee5500";
    let black = "#000000";
    let white = "#ffffff";
    let gray = "#888888";
    let sepia0 = "#ebd7cb";
    let sepia1 = "#ffe0b5";
    let sepia2 = "#d4aa7d";
    let sepia3 = "#b87d48";
    let sepia4 = "#9e6240";
    let sepia5 = "#644536";

    let stmt = format!("CREATE TABLE SETTINGS(id INTEGER PRIMARY KEY, plot_scale TEXT NOT NULL, theme TEXT NOT NULL, draw_freq_axis BOOLEAN, draw_fft_amp_axis BOOLEAN, draw_filter_amp_axis BOOLEAN);

INSERT into settings (plot_scale, theme, draw_freq_axis, draw_fft_amp_axis, draw_filter_amp_axis) values ('Log', 'SEPIA', true, true, true);

CREATE TABLE COMPONENTCOLORS(id INTEGER PRIMARY KEY,
 name TEXT NOT NULL,
 rotary_ticks TEXT NOT NULL,
 rotary_hover TEXT NOT NULL,
 slider_background TEXT NOT NULL,
 slider_border TEXT NOT NULL,
 slider_indicator TEXT NOT NULL,
 slider_hover TEXT NOT NULL,
 slider_active TEXT NOT NULL,
 plot_main TEXT NOT NULL,
 plot_single_filter TEXT NOT NULL,
 plot_total_curve TEXT NOT NULL,
 plot_filter_hover TEXT NOT NULL,
 app_background TEXT NOT NULL,
 app_text TEXT NOT NULL,
 button_background TEXT NOT NULL,
 button_text TEXT NOT NULL
);

insert into COMPONENTCOLORS(
 name,
 rotary_ticks,
 rotary_hover,
 slider_background,
 slider_border,
 slider_indicator,
 slider_hover,
 slider_active,
 plot_main,
 plot_single_filter,
 plot_total_curve,
 plot_filter_hover, 
 app_background, 
 app_text,
 button_background, 
 button_text
 ) values (
 'SEPIA',
 '{sepia3}',
 '{sepia4}',
 '{sepia0}',
 '{sepia5}',
 '{sepia2}',
 '{sepia2}',
 '{sepia1}',
 '{sepia3}',
 '{sepia5}',
 '{white}',
 '{sepia2}',
 '{sepia1}',
 '{black}',
 '{sepia5}',
 '{sepia0}'
);


insert into COMPONENTCOLORS(
 name,
 rotary_ticks,
 rotary_hover,
 slider_background,
 slider_border,
 slider_indicator,
 slider_hover,
 slider_active,
 plot_main,
 plot_single_filter,
 plot_total_curve,
 plot_filter_hover,
 app_background,
 app_text,
 button_background, 
 button_text
 ) values (
 'RGB',
 '#ff0000',
 '#0000ff',
 '{gray}',
 '#0000ff',
 '#000000',
 '#00ff00',
 '#00ff00',
 '#888888',
 '#ffffff',
 '#00ff00',
 '#ff0000',
 '{gray}',
 '{black}',
 '{black}',
 '{white}'
  );
insert into COMPONENTCOLORS(
 name,
 rotary_ticks,
 rotary_hover,
 slider_background,
 slider_border,
 slider_indicator,
 slider_hover,
 slider_active,
 plot_main,
 plot_single_filter,
 plot_total_curve,
 plot_filter_hover,
 app_background,
 app_text,
 button_background, 
 button_text
 ) values (
 'CYM',
 '{cyan}',
 '{magenta}',
 '{gray}',
 '{magenta}',
 '#000000',
 '{yellow}',
 '{yellow}',
 '#888888',
 '#ffffff',
 '{yellow}',
 '{cyan}',
 '{gray}',
 '{black}',
 '{black}',
 '{white}'
  );
insert into COMPONENTCOLORS(name,
 rotary_ticks,
 rotary_hover,
 slider_background,
 slider_border,
 slider_indicator,
 slider_hover,
 slider_active,
 plot_main,
 plot_single_filter,
 plot_total_curve,
 plot_filter_hover,
 app_background,
 app_text,
 button_background, 
 button_text
 ) values (
 'POG',
 '{purple}',
 '{green}',
 '{gray}',
 '{orange}',
 '#000000',
 '{orange}',
 '{purple}',
 '#888888',
 '#ffffff',
 '{green}',
 '{purple}',
 '{gray}',
 '{black}',
 '{black}',
 '{white}'
  );

insert into COMPONENTCOLORS(
 name,
 rotary_ticks,
 rotary_hover,
 slider_background,
 slider_border,
 slider_indicator,
 slider_hover,
 slider_active,
 plot_main,
 plot_single_filter,
 plot_total_curve,
 plot_filter_hover, 
 app_background, 
 app_text,
 button_background, 
 button_text
 ) values (
 'BWG',
 '{orange}',
 '{white}',
 '{gray}',
 '{white}',
 '{black}',
 '{orange}',
 '{orange}',
 '{orange}',
 '{gray}',
 '{white}',
 '{orange}', 
 '{gray}', 
 '{black}',
 '{black}',
 '{white}'
);


CREATE TABLE UI_PARAMS (
  id INTEGER PRIMARY KEY,
  stereo_choice TEXT NOT NULL,
  clean BOOLEAN,
  left_mute BOOLEAN,
  right_mute BOOLEAN,
  output_gain REAL,
  noise_gain REAL,
  pre_smooth_gain REAL,
  post_smooth_gain REAL
);

INSERT INTO UI_PARAMS (id, stereo_choice, output_gain, noise_gain, pre_smooth_gain, post_smooth_gain, clean, left_mute, right_mute) VALUES (1, 'left', 0.0, 0.0, 0.0, 0.0, false, false, false);
INSERT INTO UI_PARAMS (id, stereo_choice, output_gain, noise_gain, pre_smooth_gain, post_smooth_gain, clean, left_mute, right_mute) VALUES (2, 'right', 0.0, 0.0, 0.0, 0.0, false, false, false);
INSERT INTO UI_PARAMS (id, stereo_choice, output_gain, noise_gain, pre_smooth_gain, post_smooth_gain, clean, left_mute, right_mute) VALUES (3, 'both', 0.0, 0.0, 0.0, 0.0, false, false, false);

CREATE TABLE FILTERBANK (
  id INTEGER PRIMARY KEY,
  stereo_choice TEXT NOT NULL,
  bpf_gain_0 REAL,
  bpf_freq_0 REAL,
  bpf_Q_0 REAL,
  bpf_gain_1 REAL,
  bpf_freq_1 REAL,
  bpf_Q_1 REAL,
  bpf_gain_2 REAL,
  bpf_freq_2 REAL,
  bpf_Q_2 REAL,
  bpf_gain_3 REAL,
  bpf_freq_3 REAL,
  bpf_Q_3 REAL,
  bpf_gain_4 REAL,
  bpf_freq_4 REAL,
  bpf_Q_4 REAL
);

INSERT INTO FILTERBANK (stereo_choice, bpf_gain_0, bpf_freq_0, bpf_Q_0, bpf_gain_1, bpf_freq_1, bpf_Q_1,  bpf_gain_2, bpf_freq_2, bpf_Q_2,  bpf_gain_3, bpf_freq_3, bpf_Q_3,  bpf_gain_4, bpf_freq_4, bpf_Q_4) 
VALUES ('left', 0.0, 100.0, 1.0, 0.0, 500.0, 1.0, 0.0, 1000.0, 1.0, 0.0, 2000.0, 1.0, 0.0, 5000.0, 1.0);
INSERT INTO FILTERBANK (stereo_choice, bpf_gain_0, bpf_freq_0, bpf_Q_0, bpf_gain_1, bpf_freq_1, bpf_Q_1,  bpf_gain_2, bpf_freq_2, bpf_Q_2,  bpf_gain_3, bpf_freq_3, bpf_Q_3,  bpf_gain_4, bpf_freq_4, bpf_Q_4) 
VALUES ('right', 0.0, 100.0, 1.0, 0.0, 500.0, 1.0, 0.0, 1000.0, 1.0, 0.0, 2000.0, 1.0, 0.0, 5000.0, 1.0);
INSERT INTO FILTERBANK (stereo_choice, bpf_gain_0, bpf_freq_0, bpf_Q_0, bpf_gain_1, bpf_freq_1, bpf_Q_1,  bpf_gain_2, bpf_freq_2, bpf_Q_2,  bpf_gain_3, bpf_freq_3, bpf_Q_3,  bpf_gain_4, bpf_freq_4, bpf_Q_4) 
VALUES ('both', 0.0, 100.0, 1.0, 0.0, 500.0, 1.0, 0.0, 1000.0, 1.0, 0.0, 2000.0, 1.0, 0.0, 5000.0, 1.0);


CREATE TABLE FREQRANGE (
  id INTEGER PRIMARY KEY,
  stereo_choice TEXT NOT NULL,
  idx INTEGER,
  min REAL,
  max REAL
);

INSERT INTO FREQRANGE (stereo_choice, idx, min, max) VALUES (
  'left', 0, 20, 500  
);
INSERT INTO FREQRANGE (stereo_choice, idx, min, max) VALUES (
  'left', 1, 100, 1000
);
INSERT INTO FREQRANGE (stereo_choice, idx, min, max) VALUES (
  'left', 2, 500, 2000
);
INSERT INTO FREQRANGE (stereo_choice, idx, min, max) VALUES (
  'left', 3, 1000, 5000
);
INSERT INTO FREQRANGE (stereo_choice, idx, min, max) VALUES (
  'left', 4, 2000, 10000
);

");

    conn.execute_batch(stmt.as_str())?;
    Ok(())
}
