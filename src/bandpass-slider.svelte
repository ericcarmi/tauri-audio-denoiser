<script lang="ts">
	import Slider from "./slider.svelte";
	import RotarySlider from "./rotary-slider.svelte";
	import ZoomableSlider from "./zoomable-slider.svelte";
	import type { StereoChoice } from "./types.svelte";
	import { invoke } from "@tauri-apps/api/tauri";

	export let gain = 0;
	export let freq = 1000;
	export let Q = 1;
	export let index: number;
	export let stereo_choice: StereoChoice;

	const update_db = () => {
		invoke("sql_update_filter_bank", {
			stereoChoice: stereo_choice,
			bpf: { gain: gain, freq: freq, Q: Q },
			index: index,
		});
	};
</script>

<div class="wrapper">
	<div
		style="display:flex; height:4em; justify-content: space-evenly; margin-bottom: 0.5em;"
	>
		<RotarySlider
			bind:value={Q}
			update_database={update_db}
			update_backend={() => {
				invoke("message_filters", {
					stereoChoice: stereo_choice,
					index: index,
					gain: gain,
					freq: freq,
					q: Q,
				});
			}}
		/>
		<Slider
			bind:value={gain}
			bind:index
			update_database={update_db}
			update_backend={() => {
				invoke("message_filters", {
					stereoChoice: stereo_choice,
					index: index,
					gain: gain,
					freq: freq,
					q: Q,
				});
			}}
		/>
	</div>
	<ZoomableSlider
		bind:value={freq}
		bind:index
		update_database={update_db}
		update_backend={() => {
			invoke("message_filters", {
				stereoChoice: stereo_choice,
				index: index,
				gain: gain,
				freq: freq,
				q: Q,
			});
		}}
	/>
	<button
		title="reset to 0 dB"
		on:click={() => {
			gain = 0;
		}}
		>rst gain
	</button>
</div>

<style>
	button {
		align-self: center;
		margin-top: 0.4em;
	}
	.wrapper {
		display: flex;
		flex-direction: column;
	}
</style>
