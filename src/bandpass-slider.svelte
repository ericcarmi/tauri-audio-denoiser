<script lang="ts">
	import Slider from "./slider.svelte";
	import RotarySlider from "./rotary-slider.svelte";
	import { invoke } from "@tauri-apps/api/tauri";
	import { update_filters } from "./functions.svelte";
	import FreqSlider from "./freq-slider.svelte";
	import type { StereoControl } from "./types.svelte";

	export let gain = 0;
	export let freq = 1000;
	export let Q = 1;
	export let index: number;
	export let stereo_control: StereoControl;

</script>

<div class="wrapper">
	<div
		style="display:flex; height:4em; justify-content: space-evenly; margin-bottom: 0.5em;"
	>
		<RotarySlider
			bind:value={Q}
			bind:index
			update_server={() => {
				invoke("save_bpf_Q", {
					q: Q,
					index: index,
					stereoControl: stereo_control,
				});
			}}
		/>
		<Slider
			bind:value={gain}
			bind:index
			update_server={() => {
				invoke("save_bpf_gain", {
					gain: gain,
					index: index,
					stereoControl: stereo_control,
				});
			}}
		/>
	</div>
	<FreqSlider
		bind:value={freq}
		bind:index
		update_server={() => {
			invoke("save_bpf_freq", {
				freq: freq,
				index: index,
				stereoControl: stereo_control,
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
