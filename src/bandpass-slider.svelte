<script lang="ts">
	import Slider from "./slider.svelte";
	import RotarySlider from "./rotary-slider.svelte";
	import { invoke } from "@tauri-apps/api/tauri";
	import { biquad } from "./functions.svelte";
	import FreqSlider from "./freq-slider.svelte";

	export let gain = 0;
	export let freq = 1000;
	export let Q = 1;

	let is_freq_dragging = false;

	export let index: number;

	let bypass = false;

	// one param changes all coefficients, so this goes here instead of inside individual sliders
	function update() {
		let b = biquad(gain, freq, Q);
		b.x = [0, 0];
		b.y = [0, 0];
		if (index == 1) {
			invoke("update_filters", { bp1: b });
		} else if (index == 2) {
			invoke("update_filters", { bp2: b });
		} else if (index == 3) {
			invoke("update_filters", { bp3: b });
		} else if (index == 4) {
			invoke("update_filters", { bp4: b });
		} else if (index == 5) {
			invoke("update_filters", { bp5: b });
		}
	}
	$: gain, update();
	$: freq, update();
	$: Q, update();
</script>

<div class="wrapper">
	<div style="display:flex; height:4em; justify-content: space-evenly;">
		<RotarySlider bind:value={Q} bind:index />
		<Slider bind:value={gain} bind:index />
	</div>
	<FreqSlider
		bind:value={freq}
		bind:index
		update_server={() => {
			invoke("save_bpf_freq", { gain: freq, index: index });
		}}
	/>
	<div style="display: flex; justify-content: space-evenly;">
		<button
			class="switch"
			title="bypass: {bypass ? 'on' : 'off'}"
			data-attribute={bypass}
			on:click={() => {
				invoke("update_bypass", { bypass: bypass, index: index - 1 });
				bypass = !bypass;
			}}
		/>

		<button
			class="reset-gain-switch"
			title="reset to 0 dB"
			on:click={() => {
				gain = 0;
			}}
		/>
	</div>
</div>

<style>
	.wrapper {
		display: flex;
		flex-direction: column;
	}
</style>
