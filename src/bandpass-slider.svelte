<script lang="ts">
	import Slider from "./slider.svelte";
	import RotarySlider from "./rotary-slider.svelte";


	export let gain = 0;
	export let freq = 1000;
	export let Q = 1;
</script>

<div class="wrapper">
	<div style="display:flex; height:4em; justify-content: space-evenly;">
		<RotarySlider bind:value={Q} />
		<Slider bind:value={gain} />
	</div>
	<input class="freq-slider" type="range" min={40} max={20000} step={1} bind:value={freq} />
</div>

<style>
	.wrapper {
		display: flex;
		flex-direction: column;
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

	input[type="range"]::-webkit-slider-runnable-track {
		background: var(--gray100);
	}

	.freq-slider {
		border: 1px solid var(--purple);
		transition: border 0.33s;
		width: 10em;
	}
	.freq-slider[data-attribute="true"] {
		border: 1px solid var(--lightpurple);
	}
	.freq-slider:hover {
		border: 1px solid var(--lightpurple);
	}
</style>
