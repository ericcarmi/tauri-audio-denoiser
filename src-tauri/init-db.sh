#!/bin/sh
rm db.sqlite

red=#ff0000
green=#00ff00
blue=#0000ff
cyan=#00ffff
yellow=#ffff00
magenta=#ff00ff
purple=#800080
orange=#cc8800

sqlite3 -batch db.sqlite "CREATE TABLE SETTINGS(id INTEGER PRIMARY KEY, plot_scale TEXT NOT NULL, theme TEXT NOT NULL, draw_freq_axis BOOLEAN, draw_fft_amp_axis BOOLEAN, draw_filter_amp_axis BOOLEAN);
insert into settings (plot_scale, theme, draw_freq_axis, draw_fft_amp_axis, draw_filter_amp_axis) values ('Log', 'RGB', true, true, true);
CREATE TABLE THEMES(id INTEGER PRIMARY KEY, name TEXT NOT NULL, rotary_ticks TEXT NOT NULL, slider_border TEXT NOT NULL, slider_hover TEXT NOT NULL, plot_main TEXT NOT NULL, plot_total_curve TEXT NOT NULL, rotary_hover TEXT NOT NULL, slider_indicator TEXT NOT NULL, slider_active TEXT NOT NULL, plot_single_filter TEXT NOT NULL, plot_filter_hover TEXT NOT NULL);

insert into themes(name, rotary_ticks, rotary_hover, slider_border, slider_indicator, slider_hover, slider_active, plot_main, plot_single_filter, plot_total_curve, plot_filter_hover) values ('RGB', '#ff0000', '#0000ff', '#0000ff', '#000000', '#00ff00', '#00ff00', '#888888', '#ffffff', '#00ff00', '#ff0000' );

insert into themes(name, rotary_ticks, rotary_hover, slider_border, slider_indicator, slider_hover, slider_active, plot_main, plot_single_filter, plot_total_curve, plot_filter_hover) values ('CYM', '$cyan', '$magenta', '$magenta', '#000000', '$yellow', '$yellow', '#888888', '#ffffff', '$yellow', '$cyan');

insert into themes(name, rotary_ticks, rotary_hover, slider_border, slider_indicator, slider_hover, slider_active, plot_main, plot_single_filter, plot_total_curve, plot_filter_hover) values ('POG', '$purple', '$green', '$orange', '#000000', '$orange', '$purple', '#888888', '#ffffff', '$green', '$purple' );


CREATE TABLE STEREO_PARAMS (
  id INTEGER PRIMARY KEY,
  control TEXT NOT NULL,
  is_stereo BOOLEAN,
  clean BOOLEAN
);

INSERT INTO STEREO_PARAMS (id, control, is_stereo, clean) VALUES (1, 'left', true, false);
INSERT INTO STEREO_PARAMS (id, control, is_stereo, clean) VALUES (2, 'right', true, false);
INSERT INTO STEREO_PARAMS (id, control, is_stereo, clean) VALUES (3, 'both', true, false);

CREATE TABLE CHANNEL_PARAMS (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  output_gain REAL,
  noise_gain REAL,
  pre_smooth_gain REAL,
  post_smooth_gain REAL,
  clean BOOLEAN,
  mute BOOLEAN
);

INSERT INTO CHANNEL_PARAMS (id, name, output_gain, noise_gain, pre_smooth_gain, post_smooth_gain, clean, mute) VALUES (1, 'left', 0.0, 0.0, 0.0, 0.0, false, false);

INSERT INTO CHANNEL_PARAMS (id, name, output_gain, noise_gain, pre_smooth_gain, post_smooth_gain, clean, mute) VALUES (2, 'right', 0.0, 0.0, 0.0, 0.0, false, false);

INSERT INTO CHANNEL_PARAMS (id, name, output_gain, noise_gain, pre_smooth_gain, post_smooth_gain, clean, mute) VALUES (3, 'both', 0.0, 0.0, 0.0, 0.0, false, false);

CREATE TABLE BPF (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  bpf_gain_1 REAL,
  bpf_freq_1 REAL,
  bpf_q_1 REAL,
  bpf_gain_2 REAL,
  bpf_freq_2 REAL,
  bpf_q_2 REAL,
  bpf_gain_3 REAL,
  bpf_freq_3 REAL,
  bpf_q_3 REAL,
  bpf_gain_4 REAL,
  bpf_freq_4 REAL,
  bpf_q_4 REAL,
  bpf_gain_5 REAL,
  bpf_freq_5 REAL,
  bpf_q_5 REAL
);

INSERT INTO BPF (id, name, bpf_gain_1, bpf_freq_1, bpf_q_1,  bpf_gain_2, bpf_freq_2, bpf_q_2,  bpf_gain_3, bpf_freq_3, bpf_q_3,  bpf_gain_4, bpf_freq_4, bpf_q_4,  bpf_gain_5, bpf_freq_5, bpf_q_5) VALUES (1, 'left', 0.0, 1000.0, 1.0, 0.0, 1000.0, 1.0, 0.0, 1000.0, 1.0, 0.0, 1000.0, 1.0, 0.0, 1000.0, 1.0);

INSERT INTO BPF (id, name, bpf_gain_1, bpf_freq_1, bpf_q_1,  bpf_gain_2, bpf_freq_2, bpf_q_2,  bpf_gain_3, bpf_freq_3, bpf_q_3,  bpf_gain_4, bpf_freq_4, bpf_q_4,  bpf_gain_5, bpf_freq_5, bpf_q_5) VALUES (2, 'right', 0.0, 1000.0, 1.0, 0.0, 1000.0, 1.0, 0.0, 1000.0, 1.0, 0.0, 1000.0, 1.0, 0.0, 1000.0, 1.0);

INSERT INTO BPF (id, name, bpf_gain_1, bpf_freq_1, bpf_q_1,  bpf_gain_2, bpf_freq_2, bpf_q_2,  bpf_gain_3, bpf_freq_3, bpf_q_3,  bpf_gain_4, bpf_freq_4, bpf_q_4,  bpf_gain_5, bpf_freq_5, bpf_q_5) VALUES (3, 'both', 0.0, 1000.0, 1.0, 0.0, 1000.0, 1.0, 0.0, 1000.0, 1.0, 0.0, 1000.0, 1.0, 0.0, 1000.0, 1.0);

"

cp db.sqlite .
cp db.sqlite target/debug