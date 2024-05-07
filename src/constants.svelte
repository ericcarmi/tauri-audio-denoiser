<script context="module" lang="ts">
	import { invoke } from "@tauri-apps/api/tauri";
	// this doesn't work, can't be run in here, returns promise
	export async function get_num_filters() {
		let n = await invoke("get_num_filters");
		return n as number
	}
	export const FREQ_PLOT_WIDTH = 950;
	export const FREQ_PLOT_HEIGHT = 300;
	export const TIME_PLOT_WIDTH = FREQ_PLOT_WIDTH;
	export const TIME_PLOT_HEIGHT = 50;
	export const SAMPLING_RATE = 44100;
	export const DOWN_RATE = 16;
	export const NYQUIST = 44100 / 2;
	export const MIN_FREQ = 20;
	export const MAX_FREQ = 20000;
</script>
