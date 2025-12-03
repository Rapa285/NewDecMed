<script lang="ts">
	import { page } from '$app/state';
	import { cn } from '$lib/utils';
	import { Home, LucideHistory, Scan, User } from '@lucide/svelte';

	let { children } = $props();

	const navs = [
		{
			icon: Home,
			url: '/dashboard'
		},
		{
			icon: LucideHistory,
			url: '/dashboard/log'
		},
		{
			icon: Scan,
			url: '/dashboard/scan'
		},
		{
			icon: User,
			url: '/dashboard/profile'
		}
	];
</script>

<div class="relative">
	<div class="flex flex-col flex-1 p-2">
		{@render children()}
	</div>
	<div class="fixed bottom-0 border-t border-zinc-200 w-full bg-zinc-50 p-2 flex items-center">
		{#each navs as nav, i (i)}
			<a href={nav.url} class="flex-1 flex items-center justify-center">
				<div class={cn('p-2 rounded-md', page.url.pathname === nav.url && 'bg-zinc-200')}>
					<nav.icon class="flex-1" />
				</div>
			</a>
		{/each}
	</div>
</div>
