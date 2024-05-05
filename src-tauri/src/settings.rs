use rusqlite::{types::FromSql, ToSql};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct Settings {
    pub id: i32,
    pub plot_scale: PlotScale,
    pub theme: Theme,
    pub draw_freq_axis: bool,
    pub draw_fft_amp_axis: bool,
    pub draw_filter_amp_axis: bool,
    // pub fft_plot_decay: f32,
    // pub fft_plot_size: usize,
    // pub redis_update_time: usize,
    // pub redis_update_amount: usize,
    // pub colors: Colors,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            id: 0,
            plot_scale: PlotScale::Linear,
            draw_freq_axis: true,
            draw_fft_amp_axis: true,
            draw_filter_amp_axis: true,
            theme: Theme::POG,
            // fft_plot_decay: 0.8,
            // fft_plot_size: 256,
            // redis_update_time: 30,
            // redis_update_amount: 5,
            // colors: Colors::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub enum PlotScale {
    Linear,
    Mel,
    Log,
    Bark,
}

impl ToSql for PlotScale {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        use PlotScale::*;
        match self {
            Linear => "Linear".to_sql(),
            Mel => "Mel".to_sql(),
            Log => "Log".to_sql(),
            Bark => "Bark".to_sql(),
        }
    }
}

impl FromSql for PlotScale {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        use PlotScale::*;
        if let Ok(v) = value.as_str() {
            return match v {
                "Linear" => Ok(Linear),
                "Mel" => Ok(Mel),
                "Log" => Ok(Log),
                "Bark" => Ok(Bark),
                _ => Err(rusqlite::types::FromSqlError::InvalidType),
            };
        }
        Err(rusqlite::types::FromSqlError::InvalidType)
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub enum Theme {
    RGB,
    CYM,
    POG,
    BWG,
    CUSTOM,
}

impl ToSql for Theme {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        use Theme::*;
        match self {
            RGB => "RGB".to_sql(),
            CYM => "CYM".to_sql(),
            POG => "POG".to_sql(),
            BWG => "BWG".to_sql(),
            CUSTOM => "CUSTOM".to_sql(),
        }
    }
}

impl FromSql for Theme {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        use Theme::*;
        if let Ok(v) = value.as_str() {
            return match v {
                "RGB" => Ok(RGB),
                "CYM" => Ok(CYM),
                "POG" => Ok(POG),
                "BWG" => Ok(BWG),
                "CUSTOM" => Ok(CUSTOM),
                _ => Err(rusqlite::types::FromSqlError::InvalidType),
            };
        }
        Err(rusqlite::types::FromSqlError::InvalidType)
    }
}

impl Theme {
    pub fn get_colors(&self, theme: Theme) -> Colors {
        use Theme::*;
        match theme {
            RGB => self.rgb(),
            CYM => self.cym(),
            POG => self.pog(),
            BWG => self.bwg(),
            CUSTOM => self.cym(),
        }
    }
    pub fn as_str(&self) -> &str {
        use Theme::*;
        match self {
            RGB => "RGB",
            CYM => "CYM",
            POG => "POG",
            BWG => "BWG",
            CUSTOM => "CUSTOM",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, &str> {
        use Theme::*;
        match s {
            "RGB" => Ok(RGB),
            "CYM" => Ok(CYM),
            "POG" => Ok(POG),
            "CUSTOM" => Ok(CUSTOM),
            _ => Err("invalid theme string"),
        }
    }
    pub fn rgb(&self) -> Colors {
        Colors {
            rotary_tick: RED,
            rotary_hover: BLUE,
            slider_hover: LIGHTBLUE,
            slider_border: BLUE,
            slider_active: GREEN,
            slider_indicator: RED,
            plot_main: BLUE,
            plot_single_filter: GRAY100,
            plot_total_curve: GRAY200,
            plot_filter_hover: RED,
        }
    }
    pub fn cym(&self) -> Colors {
        Colors {
            rotary_tick: CYAN,
            rotary_hover: MAGENTA,
            slider_hover: CYAN,
            slider_border: MAGENTA,
            slider_active: YELLOW,
            slider_indicator: MAGENTA,
            plot_main: MAGENTA,
            plot_single_filter: GRAY100,
            plot_total_curve: GRAY200,
            plot_filter_hover: CYAN,
        }
    }
    pub fn pog(&self) -> Colors {
        Colors {
            rotary_tick: LIGHTPURPLE,
            rotary_hover: GREEN,
            slider_hover: LIGHTPURPLE,
            slider_border: PURPLE,
            slider_active: GREEN,
            slider_indicator: GREEN,
            plot_main: PURPLE,
            plot_single_filter: GRAY100,
            plot_total_curve: GRAY200,
            plot_filter_hover: PURPLE,
        }
    }
    pub fn bwg(&self) -> Colors {
        Colors {
            rotary_tick: ORANGE,
            rotary_hover: WHITE,
            slider_hover: GRAY100,
            slider_border: BLACK,
            slider_active: ORANGE,
            slider_indicator: ORANGE,
            plot_main: ORANGE,
            plot_single_filter: GRAY100,
            plot_total_curve: WHITE,
            plot_filter_hover: ORANGE,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct Colors {
    pub rotary_tick: Color,
    pub rotary_hover: Color,
    pub slider_hover: Color,
    pub slider_border: Color,
    pub slider_active: Color,
    pub slider_indicator: Color,
    pub plot_main: Color,
    pub plot_single_filter: Color,
    pub plot_total_curve: Color,
    pub plot_filter_hover: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            rotary_tick: LIGHTPURPLE,
            rotary_hover: GREEN,
            slider_hover: LIGHTPURPLE,
            slider_border: PURPLE,
            slider_active: GREEN,
            slider_indicator: GREEN,
            plot_main: PURPLE,
            plot_single_filter: GRAY100,
            plot_total_curve: GRAY200,
            plot_filter_hover: PURPLE,
        }
    }
}

impl FromSql for Color {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        if let Ok(v) = value.as_str() {
            if let Ok(c) = Color::new_from_hex(v) {
                return Ok(c);
            }
        }
        Err(rusqlite::types::FromSqlError::InvalidType)
    }
}

impl FromSql for Colors {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        if let Ok(v) = value.as_str() {
            // let mut colors = vec![];
            // for color in v {
            //     colors.push(color);
            // }
            return Ok(Colors::default());
        }
        Err(rusqlite::types::FromSqlError::InvalidType)
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: Option<f32>,
}

impl Color {
    pub fn newt(r: u8, g: u8, b: u8, a: f32) -> Self {
        return Self {
            r,
            g,
            b,
            a: Some(a),
        };
    }

    pub fn new_from_hex(hex_string: &str) -> Result<Self, String> {
        if hex_string.len() != 7 && &hex_string[0..1] != "#" {
            return Err("invalid color".to_string());
        }

        Ok(Self {
            r: u8::from_str_radix(&hex_string[1..3], 16).unwrap(),
            g: u8::from_str_radix(&hex_string[3..5], 16).unwrap(),
            b: u8::from_str_radix(&hex_string[5..7], 16).unwrap(),
            a: None,
        })
    }
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        return Self { r, g, b, a: None };
    }
    pub fn to_string(&self) -> String {
        if let Some(t) = self.a {
            format!("rgb({},{},{},{})", self.r, self.g, self.b, t)
        } else {
            format!("rgb({},{},{})", self.r, self.g, self.b)
        }
    }
}

fn hex_to_rgb(hex_string: &str) -> Result<(u8, u8, u8), String> {
    if hex_string.len() != 7 && &hex_string[0..1] != "#" {
        return Err("invalid color".to_string());
    }
    let r = hex_string[1..3].parse::<u8>().unwrap();
    let g = hex_string[3..5].parse::<u8>().unwrap();
    let b = hex_string[5..7].parse::<u8>().unwrap();
    Ok((r, g, b))
}
// fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
//     let mut hex = "".to_string();
//     // Convert the RGB values to a hexadecimal representation
//     for i in 0..3 {
//         let val = (r >> i & 0xff) as u16;
//         if val <= 9 {
//             hex.push('0');
//         }
//         hex.push(format!("{:X}", val).chars().next().unwrap());
//     }
//     // Return the hexadecimal representation
//     hex
// }

pub const RED: Color = Color::new(170, 0, 0);
// pub const GREEN: Color = Color::new(0, 180, 0);
pub const BLUE: Color = Color::new(0, 0, 170);

// pub const LIGHTRED: Color = Color::new(230, 0, 0);
// pub const LIGHTGREEN: Color = Color::new(0, 180, 0);
pub const LIGHTBLUE: Color = Color::new(0, 0, 230);

pub const PURPLE: Color = Color::new(100, 0, 140);
pub const ORANGE: Color = Color::new(220, 100, 0);
pub const GREEN: Color = Color::new(0, 140, 0);

pub const CYAN: Color = Color::new(0, 200, 240);
pub const YELLOW: Color = Color::new(220, 220, 0);
pub const MAGENTA: Color = Color::new(200, 0, 250);

pub const LIGHTPURPLE: Color = Color::new(100, 0, 140);
// pub const LIGHTORANGE: Color = Color::new(220, 100, 0);
// pub const LIGHTGREEN: Color = Color::new(0, 140, 0);

// pub const GRAY50: Color = Color::new(50, 50, 50);
pub const GRAY100: Color = Color::new(100, 100, 100);
// pub const GRAY150: Color = Color::new(100, 100, 100);
pub const GRAY200: Color = Color::new(200, 200, 200);
// pub const GRAY250: Color = Color::new(250, 250, 250);
pub const WHITE: Color = Color::new(255, 255, 255);
pub const BLACK: Color = Color::new(0, 0, 0);
