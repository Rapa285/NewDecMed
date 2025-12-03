<script lang="ts">
	import '../app.css';
	import '@fontsource-variable/inter';
	import '@fontsource-variable/montserrat';
	import { createAuthContext } from './(context)/auth-context.svelte';
	import { Toaster } from 'svelte-sonner';

	let { children } = $props();

	createAuthContext();
</script>

<Toaster position="top-right" richColors />
<main class="flex flex-col w-full flex-1 min-h-svh">
	{@render children()}
</main>
