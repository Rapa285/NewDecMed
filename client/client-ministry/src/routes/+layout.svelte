<script lang="ts">
	import { Toaster } from 'svelte-sonner';
	import '../app.css';

	let { children } = $props();
</script>

<Toaster />
<div class="flex min-h-screen w-full flex-col bg-zinc-100 p-3">
	<div class="flex w-full flex-1 flex-col rounded-md border border-zinc-200 bg-white p-2">
		{@render children()}
	</div>
</div>
