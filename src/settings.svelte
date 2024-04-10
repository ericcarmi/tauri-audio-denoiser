<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onDestroy, onMount } from "svelte";
  import { hexToRgb, rgbToHex } from "./functions.svelte";

  let fft_plot_decay = 0.8;
  let fft_plot_size = 256;

  // this will need to rewrite redis.conf...right?
  let server_update_interval = 30;
  let server_update_num_changes = 5;

  let plot_scale: string;

  export let settings: any;
  export let show_settings;

  let on_top = false;
  let rotary_tick: string;
  let rotary_hover: string;
  let slider_border: string;
  let slider_indicator: string;
  let slider_hover: string;
  let slider_active: string;
  let plot_main: string;
  let plot_single_filter: string;
  let plot_total_curve: string;

  onMount(async () => {
    if (settings) {
      rotary_tick = rgbToHex(settings.colors.rotary_tick);
      plot_total_curve = rgbToHex(settings.colors.plot_total_curve);
      plot_single_filter = rgbToHex(settings.colors.plot_single_filter);
      plot_scale = settings.plot_scale;
    }
  });
  onDestroy(async () => {
    await invoke("save_settings", { settings: settings });
  });

  $: plot_scale, update_plot_scale();

  function update_plot_scale() {
    if (settings && plot_scale) {
      settings.plot_scale = plot_scale;
    }
  }

  // to make this more compact, need types from rust? don't want to just copy/paste...ts-rs is a thing, not sure if that would generate code or what
  $: rotary_tick, update_color(rotary_tick, Object.keys({ rotary_tick })[0]);
  $: rotary_hover, update_color(rotary_hover, Object.keys({ rotary_hover })[0]);
  $: slider_border,
    update_color(slider_border, Object.keys({ slider_border })[0]);
  $: slider_indicator,
    update_color(slider_indicator, Object.keys({ slider_indicator })[0]);
  $: slider_hover, update_color(slider_hover, Object.keys({ slider_hover })[0]);
  $: slider_active,
    update_color(slider_active, Object.keys({ slider_active })[0]);
  $: plot_main, update_color(plot_main, Object.keys({ plot_main })[0]);
  $: plot_single_filter,
    update_color(plot_single_filter, Object.keys({ plot_single_filter })[0]);
  $: plot_total_curve,
    update_color(plot_total_curve, Object.keys({ plot_total_curve })[0]);

  function update_color(color: string, color_name: string) {
    if (color !== undefined) {
      document.body.style.setProperty(
        `--${color_name.replace("_", "-")}`,
        color
      );
      settings.colors[color_name] = hexToRgb(color);
    }
  }
  let ref: any;
</script>

<div style="display: flex;" bind:this={ref}>
  {#if settings}
    <div class="wrapper" data-attribute={on_top}>
      <div
        class="top-button"
        role="button"
        tabindex="0"
        on:keypress={() => {}}
        on:click={() => {
          on_top = !on_top;
        }}
      >
        move to {on_top ? "top" : "btm"}
      </div>
      <div
        class="close-button"
        role="button"
        tabindex="0"
        on:keypress={() => {}}
        on:click={() => {
          show_settings = false;
          ref && ref.parentNode.removeChild(ref);
        }}
      >
        close
      </div>
      <div class="item">
        <span class="group-label">plot scale</span>
        <span
          ><input
            type="radio"
            name="plot_scale"
            on:click={() => {
              plot_scale = "Linear";
            }}
            checked={plot_scale === "Linear"}
          /> linear</span
        >
        <span
          ><input
            type="radio"
            name="plot_scale"
            on:click={() => {
              plot_scale = "Mel";
            }}
            checked={plot_scale === "Mel"}
          /> mel</span
        >
        <span
          ><input
            type="radio"
            name="plot_scale"
            on:click={() => {
              plot_scale = "Log";
            }}
            checked={plot_scale === "Log"}
          /> log</span
        >
        <span
          ><input
            type="radio"
            name="plot_scale"
            on:click={() => {
              plot_scale = "Bark";
            }}
            checked={plot_scale === "Bark"}
          /> bark</span
        >
      </div>

      <div class="item">
        <span class="group-label">fft plot decay</span>
        <input type="range" name="fft_plot_decay" value={fft_plot_decay} />
        <span style="width:100%;">{fft_plot_decay}</span>
      </div>

      <div class="item">
        <span class="group-label">fft plot size</span>
        <span><input type="radio" name="fft_plot_size" /> 64</span>
        <span><input type="radio" name="fft_plot_size" /> 128</span>
        <span><input type="radio" name="fft_plot_size" checked /> 256</span>
        <span><input type="radio" name="fft_plot_size" /> 512</span>
      </div>

      <div class="item">
        <span class="group-label">theme</span>
        <span><input type="radio" name="theme" /> rgb</span>
        <span><input type="radio" name="theme" /> cym</span>
        <span><input type="radio" name="theme" /> pog</span>
        <span><input type="radio" name="theme" /> custom</span>
      </div>

      <div class="item">
        <span class="group-label">database update</span>
        <input
          type="range"
          name="server_update_interval"
          value={server_update_interval}
        />
        <span style="width:100%;">{server_update_interval}</span>
        <input
          type="range"
          name="server_update_num_changes"
          value={server_update_num_changes}
        />
        <span style="width:100%;">{server_update_num_changes}</span>
      </div>

      <div class="wide-item">
        <span class="group-label">colors</span>
        <span
          ><input
            type="color"
            on:change={(e) => {
              // console.log(e.currentTarget.value);
            }}
            bind:value={rotary_tick}
          />rotary ticks</span
        >
        <span
          ><input
            type="color"
            value={rgbToHex(settings.colors.rotary_tick)}
          />rotary hover</span
        >
        <span
          ><input
            type="color"
            value={rgbToHex(settings.colors.slider_border)}
          />slider border</span
        >
        <span
          ><input
            type="color"
            value={rgbToHex(settings.colors.slider_indicator)}
          />slider indicator</span
        >
        <span
          ><input
            type="color"
            value={rgbToHex(settings.colors.slider_hover)}
          />slider hover</span
        >
        <span
          ><input
            type="color"
            value={rgbToHex(settings.colors.slider_active)}
          />slider active</span
        >
        <span
          ><input
            type="color"
            value={rgbToHex(settings.colors.plot_main)}
          />plot main</span
        >
        <span
          ><input type="color" bind:value={plot_single_filter} />plot single
          filter
        </span>
        <span
          ><input type="color" bind:value={plot_total_curve} />plot total curve</span
        >
      </div>
    </div>
  {/if}
</div>

<style>
  .wrapper {
    position: absolute;
    display: grid;
    flex-direction: row;
    top: 45px;
    left: 25px;
    border: 1px solid black;
    width: 95%;
    background: rgba(130, 130, 130, 0.8);
    color: black;
    gap: 1em;
    grid-template-rows: auto auto;
    grid-template-columns: auto auto auto;
    justify-items: center;
    z-index: 1;
    transition: top 1s;
  }
  .wrapper[data-attribute="true"] {
    top: 50%;
  }
  .item {
    display: flex;
    flex-direction: column;
    align-items: start;
    text-align: center;
  }
  .wide-item {
    display: grid;
    text-align: center;
    grid-template-rows: auto auto auto;
    grid-template-columns: auto auto;
    align-items: start;
    justify-items: start;
    gap: 0.3em;
  }
  .group-label {
    text-decoration: underline;
    grid-column: 1 / span 2;
    align-self: center;
    justify-self: center;
  }

  input[type="color"] {
    border: 0px;
    padding: 0;
    background: black;
  }

  .top-button {
    background: black;
    color: white;
    cursor: pointer;
    position: absolute;
    top: 0px;
    right: 0px;
    width: max-content;
    transition: color 0.3s;
    border: 1px solid var(--rotary-tick);
  }
  .top-button:hover {
    color: var(--gray150);
  }
  .close-button {
    background: black;
    color: white;
    cursor: pointer;
    position: absolute;
    top: 35px;
    right: 0px;
    width: max-content;
    transition: color 0.3s;
    border: 1px solid var(--rotary-tick);
  }
  .close-button:hover {
    color: var(--gray150);
  }
</style>
