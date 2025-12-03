<script lang="ts">
	import { copyToClipboard } from '$lib/utils.js';
	import { Copy, Loader2 } from '@lucide/svelte';
	import { ProfileState } from './profile-state.svelte.js';

	const profileState = new ProfileState();
</script>

<h2 class="font-medium text-xl font-montserrat mb-2">Profile</h2>
{#await profileState.getProfile()}
	<div
		class="w-full h-25 animate-pulse bg-zinc-100 rounded-md border border-zinc-200 items-center justify-center flex"
	>
		<Loader2 class="animate-spin" />
	</div>
{:then profile}
	<div class="p-3 rounded-md bg-zinc-100 border border-zinc-200">
		<div class="grid grid-cols-[120px_1fr_20px] gap-2 max-w-full items-center">
			<p>CID:</p>
			<p class="bg-white border border-zinc-200 px-2 py-1 truncate rounded-md text-sm">
				{profile.id}
			</p>
			<button
				class="flex items-center justify-center cursor-pointer"
				onclick={() => copyToClipboard(profile.id || '')}><Copy size={16} /></button
			>
			<!-- <p class="break-all line-clamp-1">Id Hash:</p>
		<p class="bg-white border border-zinc-200 px-2 py-1 truncate rounded-md text-sm">
			{profile.idHash}
		</p> -->
			<p class="break-all">Name:</p>
			<p class="bg-white border border-zinc-200 px-2 py-1 truncate rounded-md text-sm">
				{profile.name}
			</p>
			<button
				class="flex items-center justify-center cursor-pointer"
				onclick={() => copyToClipboard(profile.name || '')}><Copy size={16} /></button
			>
			<p class="break-all">Hospital:</p>
			<p class="bg-white border border-zinc-200 px-2 py-1 truncate rounded-md text-sm">
				{profile.hospital}
			</p>
			<button
				class="flex items-center justify-center cursor-pointer"
				onclick={() => copyToClipboard(profile.hospital || '')}><Copy size={16} /></button
			>
		</div>
	</div>

	{#if profile.role != 'Admin'}
		<div class="w-full flex items-center justify-center flex-col">
			<div class="max-w-80 w-full my-2 border border-zinc-200">
				{@html profileState.qr}
			</div>
			<button class="button-dark max-w-sm" onclick={profileState.downloadQr}
				>Download QR Code</button
			>
		</div>
	{/if}
{/await}

<div class="p-2 rounded-md border border-zinc-200 mt-4">
	<h2 class="font-medium text-lg mb-2">Actions</h2>
	<button
		class=" border bg-red-50 px-3 py-1 rounded-md text-red-500 max-w-max"
		onclick={profileState.signout}>Signout</button
	>
</div>
