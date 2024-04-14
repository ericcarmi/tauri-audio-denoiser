<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { listen } from "@tauri-apps/api/event";
  import { message, open, save } from "@tauri-apps/api/dialog";
  import { onMount } from "svelte";

  import Plot from "./plot.svelte";
  import {
    DOWN_RATE,
    FREQ_PLOT_WIDTH,
    SAMPLING_RATE,
    num_sliders,
  } from "./constants.svelte";
  import BandpassSlider from "./bandpass-slider.svelte";
  import type { BPF, StereoControl, StereoParams } from "./types.svelte";
  import {
    init_channel_params,
    remove_slashes_ext,
    rgbToHex,
    update_css_color,
    update_filters,
  } from "./functions.svelte";
  import RotarySlider from "./rotary-slider.svelte";
  import Settings from "./settings.svelte";

  let settings: any;

  let show_settings = false;
  // if these values are the same as what is in server, values will not update when loaded, so use values that are way out of range? silly but it works
  var gains = [0, 0, 0, 0, 0];
  var freqs = [1000, 1000, 1000, 1000, 1000];
  var Qs = [1, 1, 1, 1, 1];

  const unlisten = listen("tauri://file-drop", async (event: any) => {
    change_file(event.payload[0] as string);
  });

  function change_file(path: string, from_assets?: boolean) {
    selectedRecording = path;
    invoke("update_file_path", { path: selectedRecording });
    invoke("get_is_stereo").then((r: any) => {
      if (r.is_stereo !== undefined) {
        stereo_params.is_stereo = r.is_stereo;
      }
    });
    invoke("get_stereo_control").then((r: any) => {
      if (r !== undefined) {
        stereo_params.control = r;
      }
    });
    get_time_data(from_assets);
  }

  let time = 0;
  let time_position = 0;
  let selectedRecording = "";
  let is_playing = false;

  let perf_time = performance.now();
  let time_origin = 0;
  let time_delta = 0;
  let is_time_slider_dragging = false;

  let interval: any;
  let fft_data: any;

  // these values are retrieved onMount from server...want to also get the types over eventually to not copy-paste
  // but also the audio params aren't an exact copy of the rust type?
  // keep this values to be connected to single sliders, then depending on stereo_control, will get sent to left/right/both
  let output_gain = 0.0;
  let noise_gain = 0.0;
  let pre_smooth_gain = 0.5;
  let post_smooth_gain = 0.5;
  let bpfs: Array<BPF>;
  let clean = false;

  let stereo_params: StereoParams = init_channel_params(gains, freqs, Qs);

  function set_params_control(s: StereoControl) {
    if (s == "Left") {
      bpfs = [...stereo_params.left.bpfs];
      noise_gain = stereo_params.left.noise_gain;
      output_gain = stereo_params.left.output_gain;
      post_smooth_gain = stereo_params.left.post_smooth_gain;
      pre_smooth_gain = stereo_params.left.pre_smooth_gain;
    } else if (s === "Right") {
      bpfs = stereo_params.right.bpfs;
      noise_gain = stereo_params.right.noise_gain;
      output_gain = stereo_params.right.output_gain;
      post_smooth_gain = stereo_params.right.post_smooth_gain;
      pre_smooth_gain = stereo_params.right.pre_smooth_gain;
    } else if (s === "Both") {
      bpfs = stereo_params.both.bpfs;
      noise_gain = stereo_params.both.noise_gain;
      output_gain = stereo_params.both.output_gain;
      post_smooth_gain = stereo_params.both.post_smooth_gain;
      pre_smooth_gain = stereo_params.both.pre_smooth_gain;
    }
  }

  $: stereo_params.control, set_params_control(stereo_params.control);

  let bpf_hovering = Array(num_sliders).fill(false);
  let num_time_samples = 1;

  async function get_time_data(from_assets?: boolean) {
    if (selectedRecording === "") return;
    await invoke("get_time_data", {
      path: selectedRecording,
      fromAssets: from_assets,
    }).then((res) => {
      let data: any = res;
      // need to sync this and the downsample rate from the backend
      // or not...when it reads the correct number of samples
      num_time_samples = data.length;
    });
  }

  let time_slider_max = num_time_samples;
  $: num_time_samples, (time_slider_max = num_time_samples);

  onMount(async () => {
    // await message("Tauri is awesome", "Tauri");
    settings = await invoke("get_settings");

    update_css_color(rgbToHex(settings.colors.rotary_tick), "rotary-tick");
    update_css_color(rgbToHex(settings.colors.rotary_hover), "rotary-hover");
    update_css_color(rgbToHex(settings.colors.slider_border), "slider-border");
    update_css_color(
      rgbToHex(settings.colors.slider_indicator),
      "slider-indicator"
    );
    update_css_color(rgbToHex(settings.colors.slider_hover), "slider-hover");
    update_css_color(rgbToHex(settings.colors.slider_active), "slider-active");
    update_css_color(rgbToHex(settings.colors.plot_main), "plot-main");
    update_css_color(
      rgbToHex(settings.colors.plot_single_filter),
      "plot-single-filter"
    );
    update_css_color(
      rgbToHex(settings.colors.plot_total_curve),
      "plot-total-curve"
    );
    update_css_color(
      rgbToHex(settings.colors.plot_filter_hover),
      "plot-filter-hover"
    );

    selectedRecording = "reisman.wav";
    // selected recording also needs to be in sync with backend file...should be resolved once files are imported correctly instead of one by default, tho should still have that for loading saved state?
    change_file(selectedRecording, true);
    resetInterval();

    // load from server
    let b: Array<BPF> = await invoke("get_global_state");
    bpfs = b;
    console.log(bpfs, b);

    noise_gain = await invoke("get_noise_gain");
    output_gain = await invoke("get_output_gain");
    post_smooth_gain = await invoke("get_post_smooth_gain");
    pre_smooth_gain = await invoke("get_pre_smooth_gain");

    stereo_params.left = await invoke("get_channel_state", { channel: "Left" });
    stereo_params.right = await invoke("get_channel_state", {
      channel: "Right",
    });
    stereo_params.both = await invoke("get_channel_state", { channel: "Both" });

    stereo_params.control = await invoke("get_stereo_control");
  });

  $: stereo_params.control,
    () => {
      if (stereo_params.control === "Left") {
        bpfs = stereo_params.left.bpfs;
      } else if (stereo_params.control === "Right") {
        bpfs = stereo_params.right.bpfs;
      } else if (stereo_params.control === "Both") {
        bpfs = stereo_params.both.bpfs;
      }
    };

  function resetInterval() {
    clearInterval(interval);
    interval = setInterval(
      () => {
        if (is_playing) {
          perf_time = performance.now();
          time_delta = perf_time - time_origin;

          time += 10 / 1000;
          time_position = (time / DOWN_RATE) * SAMPLING_RATE;

          invoke("get_fft_plot_data").then((r: any) => {
            if (r.spectrum) {
              fft_data = r.spectrum;
            }
          });
        }
      },
      // this works for now, just have to call resetInterval after pressing button
      is_playing ? 10 : 1000
    );
  }
</script>

<main class="container" id="app-container">
  <div class="header">
    {#if show_settings}
      <Settings bind:settings bind:show_settings />
    {/if}
    <Plot
      bind:settings
      bind:bpf_hovering
      bind:is_playing
      bind:bpfs
      bind:selectedRecording
      {fft_data}
    />
    <input
      style="width: {FREQ_PLOT_WIDTH}px;"
      class="time-slider"
      type="range"
      data-attribute={is_time_slider_dragging}
      min={0}
      step={1}
      max={time_slider_max}
      bind:value={time_position}
      on:mousedown={() => {
        is_time_slider_dragging = true;
      }}
      on:mouseup={() => {
        is_time_slider_dragging = false;
      }}
      on:input={async () => {
        // time_position = (time / DOWN_RATE) * SAMPLING_RATE;
        // time_origin = time_position*DOWN_RATE/SAMPLING_RATE
        time = (time_position * DOWN_RATE) / SAMPLING_RATE;
        await invoke("update_time", {
          t: (time * SAMPLING_RATE) / num_time_samples / DOWN_RATE,
        });
      }}
    />
    <div class="button-bar">
      <div class="stereo-control-buttons">
        <button
          style="background: {stereo_params.control === 'Right'
            ? ''
            : 'var(--green)'}"
          on:click={() => {
            // also need to update ui to switch between left/right channel params
            if (stereo_params.control === "Left") {
              stereo_params.control = "Right";
              // set_params_control(stereo_params.control);
            } else if (stereo_params.control === "Right") {
              stereo_params.control = "Both";
              // set_params_control(stereo_params.control);
            } else if (stereo_params.control === "Both") {
              stereo_params.control = "Right";
              // set_params_control(stereo_params.control);
            }
            invoke("save_stereo_control", {
              stereoControl: stereo_params.control,
            });
          }}>L</button
        >
        <button
          style="background: {stereo_params.control === 'Left'
            ? ''
            : 'var(--green)'}"
          on:click={() => {
            if (stereo_params.control === "Left") {
              stereo_params.control = "Both";
              // set_params_control(stereo_params.control);
            } else if (stereo_params.control === "Right") {
              stereo_params.control = "Left";
              // set_params_control(stereo_params.control);
            } else if (stereo_params.control === "Both") {
              stereo_params.control = "Left";
              // set_params_control(stereo_params.control);
            }
            invoke("save_stereo_control", {
              stereoControl: stereo_params.control,
            });
          }}>R</button
        >
        control: {stereo_params.control}
      </div>
      <div class="stereo-mute-buttons">
        mute
        <button
          class="mute-button"
          data-attribute={stereo_params.left.mute}
          on:click={async () => {
            stereo_params.left.mute = !stereo_params.left.mute;
          }}
        >
          left
        </button>
        <button
          class="mute-button"
          data-attribute={stereo_params.right.mute}
          on:click={async () => {
            stereo_params.right.mute = !stereo_params.right.mute;
          }}
        >
          right
        </button>
      </div>
      <button
        on:click={async () => {
          if (!is_playing) {
            await invoke("play_stream");
            is_playing = true;
            time_origin = performance.now();
          } else {
            await invoke("pause_stream");
            is_playing = false;
          }
          resetInterval();
        }}
      >
        {is_playing ? "pause" : "play"}
      </button>
      <button
        on:click={async () => {
          clean = !clean;
          invoke("update_clean", { clean: clean });
        }}
      >
        {clean ? "dry" : "wet"}
      </button>
    </div>
  </div>

  <div
    class="filter-grid"
    style="grid-template-columns:repeat({num_sliders}, auto)"
  >
    {#each bpfs as _, i}
      <div
        class="bpf-wrap"
        role="button"
        tabindex="0"
        on:mouseenter={() => {
          bpf_hovering[i] = true;
        }}
        on:mouseleave={() => {
          bpf_hovering[i] = false;
        }}
      >
        <BandpassSlider
          bind:gain={bpfs[i].gain}
          bind:freq={bpfs[i].freq}
          bind:Q={bpfs[i].Q}
          bind:stereo_control={stereo_params.control}
          index={i + 1}
        />
      </div>
    {/each}
  </div>
  <div
    style="display: flex; flex-direction: row; justify-content: space-evenly;height:6em;"
  >
    <RotarySlider
      bind:value={noise_gain}
      index={-1}
      units="dB"
      label="noise gain"
      max_val={20}
      min_val={-80}
      update_backend={() => {
        invoke("update_noise_gain", {
          gain: noise_gain,
          stereoControl: stereo_params.control,
        });
      }}
      update_server={() => {
        invoke("save_noise_gain", {
          gain: noise_gain,
          stereoControl: stereo_params.control,
        });
      }}
    />
    <RotarySlider
      bind:value={pre_smooth_gain}
      index={-1}
      label="pre smooth"
      max_val={0.999}
      min_val={0.0}
      resolution={3}
      update_backend={() => {
        invoke("update_pre_smooth_gain", {
          gain: pre_smooth_gain,
          stereoControl: stereo_params.control,
        });
      }}
      update_server={() => {
        invoke("save_pre_smooth_gain", {
          gain: pre_smooth_gain,
          stereoControl: stereo_params.control,
        });
      }}
    />
    <RotarySlider
      bind:value={post_smooth_gain}
      index={-1}
      label="post smooth"
      max_val={0.999}
      min_val={0.0}
      resolution={3}
      update_backend={() => {
        invoke("update_post_smooth_gain", {
          gain: post_smooth_gain,
          stereoControl: stereo_params.control,
        });
      }}
      update_server={() => {
        invoke("save_post_smooth_gain", {
          gain: post_smooth_gain,
          stereoControl: stereo_params.control,
        });
      }}
    />
    <RotarySlider
      bind:value={output_gain}
      index={-1}
      units="dB"
      max_val={20}
      min_val={-20}
      label="output gain"
      update_backend={() => {
        invoke("update_output_gain", {
          gain: output_gain,
          stereoControl: stereo_params.control,
        });
      }}
      update_server={() => {
        invoke("save_output_gain", {
          gain: output_gain,
          stereoControl: stereo_params.control,
        });
      }}
    />
  </div>
  <div class="menu-bar">
    <button
      class="reset-all-gains-switch"
      title="reset all gains to 0 dB"
      on:click={() => {
        bpfs = [
          ...bpfs.map((filt, i) => {
            update_filters(
              i + 1,
              0.0,
              filt.freq,
              filt.Q,
              true,
              stereo_params.control
            );
            return { gain: 0.0, freq: filt.freq, Q: filt.Q };
          }),
        ];
      }}>reset gains</button
    >
    <button
      on:click={async () => {
        const selected = await open({
          multiple: false,
          filters: [
            {
              name: "audio",
              extensions: ["wav"],
            },
          ],
        });
        if (selected !== null && !(selected instanceof Array)) {
          change_file(selected);
        }
      }}
    >
      import file
    </button>
    <span
      title="full path: {selectedRecording}"
      style="position: absolute; bottom: 0; padding-right: 2em; align-self: center;"
      >current file ({stereo_params.is_stereo ? "stereo" : "mono"}): {remove_slashes_ext(
        selectedRecording
      )}</span
    >
    <div
      class="settings"
      role="button"
      tabindex="0"
      on:keypress={() => {}}
      on:click={() => {
        show_settings = !show_settings;
      }}
    />
  </div>
</main>

<style>
  input[type="range"] {
    appearance: none;
  }
  input[type="range"]::-webkit-slider-thumb {
    background: black;
    appearance: none;
    -webkit-appearance: none;
    height: 2em;
    width: 1em;
  }

  input[type="range"]::-webkit-slider-thumb:active {
    background: var(--slider-active);
  }
  input[type="range"][data-attribute="true"]::-webkit-slider-thumb {
    background: var(--slider-active);
  }

  input[type="range"]::-webkit-slider-runnable-track {
    background: var(--gray100);
  }

  .filter-grid {
    display: grid;
    justify-items: center;
    grid-template-rows: auto;
    appearance: none;
    margin-top: 5px;
    margin-bottom: 10px;
  }

  .time-slider {
    align-self: center;
    border: 2px solid var(--slider-border);
    transition: border 0.33s;
  }
  .time-slider[data-attribute="true"] {
    border: 2px solid var(--slider-active);
  }
  .time-slider:hover {
    border: 2px solid var(--slider-hover);
  }
  .bpf-wrap {
    display: flex;
    height: max-content;
  }

  .reset-all-gains-switch {
    font-size: 14px;
    align-items: center;
    height: 2em;
    border-radius: 0;
    padding: 0;
    margin: 0;
    align-self: center;
    width: max-content;
    border: 1px solid var(--rotary-tick);
    color: var(--gray200);
  }
  .reset-all-gains-switch:hover {
    color: var(--gray150);
  }
  .reset-all-gains-switch:active {
    color: var(--gray100);
  }
  .settings {
    background: url("/tool.svg");
    background-size: 100% 100%;
    width: 30px;
    height: 30px;
    display: inline-flex;
    transition: filter 0.33s;
    filter: invert(70%);
    cursor: pointer;
    outline: none;
  }
  .settings:hover {
    filter: invert(40%);
  }
  .button-bar {
    display: flex;
    justify-content: center;
    gap: 1em;
    padding-bottom: 3px;
    border-bottom: 1px solid var(--gray2);
  }
  .menu-bar {
    display: flex;
    justify-content: space-evenly;
    flex-grow: 1;
    align-items: center;
    border-top: 1px solid black;
    border-bottom: 1px solid black;
  }
  button {
    padding: 0 1em 0 1em;
    align-self: center;
  }
  button:disabled {
    text-decoration: line-through;
    filter: contrast(70%);
  }
  .stereo-control-buttons {
    position: absolute;
    left: 2.5%;
  }
  .stereo-mute-buttons {
    position: absolute;
    right: 2.5%;
  }
  .mute-button[data-attribute="true"] {
    background: var(--purple);
    text-decoration: line-through;
    border: 1px solid black;
  }
</style>
