<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/tauri";

  listen("tauri://file-drop", async (event) => {
    console.log(event.payload);
    // need to handle this getting history of all files dropped...
    let s = event.payload[0] as string;
    await invoke("play_wav", { path: s });
  });

</script>

<main class="container">
  <div>drop zone</div>

  <button
  on:click={async() => {
    await invoke("play_wav", { path: '/Users/eric/Music/heinlein.wav' });
  }}
  > play </button>

  <button
  on:click={async() => {
    await invoke("stop");
  }}
  > stop </button>

  <button
  on:click={async() => {
    await invoke("update_filters");
  }}
  > update </button>
</main>

<style>
</style>
