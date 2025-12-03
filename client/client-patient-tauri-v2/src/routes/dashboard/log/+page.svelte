<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { waitMs } from '$lib/utils.js';
	import { Loader2 } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import moment from 'moment';

	let { data } = $props();

	let isRevoking = $state(false);
</script>

<div class="flex flex-col">
	<h2 class="font-montserrat font-medium text-xl my-2">Access History</h2>

	{#await data.accessLog}
		Loading...
	{:then accessLog}
		{#if accessLog.data.length > 0}
			<div class="flex flex-col gap-2">
				{#each accessLog.data as access}
					<div class="bg-zinc-100 border border-zinc-300 p-3 rounded-md flex flex-col gap-2">
						<p>
							{new Date(access.date).toLocaleDateString('id-ID', {
								year: 'numeric',
								month: 'short',
								day: 'numeric',
								hour: 'numeric',
								minute: 'numeric',
								hourCycle: 'h23'
							})}
						</p>
						<div class="flex flex-col">
							<p class="text-sm text-zinc-400">Hospital:</p>
							<p>{access.hospital_metadata.name}</p>
						</div>
						<div class="flex flex-col">
							<p class="text-sm text-zinc-400">Name:</p>
							<p>{access.hospital_personnel_metadata.name}</p>
						</div>
						<div class="flex flex-col">
							<p class="text-sm text-zinc-400">Access Type:</p>
							<p>{access.access_type}</p>
						</div>
						<div class="flex flex-col">
							<p class="text-sm text-zinc-400">Access Data Type:</p>
							<div class="flex items-center gap-2">
								{#each access.access_data_type as dtType}
									<p class="bg-white px-2">{dtType}</p>
								{/each}
							</div>
						</div>
						{#if !access.is_revoked && moment(access.date)
								.add(access.exp_dur, 'minutes')
								.isAfter(moment())}
							<button
								class="bg-zinc-800 text-zinc-200 p-2 cursor-pointer"
								disabled={isRevoking}
								onclick={async () => {
									try {
										isRevoking = true;
										await invoke('revoke_access', {
											hospitalPersonnelAddress: access.hospital_personnel_address,
											index: access.index
										});
										await waitMs(2000);
										invalidateAll();
									} catch (e) {
										console.log(e);
									}

									isRevoking = false;
								}}
							>
								{#if isRevoking}
									<Loader2 class="animate-spin" />
								{:else}
									Revoke Access
								{/if}
							</button>
						{/if}
					</div>
				{/each}
			</div>
		{:else}
			<div class="bg-zinc-100 p-4 border border-zinc-200 rounded-md text-zinc-500">
				<p>No access found</p>
			</div>
		{/if}
	{:catch e}
		<div>{e}</div>
	{/await}
</div>
