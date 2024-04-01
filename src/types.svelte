<script context="module" lang="ts">
	import { FREQ_PLOT_WIDTH } from "./constants.svelte";

	export type FilterBank = Record<string, Array<number>>;

	export type BPF = {
		gain: number,
		freq: number,
		Q: number,
	}

	export interface Recording {
		created: string;
		uid: number;
	}
	export interface Folder {
		folder_name: string;
		files: Array<any>;
		showing: boolean;
	}

	export function loglin(x: number, minfreq: number, maxfreq: number) {
		let y =
			((maxfreq - minfreq) * Math.log10(x / minfreq)) /
				Math.log10(maxfreq / minfreq) +
			minfreq;
		if (isNaN(y)) {
			console.error("result is nan for", x);
		}
		return y;
	}

	export function linlog(x: number, x1: number, x2: number) {
		return x1 * Math.pow(10, ((x - x1) / (x2 - x1)) * Math.log10(x2 / x1));
	}

	export const linspace = (start: number, stop: number, step: number) =>
		Array.from(
			{ length: (stop - start) / step + 1 },
			(_, i) => start + i * step
		);

	// this is probably not good
	export function setRectangle(
		gl: any,
		x: any,
		y: any,
		width: any,
		height: any
	) {
		var x1 = x;
		var x2 = x + width;
		var y1 = y;
		var y2 = y - height;
		gl.bufferData(
			gl.ARRAY_BUFFER,
			new Float32Array([x1, y1, x2, y1, x1, y2, x2, y2]),
			gl.STATIC_DRAW
		);
	}

	export function frequencyToXAxis(frequency: number) {
		const minF = Math.log(20) / Math.log(10);
		const maxF = Math.log(20000) / Math.log(10);

		let range = maxF - minF;
		let xAxis =
			((Math.log(frequency) / Math.log(10) - minF) / range) * FREQ_PLOT_WIDTH;
		return xAxis;
	}
</script>
