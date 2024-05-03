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
  import type {
    UIParams,
    StereoChoice,
    StereoParams,
    FilterBank,
    UIFilterBank,
  } from "./types.svelte";
  import {
    init_channel_params,
    remove_slashes_ext,
    rgbToHex,
    update_css_color,
  } from "./functions.svelte";
  import RotarySlider from "./rotary-slider.svelte";
  import Settings from "./settings.svelte";

  let settings: any;
  let theme: any;
  let is_backend_params_initialized = false;
  let is_processing = false;
  let processing_percentage = 0;
  let show_settings = false;
  // if these values are the same as what is in server, values will not update when loaded, so use values that are way out of range? silly but it works
  var gains = [0, 0, 0, 0, 0];
  var freqs = [1000, 1000, 1000, 1000, 1000];
  var Qs = [1, 1, 1, 1, 1];

  const unlisten = listen("tauri://file-drop", async (event: any) => {
    change_file(event.payload[0] as string);
  });

  const unlisten_audioui_message = listen(
    "message_processing_percentage",
    async (event: any) => {
      processing_percentage = event.payload as number;
    }
  );

  function change_file(path: string, from_assets?: boolean) {
    selectedRecording = path;
    invoke("message_file_path", { path: selectedRecording });
    // invoke("get_is_stereo").then((r: any) => {
    //   if (r.is_stereo !== undefined) {
    //     stereo_params.is_stereo = r.is_stereo;
    //   }
    // });
    // invoke("get_stereo_choice").then((r: any) => {
    //   if (r !== undefined) {
    //     stereo_params.stereo_choice = r;
    //   }
    // });
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
  // keep this values to be connected to single sliders, then depending on stereo_choice, will get sent to left/right/both
  let output_gain = 0.0;
  let noise_gain = 0.0;
  let pre_smooth_gain = 0.5;
  let post_smooth_gain = 0.5;
  let bpfs: any;
  let clean = false;

  let stereo_params: StereoParams = init_channel_params(gains, freqs, Qs);
  let ui_params: UIParams;

  function get_ui_params(s: StereoChoice) {
    // get from backend
    invoke("sql_ui_params", { stereoChoice: s }).then((r) => {
      ui_params = r as UIParams;
      invoke("sql_filter_bank", { stereoChoice: s }).then((res) => {
        const fb = res as FilterBank;
        bpfs = [fb.bp1, fb.bp2, fb.bp3, fb.bp4, fb.bp5];
      });

      noise_gain = ui_params.noise_gain;
      output_gain = ui_params.output_gain;
      post_smooth_gain = ui_params.post_smooth_gain;
      pre_smooth_gain = ui_params.pre_smooth_gain;
    });
  }

  function set_ui_params(s: StereoChoice, ui: UIParams) {
    let params: UIParams = {
      left_mute: false,
      right_mute: false,
      noise_gain: noise_gain,
      output_gain: output_gain,
      post_smooth_gain: post_smooth_gain,
      pre_smooth_gain: pre_smooth_gain,
      clean: clean,
      stereo_choice: "Both",
      filter_bank: {
        bp1: bpfs[0],
        bp2: bpfs[1],
        bp3: bpfs[2],
        bp4: bpfs[3],
        bp5: bpfs[4],
      } as UIFilterBank,
    };
    invoke("sql_update_ui_params", {
      stereoChoice: s,
      uiParams: params,
    });
  }

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
    settings = await invoke("sql_settings").catch(async (r) => {
      // await message("have to init settings", "denoiser");
      // await invoke("init_settings");
      // settings = await invoke("get_settings");
    });
    theme = await invoke("sql_theme", { theme: settings.theme });

    update_css_color(rgbToHex(theme.rotary_tick), "rotary-tick");
    update_css_color(rgbToHex(theme.rotary_hover), "rotary-hover");
    update_css_color(rgbToHex(theme.slider_border), "slider-border");
    update_css_color(rgbToHex(theme.slider_indicator), "slider-indicator");
    update_css_color(rgbToHex(theme.slider_hover), "slider-hover");
    update_css_color(rgbToHex(theme.slider_active), "slider-active");
    update_css_color(rgbToHex(theme.plot_main), "plot-main");
    update_css_color(rgbToHex(theme.plot_single_filter), "plot-single-filter");
    update_css_color(rgbToHex(theme.plot_total_curve), "plot-total-curve");
    update_css_color(rgbToHex(theme.plot_filter_hover), "plot-filter-hover");

    selectedRecording = "reisman.wav";
    // selected recording also needs to be in sync with backend file...should be resolved once files are imported correctly instead of one by default, tho should still have that for loading saved state?
    change_file(selectedRecording, true);
    resetInterval();

    get_ui_params(stereo_params.stereo_choice);

    // await invoke("init_audio_params_from_server");
  });

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

        if (is_processing) {
          let r = invoke("get_audioui_message").then((r: any) => {
            if (r.processing_percentage) {
              processing_percentage = r.processing_percentage;
            }
          });
        }
      },
      // this works for now, just have to call resetInterval after pressing button
      is_playing || is_processing ? 0.1 : 1000
    );
  }
</script>

<main class="container" id="app-container">
  <div class="header">
    {#if show_settings}
      <Settings bind:settings bind:show_settings bind:theme />
    {/if}
    {#if bpfs?.length === 5}
      <Plot
        bind:settings
        bind:theme
        bind:bpf_hovering
        bind:is_playing
        bind:bpfs
        bind:selectedRecording
        {fft_data}
      />
    {/if}
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
        await invoke("message_time", {
          t: (time * SAMPLING_RATE) / num_time_samples / DOWN_RATE,
        });
      }}
    />
    <div class="button-bar">
      <div class="stereo-control-buttons">
        <button
          class="stereo-control-button"
          data-attribute={stereo_params.stereo_choice !== "Right"}
          on:click={() => {
            // also need to update ui to switch between left/right channel params
            set_ui_params(stereo_params.stereo_choice, ui_params);
            if (stereo_params.stereo_choice === "Left") {
              stereo_params.stereo_choice = "Right";
            } else if (stereo_params.stereo_choice === "Right") {
              stereo_params.stereo_choice = "Both";
            } else if (stereo_params.stereo_choice === "Both") {
              stereo_params.stereo_choice = "Right";
            }
            get_ui_params(stereo_params.stereo_choice);
          }}>L</button
        >
        <button
          class="stereo-control-button"
          data-attribute={stereo_params.stereo_choice !== "Left"}
          on:click={() => {
            set_ui_params(stereo_params.stereo_choice, ui_params);
            if (stereo_params.stereo_choice === "Left") {
              stereo_params.stereo_choice = "Both";
            } else if (stereo_params.stereo_choice === "Right") {
              stereo_params.stereo_choice = "Left";
            } else if (stereo_params.stereo_choice === "Both") {
              stereo_params.stereo_choice = "Left";
            }
            get_ui_params(stereo_params.stereo_choice);
          }}>R</button
        >
        control: {stereo_params.stereo_choice}
      </div>
      <div class="stereo-mute-buttons">
        mute
        <button
          class="mute-button"
          data-attribute={stereo_params.left.ui_params?.left_mute}
          on:click={() => {
            // stereo_params.left.ui_params.left_mute = !stereo_params.left.ui_params.left_mute;
          }}
        >
          L
        </button>
        <button
          class="mute-button"
          data-attribute={stereo_params.right.ui_params?.right_mute}
          on:click={() => {
            // stereo_params.right.ui_params.right_mute = !stereo_params.right.ui_params.right_mute;
          }}
        >
          R
        </button>
      </div>
      <button
        style="animation: {is_playing
          ? '1s linear infinite alternate button-border-pulse'
          : 'none'}"
        on:click={async () => {
          if (!is_playing) {
            await invoke("play_stream");
            is_playing = true;
            time_origin = performance.now();
            if (!is_backend_params_initialized) {
              is_backend_params_initialized = true;
              // invoke("init_audio_params_from_server");
            }
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
        }}
      >
        {clean ? "dry" : "wet"}
      </button>

      <button
        on:click={() => {
          // invoke("sql_create");
          // invoke("sql_update");
          let s = invoke("sql_theme", { theme: "RGB" }).then((r) => {});
        }}
      >
        database
      </button>
    </div>
  </div>

  {#if bpfs?.length === 5}
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
            bind:stereo_choice={stereo_params.stereo_choice}
            index={i + 1}
          />
        </div>
      {/each}
    </div>
  {/if}
  <div
    style="display: flex; flex-direction: row; justify-content: space-evenly;height:6em;"
  >
    <RotarySlider
      bind:value={noise_gain}
      units="dB"
      label="noise gain"
      max_val={20}
      min_val={-80}
      update_backend={() => {}}
      update_server={() => {}}
    />
    <RotarySlider
      bind:value={pre_smooth_gain}
      label="pre smooth"
      max_val={0.999}
      min_val={0.0}
      resolution={3}
      update_backend={() => {}}
      update_server={() => {}}
    />
    <RotarySlider
      bind:value={post_smooth_gain}
      label="post smooth"
      max_val={0.999}
      min_val={0.0}
      resolution={3}
      update_backend={() => {}}
      update_server={() => {}}
    />
    <RotarySlider
      bind:value={output_gain}
      units="dB"
      max_val={20}
      min_val={-20}
      label="output gain"
      update_backend={() => {}}
      update_server={() => {}}
    />
  </div>
  <div class="menu-bar">
    <div
      class="settings"
      role="button"
      tabindex="0"
      on:keypress={() => {}}
      on:click={() => {
        show_settings = !show_settings;
      }}
    />
    <button
      class="reset-all-gains-switch"
      title="reset all gains to 0 dB"
      on:click={() => {
        // bpfs = [
        //   ...bpfs.map((filt, i) => {
        //     message_filters(
        //       i + 1,
        //       0.0,
        //       filt.freq,
        //       filt.Q,
        //       true,
        //       stereo_params.control
        //     );
        //     return { gain: 0.0, freq: filt.freq, Q: filt.Q };
        //   }),
        // ];
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
    <button
      on:click={() => {
        if (!is_processing) {
          is_processing = true;
          is_playing = false;
          // resetInterval();
          invoke("process_export").then((r) => {
            is_processing = false;
            // resetInterval();
          });
        }
      }}
    >
      {!is_processing ? "export file" : "exporting.."}
    </button>
    {#if is_processing}
      <div
        style="position: absolute; right: 30px; height: 3em; width: 3em; display: flex; align-items: center; justify-content: center; "
      >
        <span style="font-size: 12px; ">{processing_percentage.toFixed(0)}</span
        >
        <div class="spinner" />
      </div>
    {/if}
    <span
      title="full path: {selectedRecording}"
      style="position: absolute; bottom: 0; padding-right: 2em; align-self: center;"
      >current file ({stereo_params.is_stereo ? "stereo" : "mono"}): {remove_slashes_ext(
        selectedRecording
      )}</span
    >
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
  .stereo-control-buttons {
    position: absolute;
    left: 2.5%;
  }
  .stereo-mute-buttons {
    position: absolute;
    right: 2.5%;
  }
  .mute-button {
    background: var(--rotary-tick);
    border: 1px solid black;
  }
  .mute-button[data-attribute="true"] {
    background: black;
    text-decoration: line-through;
    border: 1px solid black;
    filter: contrast(70%);
  }
  .stereo-control-button[data-attribute="true"] {
    background: var(--purple);
    border: 1px solid black;
    filter: contrast(100%);
  }
  .stereo-control-button {
    filter: contrast(70%);
  }
  .spinner {
    height: 100%;
    width: 100%;
    border-radius: 100%;
    position: absolute;
  }
</style>
