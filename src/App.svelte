<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/api/dialog";
  import { onDestroy, onMount } from "svelte";

  import Plot from "./plot.svelte";
  import { DOWN_RATE, FREQ_PLOT_WIDTH, num_sliders } from "./constants.svelte";
  import BandpassSlider from "./bandpass-slider.svelte";
  import type {
    UIParams,
    StereoChoice,
    Filters,
    UIFilters,
    BPF,
  } from "./types.svelte";
  import {
    init_ui_params,
    remove_slashes_ext,
    update_css_color,
  } from "./functions.svelte";
  import RotarySlider from "./rotary-slider.svelte";
  import Settings from "./settings.svelte";

  let settings: any;
  let theme: any;
  let is_processing = false;
  let is_selecting_processing_choice = false;
  let processing_percentage = 0;

  let show_settings = false;
  // if these values are the same as what is in server, values will not update when loaded, so use values that are way out of range? silly but it works
  var gains = [0, 0, 0, 0, 0];
  var freqs = [100, 500, 1000, 2000, 5000];
  var Qs = [1, 1, 1, 1, 1];

  let time = 0;
  let time_position = 0;
  let selectedRecording = "";
  let time_data: Array<number>;
  let is_playing = false;
  let is_stereo = true;

  let is_time_slider_dragging = false;

  let fft_data: any;
  let bpfs: UIFilters = {
    bank: Array(num_sliders).flatMap((_) => {
      return { gain: 0, freq: 1000, Q: 1 };
    }),
  } as UIFilters;

  let ui_params: UIParams = init_ui_params(gains, freqs, Qs);

  const unlisten_file_drop = listen("tauri://file-drop", async (event: any) => {
    change_file(event.payload[0] as string);
  });
  const unlisten_audioui_message = listen(
    "update_processing_percentage",
    async (event: any) => {
      processing_percentage = event.payload as number;
    }
  );
  const unlisten_2 = listen("audioui_message", (event: any) => {
    fft_data = event.payload.spectrum;
    time = event.payload.time / num_time_samples;
    time_position = time * FREQ_PLOT_WIDTH;
  });
  onDestroy(() => {
    unlisten_2.then((f) => f());
    unlisten_audioui_message.then((f) => f());
    unlisten_file_drop.then((f) => f());
  });

  function change_file(path: string, from_assets?: boolean) {
    selectedRecording = path;
    invoke("message_file_path", { path: selectedRecording });
    // add this back?
    // invoke("get_is_stereo").then((r: any) => {
    // if (r.is_stereo !== undefined) {
    // is_stereo = r.is_stereo;
    // }
    // });
    get_time_data(from_assets);
  }

  async function get_ui_params(new_choice: StereoChoice) {
    // get from backend
    await invoke("sql_ui_params", { stereoChoice: new_choice }).then(
      async (r) => {
        ui_params = r as UIParams;
        ui_params.stereo_choice = new_choice;
        await invoke("sql_filter_bank", { stereoChoice: new_choice }).then(
          (res) => {
            const fb = res as UIFilters;
            bpfs = fb;
          }
        );
      }
    );
  }

  function set_ui_params() {
    let params: UIParams = {
      left_mute: false,
      right_mute: false,
      noise_gain: ui_params.noise_gain,
      output_gain: ui_params.output_gain,
      post_smooth_gain: ui_params.post_smooth_gain,
      pre_smooth_gain: ui_params.pre_smooth_gain,
      clean: ui_params.clean,
      stereo_choice: ui_params.stereo_choice,
      filters: bpfs,
    };
    invoke("sql_update_ui_params", {
      stereoChoice: ui_params.stereo_choice,
      uiParams: params,
    });
  }

  let bpf_hovering = Array(num_sliders).fill(false);
  let num_time_samples = 1;

  function get_time_data(from_assets?: boolean) {
    if (selectedRecording === "") return;
    invoke("get_time_data", {
      path: selectedRecording,
      fromAssets: from_assets,
    }).then((res) => {
      let data = res as Array<number>;
      time_data = data;
      num_time_samples = data.length * DOWN_RATE;
    });
  }

  onMount(async () => {
    await get_ui_params(ui_params.stereo_choice);
    settings = await invoke("sql_settings").catch(async (r) => {
      // await message("have to init settings", "denoiser");
      // await invoke("init_settings");
      // settings = await invoke("get_settings");
    });
    theme = await invoke("sql_theme", { theme: settings.theme });

    update_css_color(theme.rotary_tick, "rotary-tick");
    update_css_color(theme.rotary_hover, "rotary-hover");
    update_css_color(theme.slider_border, "slider-border");
    update_css_color(theme.slider_indicator, "slider-indicator");
    update_css_color(theme.slider_hover, "slider-hover");
    update_css_color(theme.slider_active, "slider-active");
    update_css_color(theme.plot_main, "plot-main");
    update_css_color(theme.plot_single_filter, "plot-single-filter");
    update_css_color(theme.plot_total_curve, "plot-total-curve");
    update_css_color(theme.plot_filter_hover, "plot-filter-hover");

    selectedRecording = "reisman.wav";
    // selected recording also needs to be in sync with backend file...should be resolved once files are imported correctly instead of one by default, tho should still have that for loading saved state?
    change_file(selectedRecording, true);
  });
</script>

<main class="container" id="app-container">
  <div class="header">
    {#if show_settings}
      <Settings bind:settings bind:show_settings bind:theme />
    {/if}
    {#if bpfs.bank.length === 5}
      <Plot
        bind:settings
        bind:theme
        bind:bpf_hovering
        bind:is_playing
        bind:bpfs={bpfs.bank}
        bind:time_data
        {fft_data}
      />
    {/if}
    <input
      style="width: {FREQ_PLOT_WIDTH}px;"
      class="time-slider"
      type="range"
      data-attribute={is_time_slider_dragging}
      min={0}
      max={FREQ_PLOT_WIDTH}
      bind:value={time_position}
      on:mousedown={() => {
        is_time_slider_dragging = true;
      }}
      on:mouseup={() => {
        is_time_slider_dragging = false;
      }}
      on:input={() => {
        time = time_position / FREQ_PLOT_WIDTH;
        // only send message when playing, otherwise after dragging the slider, the first message received is the location it started in
        is_playing &&
          invoke("message_time", {
            time: time * num_time_samples,
          });
      }}
    />
    <div class="button-bar">
      <div class="stereo-control-buttons">
        <button
          class="stereo-control-button"
          data-attribute={ui_params.stereo_choice !== "Right"}
          on:click={() => {
            // also need to update ui to switch between left/right channel params
            set_ui_params();
            if (ui_params.stereo_choice === "Left") {
              get_ui_params("Right");
            } else if (ui_params.stereo_choice === "Right") {
              get_ui_params("Both");
            } else if (ui_params.stereo_choice === "Both") {
              get_ui_params("Right");
            }
          }}>L</button
        >
        <button
          class="stereo-control-button"
          data-attribute={ui_params.stereo_choice !== "Left"}
          on:click={() => {
            set_ui_params();
            if (ui_params.stereo_choice === "Left") {
              get_ui_params("Both");
            } else if (ui_params.stereo_choice === "Right") {
              get_ui_params("Both");
            } else if (ui_params.stereo_choice === "Both") {
              get_ui_params("Left");
            }
          }}>R</button
        >
        control: {ui_params.stereo_choice}
      </div>
      <div class="stereo-mute-buttons">
        mute
        <button
          class="mute-button"
          data-attribute={ui_params.left_mute}
          on:click={() => {
            ui_params.left_mute = !ui_params.left_mute;
            invoke("sql_update_left_mute", {
              stereoChoice: ui_params.stereo_choice,
              leftMute: ui_params.left_mute,
            });
            invoke("message_left_mute", {
              stereoChoice: ui_params.stereo_choice,
              mute: ui_params.left_mute,
            });
          }}
        >
          L
        </button>
        <button
          class="mute-button"
          data-attribute={ui_params.right_mute}
          on:click={() => {
            ui_params.right_mute = !ui_params.right_mute;
            invoke("sql_update_right_mute", {
              stereoChoice: ui_params.stereo_choice,
              rightMute: ui_params.right_mute,
            });
            invoke("message_right_mute", {
              stereoChoice: ui_params.stereo_choice,
              mute: ui_params.right_mute,
            });
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
            invoke("play_stream").then(() => {
              time = time_position / FREQ_PLOT_WIDTH;
              invoke("message_time", {
                time: time * num_time_samples,
              });
            });
            is_playing = true;
          } else {
            await invoke("pause_stream");
            is_playing = false;
          }
        }}
      >
        {is_playing ? "pause" : "play"}
      </button>
      <button
        on:click={async () => {
          ui_params.clean = !ui_params.clean;
          invoke("message_clean", {
            stereoChoice: ui_params.stereo_choice,
            clean: ui_params.clean,
          });
        }}
      >
        {ui_params.clean ? "dry" : "wet"}
      </button>
    </div>
  </div>

  <div
    class="filter-grid"
    style="grid-template-columns:repeat({num_sliders}, auto)"
  >
    {#if bpfs.bank.length === 5}
      {#each bpfs.bank as _, i}
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
            bind:gain={bpfs.bank[i].gain}
            bind:freq={bpfs.bank[i].freq}
            bind:Q={bpfs.bank[i].Q}
            bind:stereo_choice={ui_params.stereo_choice}
            index={i + 1}
          />
        </div>
      {/each}
    {/if}
  </div>
  <div
    style="display: flex; flex-direction: row; justify-content: space-evenly;height:6em;"
  >
    <RotarySlider
      bind:value={ui_params.noise_gain}
      units="dB"
      label="noise gain"
      max_val={20}
      min_val={-80}
      update_database={() => {
        invoke("sql_update_noise_gain", {
          stereoChoice: ui_params.stereo_choice,
          noiseGain: ui_params.noise_gain,
        });
      }}
      update_backend={() => {
        invoke("message_noise_gain", {
          stereoChoice: ui_params.stereo_choice,
          gain: ui_params.noise_gain,
        });
      }}
    />
    <RotarySlider
      bind:value={ui_params.pre_smooth_gain}
      label="pre smooth"
      max_val={0.999}
      min_val={0.0}
      resolution={3}
      update_database={() => {
        invoke("sql_update_pre_smooth_gain", {
          stereoChoice: ui_params.stereo_choice,
          preSmoothGain: ui_params.pre_smooth_gain,
        });
      }}
      update_backend={() => {
        invoke("message_pre_smooth_gain", {
          stereoChoice: ui_params.stereo_choice,
          gain: ui_params.pre_smooth_gain,
        });
      }}
    />
    <RotarySlider
      bind:value={ui_params.post_smooth_gain}
      label="post smooth"
      max_val={0.999}
      min_val={0.0}
      resolution={3}
      update_database={() => {
        invoke("sql_update_post_smooth_gain", {
          stereoChoice: ui_params.stereo_choice,
          postSmoothGain: ui_params.post_smooth_gain,
        });
      }}
      update_backend={() => {
        invoke("message_post_smooth_gain", {
          stereoChoice: ui_params.stereo_choice,
          gain: ui_params.post_smooth_gain,
        });
      }}
    />
    <RotarySlider
      bind:value={ui_params.output_gain}
      units="dB"
      max_val={20}
      min_val={-20}
      label="output gain"
      update_database={() => {
        invoke("sql_update_output_gain", {
          stereoChoice: ui_params.stereo_choice,
          outputGain: ui_params.output_gain,
        });
      }}
      update_backend={() => {
        invoke("message_output_gain", {
          stereoChoice: ui_params.stereo_choice,
          gain: ui_params.output_gain,
        });
      }}
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
        // bpfs.bank = [
        //   ...bpfs.bank.map((filt, i) => {
        //     invoke("message_filters", {
        //       stereoChoice: ui_params.stereo_choice,
        //       index: i + 1,
        //       gain: 0.0,
        //       freq: filt.freq,
        //       q: filt.Q,
        //     });
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
          is_selecting_processing_choice = !is_selecting_processing_choice;
        }
      }}
    >
      {!is_processing ? "export file" : "exporting.."}
    </button>
    {#if is_selecting_processing_choice}
      <button
        style="position: absolute; bottom: 40px;right:30px;"
        title="apply parameters for 'Both' to left and right"
        on:click={() => {
          if (!is_processing) {
            is_processing = true;
            is_playing = false;
            invoke("process_export", { stereoChoice: "Both" }).then(() => {
              is_processing = false;
            });
          }
          is_selecting_processing_choice = false;
        }}
      >
        Both
      </button>
      <button
        style="position: absolute; bottom: 10px;right:30px;"
        title="apply parameters for left and right independently"
        on:click={() => {
          if (!is_processing) {
            is_processing = true;
            is_playing = false;
            invoke("process_export", { stereoChoice: "Left" }).then(() => {
              is_processing = false;
            });
          }
          is_selecting_processing_choice = false;
        }}
      >
        L/R
      </button>
    {/if}
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
      >current file ({is_stereo ? "stereo" : "mono"}): {remove_slashes_ext(
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
  }
  .mute-button[data-attribute="true"] {
    background: black;
    text-decoration: line-through;
    filter: contrast(70%);
  }
  .stereo-control-button[data-attribute="true"] {
    background: var(--rotary-tick);
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
