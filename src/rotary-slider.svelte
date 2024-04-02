<script lang="ts">
	import { onMount } from "svelte";

	let el: HTMLElement;
	let indicator_el: HTMLElement;
	let is_mouse_down = false;

	export let value: any;
	let radius = 12;

	let angle = 225;
	$: angle, (value = (1 - (angle + 45) / 270) * 10 + 0.01);
	// $: value, console.log(value);

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

				const x = radius * Math.cos((angle * Math.PI) / 180);
				const y = -radius * Math.sin((angle * Math.PI) / 180);

				indicator_el.style.transform = `scale(0.1) translate(${x}em, ${y}em)`;
				indicator_el.style.background = `linear-gradient(${
					90 - angle
				}deg, var(--gray50) 0%,  var(--purple) 100%)`;
				let a = (1-(angle + 45) / 270) * 40;
				console.log(a);
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

			function reset() {
				// have to call this here...maybe want to change how this is handled later
				is_mouse_down = false;
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
			var(--gray100) 100%
		);
		border-radius: 50%;
		border: 0px solid var(--purple);
		transition: border-color 0.44s;
		justify-content: center;
		position: relative;
		top: 12px;
	}

	.wrapper:hover {
		border-color: var(--lightpurple);
	}
	.wrapper[data-attribute="true"] {
		border-color: var(--lightpurple);
	}

	.indicator {
		width: 3em;
		height: 3em;
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
