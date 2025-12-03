<script lang="ts">
	let { children } = $props();
</script>

<div class="flex flex-col p-4 items-center justify-center w-full flex-1">
	{@render children()}
	<!-- <div class="absolute bottom-0 text-zinc-400 pb-2 flex flex-col items-center gap-2">
		<p class="text-sm border border-zinc-200 px-1.5 py-0.5 bg-zinc-50">v1.0.0-alpha</p>
	</div> -->
</div>
