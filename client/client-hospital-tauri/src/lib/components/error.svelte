<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		error: string;
	};

	let { error }: Props = $props();
</script>

<div class="p-2 rounded-md border border-red-500 bg-red-100 overflow-x-auto">
	<pre class="whitespace-pre">{error}</pre>
</div>
