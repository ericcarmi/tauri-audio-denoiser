<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/tauri";
  import { resolveResource } from "@tauri-apps/api/path";
  import { readTextFile, readBinaryFile } from "@tauri-apps/api/fs";
  import { onMount } from "svelte";
  import TimePlot from "./time-plot.svelte";

  listen("tauri://file-drop", async (event) => {
    // console.log(event.payload);
    // need to handle this getting history of all files dropped...
    // let s = event.payload[0] as string;
    // await invoke("play_wav", { path: s });
  });

  onMount(async () => {
    const resourcePath = await resolveResource("assets/test-file.wav");
    // const langDe = JSON.parse(await readTextFile(resourcePath));
    // console.log(langDe);
  });

  let alpha = 500;
  let time = 0;
</script>

<main class="container">
  <TimePlot selectedRecording="test-file.wav" />
  <input
    style="width: 100%;"
    type="range"
    min={0}
    max={100000}
    bind:value={time}
    on:input={async () => {
      await invoke("update_time", { t: time / 100000 });
    }}
  />
  <span>time</span>
  <div style="display:flex;">
    <button
      on:click={async () => {
        await invoke("play_stream");
      }}
    >
      play
    </button>

    <button
      on:click={async () => {
        await invoke("pause_stream");
      }}
    >
      pause
    </button>
  </div>

  <div style="display:flex">
    <span>filter coeff</span>
    <input
      style="width: 100%"
      type="range"
      min={0}
      max={1000}
      bind:value={alpha}
      on:input={async () => {
        await invoke("update_filters", { alpha: alpha / 1000 });
      }}
    />
  </div>

</main>

<style>
  button {
    width: 100%;
  }
</style>
