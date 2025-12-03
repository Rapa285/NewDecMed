<script lang="ts">
	import { getAuthContext } from '../(context)/auth-context.svelte';
	import { page } from '$app/state';
	import { cn } from '$lib/utils';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();

	const authContext = getAuthContext();
	authContext.role = data.role;
</script>

<div class="flex flex-1 flex-col w-full mx-auto p-3">
	<div class="flex flex-1 bg-zinc-50 border border-zinc-200 rounded-lg">
		<div class="w-44 border-r border-zinc-200 flex flex-col shrink-0">
			<h2 class="font-montserrat font-semibold px-4 py-2 text-lg border-b border-zinc-200">
				Decmed
			</h2>
			{#each authContext.getNav() as navlink (navlink.link)}
				<a
					class={cn(
						'px-4 py-2 font-medium bg-white border-b border-zinc-200 w-full',
						page.url.pathname === navlink.link && 'bg-zinc-800 text-zinc-100'
					)}
					href={navlink.link}>{navlink.label}</a
				>
			{/each}
		</div>
		<div class="p-4 flex-1 flex flex-col min-w-0">
			{@render children()}
		</div>
	</div>
</div>
