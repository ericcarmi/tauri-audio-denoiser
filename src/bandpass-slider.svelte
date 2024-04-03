<script lang="ts">
	import Slider from "./slider.svelte";
	import RotarySlider from "./rotary-slider.svelte";
	import { invoke } from "@tauri-apps/api/tauri";
	import { biquad } from "./functions.svelte";
	import type { BPF } from "./types.svelte";

	export let gain = 0;
	export let freq = 1000;
	export let Q = 1;

	let is_freq_dragging = false;

	export let bpf: BPF = { gain: gain, freq: freq, Q: Q };

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
	<input
		class="freq-slider"
		type="range"
		min={100}
		max={20000}
		step={0.001}
		bind:value={freq}
		data-attribute={is_freq_dragging}
		on:mouseup={() => {
			invoke("save_bpf_freq", { freq: freq, index: index });
			is_freq_dragging = false;
		}}
		on:mousedown={() => {
			is_freq_dragging = true;
		}}
	/>
	<button
		class="switch"
		data-attribute={bypass}
		on:click={() => {
			invoke("update_bypass", { bypass: bypass, index: index - 1 });
			bypass = !bypass;
		}}
	/>
</div>

<style>
	.wrapper {
		display: flex;
		flex-direction: column;
	}

	.switch {
		display: flex;
		width: 1em;
		height: 1em;
		padding: 0;
		border-radius: 50%;
		background: var(--purple);
		align-self: center;
		transition: background 0.33s, border-color 0.33s;
		border-color: var(--gray100);
	}
	.switch[data-attribute="true"]:hover {
		border-color: var(--lightpurple);
	}
	.switch:hover {
		border-color: var(--gray150);
	}

	.switch[data-attribute="true"] {
		background: var(--gray150);
		border-color: var(--purple);
	}

	input[type="range"] {
		appearance: none;
	}
	input[type="range"]::-webkit-slider-thumb {
		background: black;
		appearance: none;
		-webkit-appearance: none;
		height: 2em;
		width: 1em;
	}

	input[type="range"]::-webkit-slider-thumb:active {
		background: var(--purple);
	}
	input[type="range"][data-attribute="true"]::-webkit-slider-thumb {
		background: var(--purple);
	}

	input[type="range"]::-webkit-slider-runnable-track {
		background: var(--gray100);
	}

	.freq-slider {
		border: 1px solid var(--purple);
		transition: border 0.33s;
		width: 12em;
	}
	.freq-slider[data-attribute="true"] {
		border: 1px solid var(--lightpurple);
	}
	.freq-slider:hover {
		border: 1px solid var(--lightpurple);
	}
</style>
