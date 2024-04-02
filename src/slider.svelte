<script lang="ts">
	import { onMount } from "svelte";

	export let value;
	let position = 0;
	let el: HTMLElement;

	let height = 52;
	let is_dragging = false;
	let is_hovering = false;

	function draggable() {
		if (el === null) {
			return;
		}
		el.addEventListener("mousedown", function (e: MouseEvent) {
			// var offsetX = e.clientX - 10;
			var offsetY = e.clientY - position;
			function mouseMoveHandler(e: any) {
				if (el === null || !is_hovering) {
					return;
				}

				position = e.clientY - offsetY;
				position = Math.max(Math.min(position, height), 1);
				el.style.top = position + "px";
				// el.style.left = e.clientX - offsetX + "px";
				value = (0.5-(position / height))*44;
				// console.log(value)

			}

			function reset() {
				// have to call this here...maybe want to change how this is handled later
				is_dragging = false;
				window.removeEventListener("mousemove", mouseMoveHandler);
				window.removeEventListener("mouseup", reset);
			}
			window.addEventListener("mousemove", mouseMoveHandler);
			window.addEventListener("mouseup", reset);
		});
	}
	onMount(() => {
		draggable();
	});


</script>

<div
	class="wrapper"
	data-attribute={is_dragging}
	role="button"
	tabindex={-1}
		on:mouseenter={() => {
			is_hovering = true;
		}}
		on:mouseleave={() => {
			is_hovering = false;
		}}
>
	<div
		class="thumb"
		bind:this={el}
		role="button"
		tabindex={-1}
		on:mousedown={() => {
			is_dragging = true;
		}}
		on:mouseup={() => {
			is_dragging = false;
		}}
	/>
</div>

<style>
	.wrapper {
		background: var(--gray100);
		width: 2.5em;
		display: flex;
		justify-content: center;
		border: 1px solid var(--purple);
		position: relative;
		transition: border 0.33s;
	}

	.wrapper:hover {
		border: 1px solid var(--lightpurple);
	}

	.wrapper[data-attribute="true"] {
		border: 1px solid var(--lightpurple);
	}

	.thumb {
		background: black;
		width: 90%;
		height: 0.5em;
		position: absolute;
		top: 1px;
	}

	.thumb:active {
		background: var(--purple);
	}
</style>
