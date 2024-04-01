<script lang="ts" context="module">
	import { FREQ_PLOT_WIDTH, SAMPLING_RATE } from "./constants.svelte";
	import type { Complex, FilterCoeffs2 } from "./types.svelte";

	export function cabs(z: Complex) {
		return Math.sqrt(z.re * z.re + z.im * z.im);
	}

	export function cdiv(z1: Complex, z2: Complex) {
		let a = z1.re;
		let b = z1.im;
		let c = z2.re;
		let d = z2.im;
		return {
			re: (a * c + b * d) / (c ** 2 + d ** 2),
			im: (b * c - a * d) / (c ** 2 + d ** 2),
		} as Complex;
	}

	export function biquad(gain: number, freq: number, Q: number) {
		let A = Math.pow(10, gain / 40);
		let w0 = (2 * Math.PI * freq) / SAMPLING_RATE;
		let alpha = Math.sin(w0) / 2 / Q;

		return {
			b0: 1 + alpha * A,
			b1: -2 * Math.cos(w0),
			b2: 1 - alpha * A,
			a0: 1 + alpha / A,
			a1: -2 * Math.cos(w0),
			a2: 1 - alpha / A,
		} as FilterCoeffs2;
	}

	export function freq_response(coeffs: FilterCoeffs2, len: number) {
		let H: Array<number> = [];
		for (let i = 0; i < len; i++) {
			let w = (Math.PI * i) / len;
			let x = Math.cos(w);
			let y = Math.sin(w);
			let c1: Complex = { re: x, im: y };
			let c2: Complex = { re: x * x - y * y, im: 2 * x * y };

			let numre = coeffs.b0 + coeffs.b1 * c1.re + coeffs.b2 * c2.re
			let numim = coeffs.b1 * c1.im + coeffs.b2 * c2.im

			let denre = coeffs.a0 + coeffs.a1 * c1.re + coeffs.a2 * c2.re
			let denim = coeffs.a1 * c1.im + coeffs.a2 * c2.im

			let num: Complex = { re: numre, im: numim };
			let den: Complex = { re: denre, im: denim };

			H.push(20 * Math.log10(cabs(cdiv(num, den)) + 1e-6));
		}

		return H;
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
