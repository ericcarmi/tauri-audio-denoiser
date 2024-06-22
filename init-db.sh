#!/bin/bash
rm db.sqlite

red=#ff0000
green=#00ff00
blue=#0000ff
cyan=#00ffff
yellow=#ffff00
magenta=#ff00ff
purple=#800080
orange=#ee5500
black=#000000
white=#ffffff
gray=#888888

sepia1=#ffe0b5
sepia2=#d4aa7d
sepia3=#b87d48
sepia4=#9e6240
sepia5=#644536


sqlite3 -batch db.sqlite "CREATE TABLE SETTINGS(id INTEGER PRIMARY KEY, plot_scale TEXT NOT NULL, theme TEXT NOT NULL, draw_freq_axis BOOLEAN, draw_fft_amp_axis BOOLEAN, draw_filter_amp_axis BOOLEAN);

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
 app_text TEXT NOT NULL
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
 app_text) values (
 'SEPIA',
 '$sepia1',
 '$white',
 '$sepia4',
 '$black',
 '$black',
 '$white',
 '$sepia4',
 '$sepia3',
 '$sepia4',
 '$white',
 '$sepia2', '$sepia2', '$black'
);


insert into COMPONENTCOLORS(name, rotary_ticks, rotary_hover, slider_background, slider_border, slider_indicator, slider_hover, slider_active, plot_main, plot_single_filter, plot_total_curve, plot_filter_hover, app_background, app_text) values ('RGB', '#ff0000', '#0000ff', '$gray', '#0000ff', '#000000', '#00ff00', '#00ff00', '#888888', '#ffffff', '#00ff00', '#ff0000', '$gray', '$black' );
insert into COMPONENTCOLORS(name, rotary_ticks, rotary_hover, slider_background, slider_border, slider_indicator, slider_hover, slider_active, plot_main, plot_single_filter, plot_total_curve, plot_filter_hover, app_background, app_text) values ('CYM', '$cyan', '$magenta', '$gray', '$magenta', '#000000', '$yellow', '$yellow', '#888888', '#ffffff', '$yellow', '$cyan', '$gray', '$black');
insert into COMPONENTCOLORS(name, rotary_ticks, rotary_hover, slider_background, slider_border, slider_indicator, slider_hover, slider_active, plot_main, plot_single_filter, plot_total_curve, plot_filter_hover, app_background, app_text) values ('POG', '$purple', '$green', '$gray', '$orange', '#000000', '$orange', '$purple', '#888888', '#ffffff', '$green', '$purple', '$gray', '$black' );

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
 plot_filter_hover, app_background, app_text) values (
 'BWG',
 '$orange',
 '$white',
 '$gray',
 '$white',
 '$black',
 '$orange',
 '$orange',
 '$orange',
 '$gray',
 '$white',
 '$orange', '$gray', '$black'
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

"

cp db.sqlite ./src-tauri/
cp db.sqlite ./src-tauri/target/debug
# rm db.sqlite

echo "database generated"
