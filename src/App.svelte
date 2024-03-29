<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/tauri";
  import { resolveResource } from "@tauri-apps/api/path";
  import { readTextFile, readBinaryFile } from "@tauri-apps/api/fs";
    import { onMount } from "svelte";

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
</script>

<main class="container">

  <button
    on:click={async () => {
      await invoke("play_wav", { path: "/Users/eric/Music/heinlein.wav" });
    }}
  >
    play
  </button>

  <button
    on:click={async () => {
      await invoke("stop");
    }}
  >
    stop
  </button>

  <button
    on:click={async () => {
      await invoke("update_filters");
    }}
  >
    update
  </button>

  <input
    type="range"
    min={0}
    max={1000}
    bind:value={alpha}
    on:input={async () => {
      await invoke("update_filters", { alpha: alpha / 1000 });
    }}
  />
</main>

<style>
</style>
