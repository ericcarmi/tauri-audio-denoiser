use rusqlite::{Connection, Result};
use tauri::AppHandle;

use crate::settings::{PlotScale, Settings, Theme};

// to avoid passing string for path into every location, the connection could be created and maintained as tauri state, which is fine because these function will be tauri commands called from frontend
pub fn create(s: String) -> Result<()> {
    let conn = Connection::open(s + "/test.sqlite")?;
    conn.execute(
        format!(
            "CREATE TABLE {} (
            id    INTEGER PRIMARY KEY,
            plot_scale  TEXT NOT NULL,
            theme  TEXT NOT NULL,
            draw_freq_axis BOOLEAN
        )",
            "settings",
        )
        .as_str(),
        (), // empty list of parameters.
    )?;
    Ok(())
}

#[tauri::command]
pub fn sql_create(app_handle: AppHandle) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    if let Ok(_) = create(p) {
        return Ok(());
    }
    Err("failed to update database".to_string())
}

pub fn update(s: String) -> Result<()> {
    let conn = Connection::open(s + "/test.sqlite")?;
    let me = Settings {
        plot_scale: PlotScale::Linear,
        theme: Theme::RGB,
        draw_freq_axis: true,
        ..Default::default()
    };
    conn.execute(
        "INSERT INTO settings (plot_scale, theme, draw_freq_axis) VALUES (?1, ?2, ?3)",
        (&me.plot_scale, &me.theme, &me.draw_freq_axis),
    )?;
    Ok(())
}

#[tauri::command]
pub fn sql_update(app_handle: AppHandle) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    if let Ok(_) = update(p) {
        return Ok(());
    }
    Err("failed to update database".to_string())
}

pub fn query(s: String) -> Result<()> {
    let conn = Connection::open(s + "/test.sqlite")?;
    let mut stmt = conn.prepare("SELECT id, plot_scale, draw_freq_axis FROM settings")?;
    let settings_iter = stmt.query_map([], |row| {
        Ok(Settings {
            plot_scale: row.get(1)?,
            draw_freq_axis: row.get(3)?,
            ..Default::default()
        })
    })?;

    // for settings in settings_iter.into_iter(). {
    //     println!("Found settings {:?}", settings.unwrap());
    // }
    println!("length of query {:?}", settings_iter.count());
    Ok(())
}

#[tauri::command]
pub fn sql_query(app_handle: AppHandle) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    if let Ok(_) = query(p) {
        return Ok(());
    }
    Err("failed to update database".to_string())
}
