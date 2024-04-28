<script lang="ts">
	import Slider from "./slider.svelte";
	import RotarySlider from "./rotary-slider.svelte";
	import FreqSlider from "./freq-slider.svelte";
	import type { StereoControl } from "./types.svelte";

	export let gain = 0;
	export let freq = 1000;
	export let Q = 1;
	export let index: number;
	export let stereo_control: StereoControl;

	// $: gain, update_filters(index, gain, freq, Q, true, stereo_control);
	// $: freq, update_filters(index, gain, freq, Q, true, stereo_control);
	// $: Q, update_filters(index, gain, freq, Q, true, stereo_control);
</script>

<div class="wrapper">
	<div
		style="display:flex; height:4em; justify-content: space-evenly; margin-bottom: 0.5em;"
	>
		<RotarySlider bind:value={Q} update_server={() => {}} />
		<Slider bind:value={gain} bind:index />
	</div>
	<FreqSlider bind:value={freq} bind:index />
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
