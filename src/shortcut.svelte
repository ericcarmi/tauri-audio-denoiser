<script lang="ts" context="module">
	export const shortcut = (node: any, params: any) => {
		let handler: (e: KeyboardEvent) => void;
		const removeHandler = () => window.removeEventListener("keypress", handler),
			setHandler = () => {
				removeHandler();
				if (!params) return;
				handler = (e: KeyboardEvent) => {
					if (
						!!params.alt != e.altKey ||
						!!params.shift != e.shiftKey ||
						!!params.control != (e.ctrlKey || e.metaKey) ||
						params.code != e.code
					)
						return;
					e.preventDefault();
					params.callback ? params.callback() : node.click();
				};
				window.addEventListener("keypress", handler);
			};
		setHandler();
		return {
			update: setHandler,
			destroy: removeHandler,
		};
	};

	export const shortcutRelease = (node: any, params: any) => {
		let handler: any;
		const removeHandler = () => window.removeEventListener("keyup", handler),
			setHandler = () => {
				removeHandler();
				if (!params) return;
				handler = (e: any) => {
					if (
						!!params.alt != e.altKey ||
						!!params.shift != e.shiftKey ||
						!!params.control != (e.ctrlKey || e.metaKey) ||
						params.code != e.code
					)
						return;
					e.preventDefault();
					params.callback ? params.callback() : node.click();
				};
				window.addEventListener("keyup", handler);
			};
		setHandler();
		return {
			update: setHandler,
			destroy: removeHandler,
		};
	};
</script>
