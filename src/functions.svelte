<script lang="ts" context="module">
	import { invoke } from "@tauri-apps/api/tauri";
	import { FREQ_PLOT_WIDTH } from "./constants.svelte";
	import type {
		BPF,
		UIParams,
		AudioParams,
		Complex,
		IIR2,
		StereoParams,
		UIFilters,
	} from "./types";

	export function cabs(z: Complex) {
		return Math.sqrt(z.re * z.re + z.im * z.im);
	}

	export function init_channel_params(
		gains: number[],
		freqs: number[],
		Qs: number[],
	): StereoParams {
		if (gains.length !== freqs.length || gains.length !== Qs.length) {
			console.error("array length mismatch");
			// return;
		}
		let output_gain = 0.0;
		let noise_gain = 0.0;
		let pre_smooth_gain = 0.5;
		let post_smooth_gain = 0.5;
		let clean = false;
		let bpf_filters: Array<BPF> = Array(gains.length)
			.fill(0)
			.map((_, i) => {
				return { gain: gains[i], freq: freqs[i], Q: Qs[i] };
			});

		let ui = {
			clean: clean,
			left_mute: false,
			right_mute: false,
			output_gain: output_gain,
			noise_gain: noise_gain,
			pre_smooth_gain: pre_smooth_gain,
			post_smooth_gain: post_smooth_gain,
			stereo_choice: "Both",
			filters: { bank: bpf_filters },
		} as UIParams;

		let c = {
			ui_params: ui,
			time: 0,
			dft_size: 256,
		} as AudioParams;

		return {
			left: c,
			right: c,
			stereo_choice: "Both",
			is_stereo: true,
			clean: false,
			num_file_samples: 0,
			file_path: "",
			time: 0,
		};
	}

	export function init_ui_params(
		gains: number[],
		freqs: number[],
		Qs: number[],
	): UIParams {
		if (gains.length !== freqs.length || gains.length !== Qs.length) {
			console.error("array length mismatch");
			// return;
		}
		let output_gain = 0.0;
		let noise_gain = 0.0;
		let pre_smooth_gain = 0.5;
		let post_smooth_gain = 0.5;
		let clean = false;
		let bpf_filters: Array<BPF> = Array(gains.length)
			.fill(0)
			.map((_, i) => {
				return { gain: gains[i], freq: freqs[i], Q: Qs[i] };
			});

		return {
			clean: clean,
			left_mute: false,
			right_mute: false,
			output_gain: output_gain,
			noise_gain: noise_gain,
			pre_smooth_gain: pre_smooth_gain,
			post_smooth_gain: post_smooth_gain,
			stereo_choice: "Both",
			filters: { bank: bpf_filters },
		} as UIParams;
	}

	export function update_css_color(color: string, color_name: string) {
		if (color !== undefined) {
			if (color_name === "app-background") {
				document.body.style.setProperty("background", color);
			} else if (color_name === "app-text") {
				console.log("change text");

				document.body.style.setProperty("color", color);
			}
			document.body.style.setProperty(
				`--${color_name.replace("_", "-")}`,
				color,
			);
		}
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

	export function biquad(gain: number, freq: number, Q: number, rate: number) {
		let A = Math.pow(10, gain / 40);
		let w0 = (2 * Math.PI * freq) / rate;
		let alpha = Math.sin(w0) / 2 / Q;

		return {
			b0: 1 + alpha * A,
			b1: -2 * Math.cos(w0),
			b2: 1 - alpha * A,
			a0: 1 + alpha / A,
			a1: -2 * Math.cos(w0),
			a2: 1 - alpha / A,
		} as IIR2;
	}

	export function freq_response(coeffs: IIR2, len: number) {
		let H: Array<number> = [];
		for (let i = 0; i < len; i++) {
			let w = (Math.PI * i) / len;
			let x = Math.cos(w);
			let y = Math.sin(w);
			let c1: Complex = { re: x, im: y };
			let c2: Complex = { re: x * x - y * y, im: 2 * x * y };

			let numre = coeffs.b0 + coeffs.b1 * c1.re + coeffs.b2 * c2.re;
			let numim = coeffs.b1 * c1.im + coeffs.b2 * c2.im;

			let denre = coeffs.a0 + coeffs.a1 * c1.re + coeffs.a2 * c2.re;
			let denim = coeffs.a1 * c1.im + coeffs.a2 * c2.im;

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

	export function loglin2(x: number, minfreq: number, maxfreq: number) {
		let y =
			((maxfreq - minfreq) * Math.log2(x / minfreq)) /
				Math.log2(maxfreq / minfreq) +
			minfreq;
		if (isNaN(y)) {
			console.error("result is nan for", x);
		}
		return y;
	}

	export function linlog(x: number, minfreq: number, maxfreq: number) {
		return (
			minfreq *
			Math.pow(
				10,
				((x - minfreq) / (maxfreq - minfreq)) * Math.log10(maxfreq / minfreq),
			)
		);
	}

	export function linlog2(x: number, minfreq: number, maxfreq: number) {
		return (
			minfreq *
			Math.pow(
				2,
				((x - minfreq) / (maxfreq - minfreq)) * Math.log2(maxfreq / minfreq),
			)
		);
	}

	export function mel(x: number) {
		return 2595 * Math.log10(1 + x / 700);
	}
	export function bark_scale(x: number) {
		return 13 * Math.atan(0.00076 * x) + 3.5 * Math.atan((x / 7500) ** 2);
	}

	export const linspace = (start: number, stop: number, step: number) =>
		Array.from(
			{ length: (stop - start) / step + 1 },
			(_, i) => start + i * step,
		);

	export function frequencyToXAxis(frequency: number) {
		const minF = Math.log(20) / Math.log(10);
		const maxF = Math.log(20000) / Math.log(10);

		let range = maxF - minF;
		let xAxis =
			((Math.log(frequency) / Math.log(10) - minF) / range) * FREQ_PLOT_WIDTH;
		return xAxis;
	}

	export function remove_slashes_ext(s: string) {
		if (s.includes("/")) {
			let x = s.split("/");
			return x[x.length - 1].split(".")[0];
		} else {
			return s.split(".")[0];
		}
	}

	export function hexToRgb(hex: string) {
		var result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
		return result
			? {
					r: parseInt(result[1], 16),
					g: parseInt(result[2], 16),
					b: parseInt(result[3], 16),
					a: null,
				}
			: null;
	}

	export function resetGains(ui_params: UIParams) {
		let bpfs = [
			...ui_params.filters.bank.map((filt, i) => {
				invoke("message_filters", {
					stereoChoice: ui_params.stereo_choice,
					index: i + 1,
					gain: 0.0,
					freq: filt.freq,
					q: filt.Q,
				});
				return { gain: 0.0, freq: filt.freq, Q: filt.Q };
			}),
		];
		return { bank: bpfs } as UIFilters;
	}
</script>
