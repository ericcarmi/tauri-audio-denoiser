<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
	import { onMount } from "svelte";

	export let value: any;
	export let index: number;

	let el: HTMLElement;
	let indicator_el: HTMLElement;
	let is_mouse_down = false;

	let radius = 12;

	let angle = 225;
	$: value, redraw();

	function redraw() {
		if (!is_mouse_down && indicator_el !== undefined && el !== undefined) {
			angle = 1 + ((10.01 - value) / 10) * 270 - 45;
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
						return "var(--lightpurple)";
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
			// var offsetX = e.clientX - 10;
			// var offsetY = e.clientY - position;

			// offsetY = Math.max(offsetY, 0);

			function mouseMoveHandler(e: any) {
				if (el === null || indicator_el === null) {
					return;
				}
				if (!is_mouse_down) return;

				angle += (-e.movementX + e.movementY) * 3;
				angle = Math.max(Math.min(angle, 225), -45);

				value = (1 - (angle + 45) / 270) * 10 + 0.01;

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
							return "var(--lightpurple)";
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
        invoke("save_bpf_Q", { q: value, index: index });

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
</div>

<style>
	.wrapper {
		display: flex;
		width: 3em;
		height: 3em;
		background: radial-gradient(
			var(--gray100) 0,
			var(--gray150) 80%,
			var(--purple) 100%
		);
		border-radius: 50%;
		border: 2px solid var(--gray100);
		transition: filter 0.44s, border 0.44s;
		justify-content: center;
		position: relative;
		top: 12px;
	}

	.wrapper:hover {
		border: 2px solid var(--lightpurple);
	}
	.wrapper[data-attribute="true"] {
		border: 2px solid var(--lightpurple);
	}

	.indicator {
		transform: scale(0.1);
		border-radius: 50%;
		transform-origin: center;
	}

	.tick {
		background: red;
		width: 2px;
		height: 6px;
		position: absolute;
		align-self: center;
		transition: background 0.2s;
	}
</style>
