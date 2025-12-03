<script lang="ts">
	import type { InvokeGetMedicalRecordsResponse, SuccessResponse } from '$lib/types.js';
	import { tryCatchAsVal } from '$lib/utils.js';
	import { ChevronRight, Loader2 } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';

	let fetchMedicalRecords = $state(getMedicalRecords());

	async function getMedicalRecords() {
		const resInvokeGetMedicalRecords = await tryCatchAsVal(async () => {
			return (await invoke('get_medical_records')) as SuccessResponse<
				InvokeGetMedicalRecordsResponse[]
			>;
		});

		if (!resInvokeGetMedicalRecords.success) {
			toast.error(resInvokeGetMedicalRecords.error);
			throw new Error(resInvokeGetMedicalRecords.error);
		}

		return resInvokeGetMedicalRecords.data.data;
	}
</script>

<h2 class="font-montserrat font-medium text-xl my-2">My Records</h2>
{#await fetchMedicalRecords}
	<div class="h-20 animate-pulse bg-zinc-100 w-full flex items-center justify-center">
		<Loader2 class="animate-spin" />
	</div>
{:then records}
	{#if records.length > 0}
		<div class="flex flex-col border border-zinc-200 rounded-md">
			{#each records as metadata, i (i)}
				<a
					class="flex items-center p-4 [&:not(:last-child)]:border-b border-zinc-200 justify-between"
					href={`/dashboard/emr/${metadata.index}`}
				>
					<span class="font-medium">Record {i + 1}</span>
					<div class="flex items-center gap-2">
						<p>
							{new Date(metadata.createdAt).toLocaleDateString('en-US', {
								year: 'numeric',
								month: 'short',
								day: '2-digit'
							})}
						</p>
						<ChevronRight size={16} />
					</div>
				</a>
			{/each}
		</div>
	{:else}
		<div class="bg-zinc-100 p-4 border border-zinc-200 rounded-md text-zinc-500">
			<p>No EMR found</p>
		</div>
	{/if}
{:catch e}
	<p>Something went wrong.</p>
	{JSON.stringify(e)}
{/await}
