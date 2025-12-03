<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import type { SuccessResponse, TauriAdministrativeData } from '$lib/types.js';
	import { copyToClipboard, tryCatchAsVal } from '$lib/utils.js';
	import { Copy, Loader2 } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';

	let fetchProfile = $state(getProfile());

	async function signout() {
		await invoke('signout');
		invalidateAll();
	}

	async function getProfile() {
		const resInvokeGetProfile = await tryCatchAsVal(async () => {
			return (await invoke('get_profile')) as SuccessResponse<TauriAdministrativeData>;
		});

		if (!resInvokeGetProfile.success) {
			toast.error(resInvokeGetProfile.error);
			throw new Error(resInvokeGetProfile.error);
		}

		return resInvokeGetProfile.data.data;
	}
</script>

<h2 class="font-montserrat font-medium text-xl my-2">Profile</h2>
{#await fetchProfile}
	<div class="h-20 animate-pulse bg-zinc-100 w-full flex items-center justify-center">
		<Loader2 class="animate-spin" />
	</div>
{:then profile}
	<div class="p-3 rounded-md bg-zinc-100 border border-zinc-200 my-4">
		<div class="flex flex-col gap-2">
			<div class="flex flex-col gap-1">
				<p class="text-sm text-zinc-500">NIK:</p>
				<div class="flex items-center gap-2">
					<p class="bg-white px-2 py-1 rounded-md border border-zinc-200 truncate text-sm flex-1">
						{profile.id}
					</p>
					<button
						class="flex items-center justify-center cursor-pointer"
						onclick={() => copyToClipboard(profile.id || 'N/A')}><Copy size={16} /></button
					>
				</div>
			</div>
			<div class="flex flex-col gap-1">
				<p class="text-sm text-zinc-500">Name:</p>
				<div class="flex items-center gap-2">
					<p class="bg-white px-2 py-1 rounded-md border border-zinc-200 truncate text-sm flex-1">
						{profile.name}
					</p>
					<button
						class="flex items-center justify-center cursor-pointer"
						onclick={() => copyToClipboard(profile.name || 'N/A')}><Copy size={16} /></button
					>
				</div>
			</div>
			<div class="flex flex-col gap-1">
				<p class="text-sm text-zinc-500">Birth Place:</p>
				<div class="flex items-center gap-2">
					<p class="bg-white px-2 py-1 rounded-md border border-zinc-200 truncate text-sm flex-1">
						{profile.birthPlace}
					</p>
					<button
						class="flex items-center justify-center cursor-pointer"
						onclick={() => copyToClipboard(profile.birthPlace || 'N/A')}><Copy size={16} /></button
					>
				</div>
			</div>
			<div class="flex flex-col gap-1">
				<p class="text-sm text-zinc-500">Date of Birth:</p>
				<div class="flex items-center gap-2">
					<p class="bg-white px-2 py-1 rounded-md border border-zinc-200 truncate text-sm flex-1">
						{profile.dateOfBirth}
					</p>
					<button
						class="flex items-center justify-center cursor-pointer"
						onclick={() => copyToClipboard(profile.dateOfBirth || 'N/A')}><Copy size={16} /></button
					>
				</div>
			</div>
			<div class="flex flex-col gap-1">
				<p class="text-sm text-zinc-500">Gender:</p>
				<div class="flex items-center gap-2">
					<p class="bg-white px-2 py-1 rounded-md border border-zinc-200 truncate text-sm flex-1">
						{profile.gender}
					</p>
					<button
						class="flex items-center justify-center cursor-pointer"
						onclick={() => copyToClipboard(profile.gender || 'N/A')}><Copy size={16} /></button
					>
				</div>
			</div>
			<div class="flex flex-col gap-1">
				<p class="text-sm text-zinc-500">Religion:</p>
				<div class="flex items-center gap-2">
					<p class="bg-white px-2 py-1 rounded-md border border-zinc-200 truncate text-sm flex-1">
						{profile.religion}
					</p>
					<button
						class="flex items-center justify-center cursor-pointer"
						onclick={() => copyToClipboard(profile.religion || 'N/A')}><Copy size={16} /></button
					>
				</div>
			</div>
			<div class="flex flex-col gap-1">
				<p class="text-sm text-zinc-500">Education:</p>
				<div class="flex items-center gap-2">
					<p class="bg-white px-2 py-1 rounded-md border border-zinc-200 truncate text-sm flex-1">
						{profile.education}
					</p>
					<button
						class="flex items-center justify-center cursor-pointer"
						onclick={() => copyToClipboard(profile.education || 'N/A')}><Copy size={16} /></button
					>
				</div>
			</div>
			<div class="flex flex-col gap-1">
				<p class="text-sm text-zinc-500">Occupation:</p>
				<div class="flex items-center gap-2">
					<p class="bg-white px-2 py-1 rounded-md border border-zinc-200 truncate text-sm flex-1">
						{profile.occupation}
					</p>
					<button
						class="flex items-center justify-center cursor-pointer"
						onclick={() => copyToClipboard(profile.occupation || 'N/A')}><Copy size={16} /></button
					>
				</div>
			</div>
			<div class="flex flex-col gap-1">
				<p class="text-sm text-zinc-500">Marital Status:</p>
				<div class="flex items-center gap-2">
					<p class="bg-white px-2 py-1 rounded-md border border-zinc-200 truncate text-sm flex-1">
						{profile.maritalStatus}
					</p>
					<button
						class="flex items-center justify-center cursor-pointer"
						onclick={() => copyToClipboard(profile.maritalStatus || 'N/A')}
						><Copy size={16} /></button
					>
				</div>
			</div>
			<div class="flex flex-col gap-1">
				<p class="text-sm text-zinc-500">IOTA Address:</p>
				<div class="flex items-center gap-2">
					<p class="bg-white px-2 py-1 rounded-md border border-zinc-200 truncate text-sm flex-1">
						{profile.iotaAddress}
					</p>
					<button
						class="flex items-center justify-center cursor-pointer"
						onclick={() => copyToClipboard(profile.iotaAddress || 'N/A')}><Copy size={16} /></button
					>
				</div>
			</div>
		</div>
	</div>
{:catch e}
	<p>Something went wrong.</p>
	{JSON.stringify(e)}
{/await}

<button onclick={signout} class="border border-red-500 bg-red-50 py-2 rounded-md text-red-500"
	>Sign Out</button
>
