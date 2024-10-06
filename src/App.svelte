<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/api/dialog";
  import { onDestroy, onMount } from "svelte";

  import FreqPlot from "./freq-plot.svelte";
  import {
    DOWN_RATE,
    FREQ_PLOT_WIDTH,
    TIME_PLOT_WIDTH,
    get_num_filters,
  } from "./constants.svelte";
  import BandpassSlider from "./bandpass-slider.svelte";
  import type { UIParams, StereoChoice, UIFilters, Settings } from "./types.ts";
  import {
    init_ui_params,
    remove_slashes_ext,
    resetGains,
    update_css_color,
  } from "./functions.svelte";
  import RotarySlider from "./rotary-slider.svelte";
  import SettingsMenu from "./settings.svelte";
  import TimePlot from "./time-plot.svelte";

  let num_sliders = 0;
  let settings: Settings;
  let theme: any;
  let is_processing = false;
  let is_selecting_processing_choice = false;
  let processing_percentage = 0;
  let time_position = 0;
  let time = 0;
  let num_time_samples = 1;
  let time_data: Array<number> = [];
  let sampling_rate: number = 44100;

  let loop_start_time = 0;
  let loop_length = 0;

  let time_hover_position = 0;

  let show_settings = false;
  var gains = [0, 0, 0, 0, 0];
  var freqs = [100, 500, 1000, 2000, 5000];
  var Qs = [1, 1, 1, 1, 1];

  let selectedRecording = "";
  let is_playing = false;
  let is_stereo = true;

  let fft_data: any;
  let ui_params: UIParams = init_ui_params(gains, freqs, Qs);

  const unlisten_file_drop = listen("tauri://file-drop", async (event: any) => {
    change_file(event.payload[0] as string);
  });
  const unlisten_audioui_message = listen(
    "update_processing_percentage",
    async (event: any) => {
      processing_percentage = event.payload as number;
    },
  );
  const unlisten_samplerate_message = listen(
    "update_sampling_rate",
    async (event: any) => {
      sampling_rate = event.payload as number;
    },
  );
  onDestroy(() => {
    unlisten_audioui_message.then((f) => f());
    unlisten_file_drop.then((f) => f());
    unlisten_samplerate_message.then((f) => f());
  });

  function change_file(path: string, from_assets?: boolean) {
    selectedRecording = path;
    invoke("message_file_path", { path: selectedRecording });
    // add this back?
    // invoke("get_is_stereo").then((r: any) => {
    //   if (r.is_stereo !== undefined) {
    //     is_stereo = r.is_stereo;
    //   }
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
            // bpfs = fb;
            ui_params.filters = fb;
          },
        );
      },
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
      filters: ui_params.filters,
    };
    invoke("sql_update_ui_params", {
      stereoChoice: ui_params.stereo_choice,
      uiParams: params,
    });
  }

  let bpf_hovering = Array(num_sliders).fill(false);

  function get_time_data(from_assets?: boolean) {
    if (selectedRecording === "") return;
    invoke("get_time_data", {
      path: selectedRecording,
      fromAssets: from_assets,
    }).then((res) => {
      let data = res as Array<number>;
      // this is stereo, it is interleaved
      time_data = data.filter(function (element, index, array) {
        return index % 2 === 0;
      });

      // does it need to be halved? not if only one channel is displayed...duh

      if (is_stereo) {
        num_time_samples = time_data.length;
        // console.log("yes");
      } else {
        num_time_samples = time_data.length;
      }
    });
  }

  onMount(async () => {
    num_sliders = await get_num_filters();
    await get_ui_params(ui_params.stereo_choice);
    settings = await invoke("sql_settings").catch(async (_r) => {
      // await message("have to init settings", "denoiser");
      await invoke("init_settings");
      settings = await invoke("get_settings");
    }) as Settings;
    theme = await invoke("sql_theme", { theme: settings.theme });

    update_css_color(theme.rotary_tick, "rotary-tick");
    update_css_color(theme.rotary_hover, "rotary-hover");
    update_css_color(theme.slider_border, "slider-border");
    update_css_color(theme.slider_indicator, "slider-indicator");
    update_css_color(theme.slider_background, "slider-background");
    update_css_color(theme.slider_hover, "slider-hover");
    update_css_color(theme.slider_active, "slider-active");
    update_css_color(theme.plot_main, "plot-main");
    update_css_color(theme.plot_single_filter, "plot-single-filter");
    update_css_color(theme.app_background, "app-background");
    update_css_color(theme.app_text, "app-text");
    update_css_color(theme.button_background, "button-background");
    update_css_color(theme.button_text, "button-text");
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
      <SettingsMenu bind:settings bind:show_settings bind:theme />
    {/if}
    {#if ui_params.filters.bank.length === num_sliders}
      <FreqPlot
        bind:settings={settings}
        bind:sampling_rate
        bind:theme
        bind:bpf_hovering
        bind:num_sliders
        bind:is_playing
        bind:bpfs={ui_params.filters.bank}
        {fft_data}
      />
    {/if}
    {#if theme}
      <TimePlot
        bind:time_position
        bind:sampling_rate
        bind:time
        bind:time_data
        bind:num_time_samples
        bind:is_playing
        bind:hover_position={time_hover_position}
        bind:plot_color={theme.plot_main}
        bind:loop_length
        bind:loop_start_time
      />
    {/if}
    <div class="button-bar">
      <div class="stereo-buttons-box">
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
                get_ui_params("Left");
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
                get_ui_params("Left");
              } else if (ui_params.stereo_choice === "Both") {
                get_ui_params("Right");
              }
            }}>R</button
          >
          <span style="margin:0em 1em;">
            control: {ui_params.stereo_choice}</span
          >
        </div>
        <div class="stereo-mute-buttons">
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
          <span style="margin:0em 1em;">mute</span>
        </div>
      </div>
      <button
        style="animation: {is_playing
          ? '1s linear infinite alternate button-border-pulse'
          : 'none'}"
        on:click={async () => {
          if (!is_playing) {
            invoke("play_stream").then(() => {
              time = time_position / TIME_PLOT_WIDTH;
              invoke("message_time", {
                time: time * num_time_samples,
              });

              invoke("message_all", {
                stereoChoice: ui_params.stereo_choice,
                leftMute: ui_params.left_mute,
                rightMute: ui_params.right_mute,
                noiseGain: ui_params.noise_gain,
                outputGain: ui_params.output_gain,
                postSmoothGain: ui_params.post_smooth_gain,
                preSmoothGain: ui_params.pre_smooth_gain,
                clean: ui_params.clean,
                filters: ui_params.filters.bank,
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
      <button
        id="loop-button"
        on:click={async () => {
          if (!is_playing) {
            invoke("play_stream").then(() => {
              console.log(
                loop_start_time / sampling_rate,
                loop_length / sampling_rate,
              );

              invoke("message_loop_time", {
                loopTime: loop_start_time,
                loopLength: loop_length,
              });

              invoke("message_all", {
                stereoChoice: ui_params.stereo_choice,
                leftMute: ui_params.left_mute,
                rightMute: ui_params.right_mute,
                noiseGain: ui_params.noise_gain,
                outputGain: ui_params.output_gain,
                postSmoothGain: ui_params.post_smooth_gain,
                preSmoothGain: ui_params.pre_smooth_gain,
                clean: ui_params.clean,
                filters: ui_params.filters.bank,
              });
            });
            is_playing = true;
          } else {
            await invoke("pause_stream");
            is_playing = false;
          }
        }}>loop</button
      >
      <button
        on:click={() => {
          invoke("message_fingerprint", {
            start: Math.round(loop_start_time / sampling_rate),
            len: Math.round(loop_length / sampling_rate),
          });
        }}>fingerprint</button
      >
      <div
        style="display: flex; flex-direction: column;  flex-grow: 1; justify-content: space-evenly"
      >
        <span style="align-self: center;"
          >cursor: {(
            ((time_hover_position / TIME_PLOT_WIDTH) * num_time_samples) /
            sampling_rate
          ).toFixed(9)}</span
        >
        <span
          >time: {(
            ((time_position / TIME_PLOT_WIDTH) * num_time_samples) /
            sampling_rate
          ).toFixed(1)}</span
        >
      </div>
    </div>
  </div>

  <div
    class="filter-grid"
    style="grid-template-columns:repeat({num_sliders}, auto)"
  >
    {#if ui_params.filters.bank.length === num_sliders}
      {#each ui_params.filters.bank as _, i}
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
            bind:gain={ui_params.filters.bank[i].gain}
            bind:freq={ui_params.filters.bank[i].freq}
            bind:Q={ui_params.filters.bank[i].Q}
            bind:stereo_choice={ui_params.stereo_choice}
            index={i}
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
        ui_params.filters = resetGains(ui_params);
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
        selectedRecording,
      )}</span
    >
  </div>
</main>

<style>
  .filter-grid {
    display: grid;
    justify-items: center;
    grid-template-rows: auto;
    appearance: none;
    margin-top: 5px;
    margin-bottom: 10px;
  }

  .bpf-wrap {
    display: flex;
    height: max-content;
  }

  .settings {
    background: url("/tool.svg");
    background-size: 100% 100%;
    width: 30px;
    height: 30px;
    display: inline-flex;
    transition: filter 0.33s;
    filter: drop-shadow(0 0 10px var(--app-text));
    cursor: pointer;
    outline: none;
  }
  .settings:hover {
    filter: drop-shadow(
      0,
      0,
      10px,
      rgb(from var(--app-text) calc(r), calc(b), calc(g))
    );
  }
  .button-bar {
    display: flex;
    justify-content: space-evenly;
    gap: 1em;
    height: 5em;
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
    align-self: center;
  }
  .stereo-buttons-box {
    display: flex;
    flex-direction: column;
    width: 17em;
  }
  .stereo-control-buttons {
    margin-top: 0.2em;
    align-self: flex-start;
  }
  .stereo-mute-buttons {
    margin-top: 1em;
    align-self: flex-start;
  }
  .mute-button[data-attribute="true"] {
    background: black;
    color: white;
    border: 1px solid var(--rotary-tick);
    text-decoration: line-through;
  }
  .stereo-control-button {
    background: black;
    color: white;
    border: 1px solid black;
  }
  .stereo-control-button[data-attribute="true"] {
    background: var(--button-background);
    color: var(--button-text);
  }
  .stereo-control-button[data-attribute="true"]:hover {
    border-color: var(--sepia0);
  }
  .spinner {
    height: 100%;
    width: 100%;
    border-radius: 100%;
    position: absolute;
  }
</style>
