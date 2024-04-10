<script lang="ts">
	import { invoke } from "@tauri-apps/api/tauri";
	import { onMount } from "svelte";

	export let value: any;
	export let index: number;
	export let label = "";
	export let update_backend = () => {};
	export let update_server = () => {};

	export let min_val = 0.1;
	export let max_val = 10;
	let vals_inverted = Math.sign(max_val - min_val);
	$: min_val, (vals_inverted = Math.sign(max_val - min_val));
	$: max_val, (vals_inverted = Math.sign(max_val - min_val));

	let el: HTMLElement;
	let indicator_el: HTMLElement;
	let is_mouse_down = false;

	let radius = 12;

	let angle = 225;
	$: value, redraw(), update_backend();

	function redraw() {
		if (!is_mouse_down && indicator_el !== undefined) {
			angle = (-(value - max_val) / Math.abs(max_val - min_val)) * 270 - 45;
			const x = radius * Math.cos((angle * Math.PI) / 180);
			const y = -radius * Math.sin((angle * Math.PI) / 180);

			indicator_el.style.transform = `scale(0.1) translate(${x}em, ${y}em)`;
			indicator_el.style.background = `linear-gradient(${
				90 - angle
			}deg, var(--gray50) 0%,  var(--purple) 100%)`;
			let a = (1 - (angle + 45) / 270) * 40;
			let highlight = Math.round(a);
			colors = Array(num_indicators)
				.fill(0)
				.map((_, idx) => {
					if (highlight > idx) {
						return "var(--rotary-tick)";
					} else {
						return "black";
					}
				});
		}
	}

	function draggable() {
		if (el === null) {
			return;
		}
		el.addEventListener("mousedown", function (e: MouseEvent) {
			function mouseMoveHandler(e: any) {
				if (el === null || indicator_el === null) {
					return;
				}
				if (!is_mouse_down) return;

				angle += (-e.movementX + e.movementY) * 3;
				angle = Math.max(Math.min(angle, 225), -45);

				value = max_val - ((angle + 45) / 270) * Math.abs(max_val - min_val);

				const x = radius * Math.cos((angle * Math.PI) / 180);
				const y = -radius * Math.sin((angle * Math.PI) / 180);

				indicator_el.style.transform = `scale(0.1) translate(${x}em, ${y}em)`;
				indicator_el.style.background = `linear-gradient(${
					90 - angle
				}deg, var(--gray50) 0%,  var(--purple) 100%)`;
				let a = (1 - (angle + 45) / 270) * 40;
				let highlight = Math.round(a);
				colors = Array(num_indicators)
					.fill(0)
					.map((_, idx) => {
						if (highlight > idx) {
							return "var(--rotary-tick)";
						} else {
							return "black";
						}
					});
				// if you want to color circle also...but ticks are enough for now
				// let b = Math.round((a / 40) * 50);
				// el.style.background = `radial-gradient(var(--gray150) ${b}% ,var(--gray100) 100%)`;
			}

			function reset() {
				// have to call this here...maybe want to change how this is handled later
				is_mouse_down = false;
				// needs to be lowercase here...tauri does that
				if (index !== -1) invoke("save_bpf_Q", { q: value, index: index });
				else {
					update_server();
				}

				window.removeEventListener("mousemove", mouseMoveHandler);
				window.removeEventListener("mouseup", reset);
			}
			window.addEventListener("mousemove", mouseMoveHandler);
			window.addEventListener("mouseup", reset);
		});
	}
	onMount(() => {
		draggable();
		const x = radius * Math.cos((angle * Math.PI) / 180);
		const y = -radius * Math.sin((angle * Math.PI) / 180);
		indicator_el.style.transform = `scale(0.1) translate(${x}em, ${y}em)`;
		indicator_el.style.background = `linear-gradient(${
			90 - angle
		}deg, var(--gray50) 0%,  var(--purple) 100%)`;
	});

	let num_indicators = 40;

	let delta = num_indicators / 360;

	let ticks = Array(num_indicators)
		.fill(0)
		.map((_, i) => {
			return -i * delta + Math.PI / 5;
		});
	let colors = Array(num_indicators)
		.fill(0)
		.map(() => {
			return "black";
		});
</script>

<div
	class="wrapper"
	role="button"
	bind:this={el}
	tabindex={-1}
	data-attribute={is_mouse_down}
	on:mousedown={() => {
		is_mouse_down = true;
	}}
	on:mouseup={() => {
		is_mouse_down = false;
	}}
>
	{#each ticks as tick, i}
		<div
			class="tick"
			style="transform:translate({-2 * Math.cos(tick)}em, {2 *
				Math.sin(tick)}em) rotate({(-tick * 180) / Math.PI +
				90}deg); background: {colors[i]}"
		/>
	{/each}
	<div bind:this={indicator_el} class="indicator" />
	<span>{label}</span>
</div>

<style>
	span {
		position: relative;
		top: 100%;
	}
	.wrapper {
		display: flex;
		width: 3em;
		height: 3em;
		background: radial-gradient(
			var(--gray100) 0,
			var(--gray150) 60%,
			var(--orange) 100%
		);
		border-radius: 50%;
		border: 2px solid var(--gray100);
		transition: filter 0.44s, border 0.44s;
		justify-content: center;
		position: relative;
		top: 12px;
	}

	.wrapper:hover {
		border: 2px solid var(--rotary-hover);
	}
	.wrapper[data-attribute="true"] {
		border: 2px solid var(--rotary-hover);
	}

	.indicator {
		transform: scale(0.1);
		border-radius: 50%;
		transform-origin: center;
	}

	.tick {
		width: 2px;
		height: 6px;
		position: absolute;
		align-self: center;
		transition: background 0.2s;
	}
</style>
