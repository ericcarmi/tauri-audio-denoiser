<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onDestroy, onMount } from "svelte";
  import { shortcut } from "./shortcut.svelte";
  import type { ComponentColors, PlotScale, Settings, Theme } from "./types.ts";

  let fft_plot_decay = 0.8;
  let fft_plot_size = 256;
  let draw_fft_amp_axis = true;
  let draw_filter_amp_axis = true;
  let draw_freq_axis = true;

  let plot_scale: PlotScale;

  export let settings: Settings;
  export let show_settings;

  export let theme: ComponentColors;
  let theme_name: Theme;

  let on_top = false;
  let rotary_tick: string;
  let rotary_hover: string;
  let slider_border: string;
  let slider_indicator: string;
  let slider_background: string;
  let slider_hover: string;
  let slider_active: string;
  let plot_main: string;
  let plot_single_filter: string;
  let plot_total_curve: string;
  let plot_filter_hover: string;
  let app_background: string;
  let app_text: string;
  let button_background: string;
  let button_text: string;

  function update_local_colors() {
    rotary_tick = theme.rotary_tick;
    rotary_hover = theme.rotary_hover;
    slider_border = theme.slider_border;
    slider_background = theme.slider_background;
    slider_indicator = theme.slider_indicator;
    slider_hover = theme.slider_hover;
    slider_active = theme.slider_active;
    plot_main = theme.plot_main;
    plot_total_curve = theme.plot_total_curve;
    plot_single_filter = theme.plot_single_filter;
    plot_filter_hover = theme.plot_filter_hover;
    app_background = theme.app_background;
    app_text = theme.app_text;
    button_background = theme.button_background;
    button_text = theme.button_text;

    plot_scale = settings.plot_scale;
  }

  onMount(async () => {
    if (settings) {
      theme_name = settings.theme;
      update_local_colors();
    } else {
      return;
    }
  });
  onDestroy(async () => {
    // await invoke("save_settings", { settings: settings });
  });

  $: draw_fft_amp_axis, (settings.draw_fft_amp_axis = draw_fft_amp_axis);
  $: draw_filter_amp_axis,
    (settings.draw_filter_amp_axis = draw_filter_amp_axis);
  $: draw_freq_axis, (settings.draw_freq_axis = draw_freq_axis);

  // to make this more compact, need types from rust? don't want to just copy/paste...ts-rs is a thing, not sure if that would generate code or what
  $: rotary_tick, update_color(rotary_tick, Object.keys({ rotary_tick })[0]);
  $: rotary_hover, update_color(rotary_hover, Object.keys({ rotary_hover })[0]);
  $: slider_border,
    update_color(slider_border, Object.keys({ slider_border })[0]);
  $: slider_background,
    update_color(slider_background, Object.keys({ slider_background })[0]);
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
  $: plot_filter_hover,
    update_color(plot_filter_hover, Object.keys({ plot_filter_hover })[0]);

  $: app_background,
    update_color(app_background, Object.keys({ app_background })[0]);
  $: app_text, update_color(app_text, Object.keys({ app_text })[0]);
  $: button_background,
    update_color(button_background, Object.keys({ button_background })[0]);
  $: button_text, update_color(button_text, Object.keys({ button_text })[0]);

  function update_color(color: string, color_name: string) {
    if (color !== undefined) {
      if (color_name === "app_background") {
        document.body.style.setProperty("background", color);
      } else if (color_name === "app_text") {
        document.body.style.setProperty("color", color);
      }
      document.body.style.setProperty(
        `--${color_name.replace("_", "-")}`,
        color,
      );
      let s = color_name as keyof ComponentColors;
      theme[s] = color;
    }
  }

  let ref: any;
</script>

<div class="grid-wrap" bind:this={ref} data-attribute={on_top}>
  <div class="title-bar">
    <span class="title">settings</span>
    <div
      style="display: flex; width: 100%; position: absolute; justify-content: flex-end; align-items: center;height: 100%; gap: 1em;"
    >
      <!--
      <div
        class="top-button"
        role="button"
        tabindex="0"
        on:keypress={() => {}}
        on:click={async () => {
          await invoke("init_settings");
          settings = await invoke("get_settings");
          update_local_colors();
        }}
      >
        reset
      </div> 
-->

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
      <button
        class="close-button"
        use:shortcut={{
          code: "Escape",
          callback: () => {
            show_settings = false;
          },
        }}
        on:click={() => {
          show_settings = false;
          ref && ref.parentNode.removeChild(ref);
        }}
      >
        close
      </button>
    </div>
  </div>
  {#if settings}
    <div class="wrapper">
      <div class="item">
        <span class="group-label">plot scale</span>
        <span class="check-label"
          ><input
            type="radio"
            name="plot_scale"
            on:click={() => {
              plot_scale = "Linear";
              settings.plot_scale = plot_scale;
            }}
            checked={plot_scale === "Linear"}
          /> linear</span
        >
        <span class="check-label"
          ><input
            type="radio"
            name="plot_scale"
            on:click={() => {
              plot_scale = "Mel";
              settings.plot_scale = plot_scale;
            }}
            checked={plot_scale === "Mel"}
          /> mel</span
        >
        <span class="check-label"
          ><input
            type="radio"
            name="plot_scale"
            on:click={() => {
              plot_scale = "Log";
              settings.plot_scale = plot_scale;
            }}
            checked={plot_scale === "Log"}
          /> log</span
        >
        <span class="check-label"
          ><input
            type="radio"
            name="plot_scale"
            on:click={() => {
              plot_scale = "Bark";
              settings.plot_scale = plot_scale;
            }}
            checked={plot_scale === "Bark"}
          /> bark</span
        >
      </div>

      <div class="item">
        <span class="group-label">fft plot decay</span>
        <input
          type="range"
          name="fft_plot_decay"
          min={0}
          max={1}
          step={0.01}
          bind:value={fft_plot_decay}
        />
        <span style="width:100%;">{fft_plot_decay}</span>
        <span class="check-label"
          ><input
            type="checkbox"
            on:click={() => {
              draw_fft_amp_axis = !draw_fft_amp_axis;
            }}
            checked={draw_fft_amp_axis}
          />fft amp axis</span
        >

        <span class="check-label"
          ><input
            type="checkbox"
            on:click={() => {
              draw_filter_amp_axis = !draw_filter_amp_axis;
            }}
            checked={draw_filter_amp_axis}
          />filter amp axis</span
        >

        <span class="check-label"
          ><input
            type="checkbox"
            on:click={() => {
              draw_freq_axis = !draw_freq_axis;
            }}
            checked={draw_freq_axis}
          />frequency axis</span
        >
      </div>

      <div class="item">
        <span class="group-label">fft plot size</span>
        <span class="check-label"
          ><input
            type="radio"
            name="fft_plot_size"
            on:click={() => {
              fft_plot_size = 64;
            }}
            checked={fft_plot_size === 64}
          /> 64</span
        >
        <span class="check-label"
          ><input
            type="radio"
            name="fft_plot_size"
            on:click={() => {
              fft_plot_size = 128;
            }}
            checked={fft_plot_size === 128}
          /> 128</span
        >
        <span class="check-label"
          ><input
            type="radio"
            name="fft_plot_size"
            on:click={() => {
              fft_plot_size = 256;
            }}
            checked={fft_plot_size === 256}
          /> 256</span
        >
        <span class="check-label"
          ><input
            type="radio"
            name="fft_plot_size"
            on:click={() => {
              fft_plot_size = 512;
            }}
            checked={fft_plot_size === 512}
          /> 512</span
        >
      </div>

      <div class="item">
        <span class="group-label">theme</span>
        <span class="check-label"
          ><input
            type="radio"
            name="theme"
            on:click={async () => {
              theme_name = "RGB";
              settings.theme = theme_name;
              theme = await invoke("sql_theme", {
                theme: theme_name,
              });
              update_local_colors();
            }}
            checked={theme_name === "RGB"}
          /> rgb</span
        >
        <span class="check-label"
          ><input
            type="radio"
            name="theme"
            on:click={async () => {
              theme_name = "CYM";
              settings.theme = theme_name;
              theme = await invoke("sql_theme", {
                theme: theme_name,
              });
              update_local_colors();
            }}
            checked={theme_name === "CYM"}
          /> cym</span
        >
        <span class="check-label"
          ><input
            type="radio"
            name="theme"
            on:click={async () => {
              theme_name = "POG";
              settings.theme = theme_name;
              theme = await invoke("sql_theme", {
                theme: theme_name,
              });
              update_local_colors();
            }}
            checked={theme_name === "POG"}
          /> pog</span
        >
        <span class="check-label"
          ><input
            type="radio"
            name="theme"
            on:click={async () => {
              theme_name = "BWG";
              settings.theme = theme_name;
              theme = await invoke("sql_theme", {
                theme: theme_name,
              });
              update_local_colors();
            }}
            checked={theme_name === "BWG"}
          /> bwg</span
        >
        <span class="check-label"
          ><input
            type="radio"
            name="theme"
            on:click={async () => {
              theme_name = "SEPIA";
              settings.theme = theme_name;
              theme = await invoke("sql_theme", {
                theme: theme_name,
              });
              update_local_colors();
            }}
            checked={theme_name === "SEPIA"}
          /> sepia</span
        >
        <span class="check-label"
          ><input
            type="radio"
            name="theme"
            on:click={async () => {
              theme_name = "CUSTOM";
              settings.theme = theme_name;
              theme = await invoke("sql_theme", {
                theme: theme_name,
              });
              update_local_colors();
            }}
            checked={theme_name === "CUSTOM"}
          /> custom</span
        >
        <button
          type="button"
          title="this will not erase the custom theme"
          on:click={async () => {
            await invoke("init_settings");
            // settings = await invoke("get_settings");
            update_local_colors();
          }}>reset to defaults</button
        >
      </div>

      <div class="wide-item">
        <span class="group-label">colors</span>
        <span
          ><input
            style="--col: {rotary_tick};"
            type="color"
            on:change={() => {}}
            bind:value={rotary_tick}
          />rotary ticks</span
        >
        <span
          ><input
            style="--col: {rotary_hover};"
            type="color"
            bind:value={rotary_hover}
          />rotary hover</span
        >
        <span
          ><input
            style="--col: {slider_border};"
            type="color"
            bind:value={slider_border}
          />slider border</span
        >
        <span
          ><input
            style="--col: {slider_indicator};"
            type="color"
            bind:value={slider_indicator}
          />slider indicator</span
        >
        <span
          ><input
            style="--col: {slider_background};"
            type="color"
            bind:value={slider_background}
          />slider background</span
        >
        <span
          ><input
            style="--col: {slider_hover};"
            type="color"
            bind:value={slider_hover}
          />slider hover</span
        >
        <span
          ><input
            style="--col: {slider_active};"
            type="color"
            bind:value={slider_active}
          />slider active</span
        >
        <span
          ><input
            style="--col: {plot_main};"
            type="color"
            bind:value={plot_main}
          />plot main</span
        >
        <span
          ><input
            style="--col: {plot_single_filter};"
            type="color"
            bind:value={plot_single_filter}
          />plot single filter
        </span>
        <span
          ><input
            style="--col: {plot_total_curve};"
            type="color"
            bind:value={plot_total_curve}
          />plot total curve</span
        >
        <span
          ><input
            style="--col: {plot_filter_hover};"
            type="color"
            bind:value={plot_filter_hover}
          />plot filter hover</span
        >

        <span
          ><input
            style="--col: {app_background};"
            type="color"
            bind:value={app_background}
          />app background</span
        >
        <span
          ><input
            style="--col: {app_text};"
            type="color"
            bind:value={app_text}
          />app text</span
        >
        <span
          ><input
            style="--col: {button_background};"
            type="color"
            bind:value={button_background}
          />button background</span
        >
        <span
          ><input
            style="--col: {button_text};"
            type="color"
            bind:value={button_text}
          />button text</span
        >

        <!--
      this displays all but colors won't update, the bind isn't right...cuz it iterates over theme instead of local vars?
        {#each Object.entries(theme) as [name, color]}
          <span
            ><input
              style="--col: {name};"
              type="color"
              bind:value={color}
            />{name.replace("_", " ")}</span
          >
        {/each}
        -->
      </div>
    </div>
  {/if}
</div>

<style>
  .check-label {
    display: flex;
    align-items: center;
    gap: 3px;
  }
  .grid-wrap {
    display: flex;
    justify-content: center;
    height: 50%;
    width: calc(100% - 1px);
    position: absolute;
    top: 0px;
    z-index: 2;
    transition: top 1s;
  }
  .grid-wrap[data-attribute="true"] {
    top: 50%;
  }
  .wrapper {
    display: grid;
    flex-direction: row;
    border: 1px solid black;
    width: 100%;
    background: rgb(from var(--gray2) r g b / 0.8);
    border: 1px solid var(--gray6);
    color: var(--gray8);
    gap: 1em;
    grid-template-rows: auto auto;
    grid-template-columns: auto auto auto;
    justify-items: center;
    height: calc(100% - 2em);
    padding-top: 2em;
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
    grid-template-columns: auto auto auto;
    align-items: center;
    justify-items: start;
    gap: 0.3em;
  }
  .group-label {
    text-decoration: underline;
    grid-column: 1 / span 3;
    align-self: center;
    justify-self: center;
  }

  input[type="color"] {
    border: 0px;
    padding: 0;
    /*background: lch(from var(--col) calc(l + 20) calc(c + 200) calc(h + 180deg));*/
    background: hsl(
      from var(--col) calc(h + 180deg) calc(s * 0.3) calc(l + 30%)
    );
  }

  .top-button {
    background: black;
    color: white;
    cursor: pointer;
    padding: 1px;
    justify-self: flex-end;
    width: max-content;
    transition: color 0.3s;
    border: 1px solid var(--orange);
  }
  .top-button:hover {
    color: var(--gray150);
  }
  .close-button {
    background: black;
    color: white;
    cursor: pointer;
    padding: 1px;
    width: max-content;
    transition: color 0.3s;
    border: 1px solid var(--orange);
  }
  .close-button:hover {
    color: var(--gray150);
  }
  input[type="range"]::-webkit-slider-runnable-track {
    background: var(--gray100);
    border: 1px solid var(--gray7);
  }
  input[type="range"]::-webkit-slider-thumb {
    background: black;
    appearance: none;
    -webkit-appearance: none;
    height: 2em;
    width: 1em;
  }
  input[type="range"] {
    appearance: none;
    -webkit-appearance: none;
  }
  input[type="range"]:active::-webkit-slider-thumb {
    background: var(--slider-active);
  }
  .title-bar {
    position: absolute;
    width: 100%;
    display: flex;
    justify-content: center;
    border-bottom: 1px solid var(--gray5);
    height: 2em;
  }
  .title {
    color: var(--orange);
    letter-spacing: 0.4em;
    font-weight: bold;
    align-self: center;
  }
</style>
