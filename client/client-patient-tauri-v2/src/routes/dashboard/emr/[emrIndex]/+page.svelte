<script lang="ts">
	import type {
		InvokeGetMedicalRecordResponse,
		SuccessResponse,
		TauriMedicalData
	} from '$lib/types.js';
	import { tryCatchAsVal } from '$lib/utils.js';
	import { Loader2, LucideArrowLeft } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';

	let { data } = $props();

	let fetchMedicalRecord = $state(getMedicalRecord());

	async function getMedicalRecord() {
		const resInvokeGetMedicalRecord = await tryCatchAsVal(async () => {
			return (await invoke('get_medical_record', {
				index: data.emrIndex
			})) as SuccessResponse<InvokeGetMedicalRecordResponse>;
		});

		if (!resInvokeGetMedicalRecord.success) {
			toast.error(resInvokeGetMedicalRecord.error);
			throw new Error(resInvokeGetMedicalRecord.error);
		}

		return resInvokeGetMedicalRecord.data.data;
	}
</script>

<div class="mb-4 mt-2">
	<a href="/dashboard" class="flex max-w-max items-center gap-1"
		><LucideArrowLeft size={18} />Back</a
	>
</div>

{#await fetchMedicalRecord}
	<div class="h-20 animate-pulse bg-zinc-100 w-full flex items-center justify-center">
		<Loader2 class="animate-spin" />
	</div>
{:then record}
	<div class="flex flex-col gap-2">
		<div class="bg-zinc-50 rounded-md border border-zinc-200 p-2 flex flex-col gap-3">
			<div class="flex flex-col">
				<p class="text-xs font-medium text-zinc-600">Index</p>
				<p>{data.emrIndex}</p>
			</div>
			<div class="flex flex-col">
				<p class="text-xs font-medium text-zinc-600">Created At</p>
				<p>
					{new Date(record.createdAt).toLocaleDateString('id-ID', {
						year: 'numeric',
						month: 'short',
						day: 'numeric',
						hour: 'numeric',
						minute: 'numeric'
					})}
				</p>
			</div>
			<div class="flex flex-col">
				<p class="text-xs font-medium text-zinc-600">Anamnesis</p>
				<p>{record.medicalData.anamnesis}</p>
			</div>
			<div class="flex flex-col">
				<p class="text-xs font-medium text-zinc-600">Physical Check</p>
				<p>{record.medicalData.physical_check}</p>
			</div>
			<div class="flex flex-col">
				<p class="text-xs font-medium text-zinc-600">Psychological Check</p>
				<p>{record.medicalData.psychological_check}</p>
			</div>
			<div class="flex flex-col">
				<p class="text-xs font-medium text-zinc-600">Diagnose</p>
				<p>{record.medicalData.diagnose}</p>
			</div>
			<div class="flex flex-col">
				<p class="text-xs font-medium text-zinc-600">Therapy</p>
				<p>{record.medicalData.therapy}</p>
			</div>
		</div>
	</div>
{:catch e}
	<p>Something went wrong.</p>
	{JSON.stringify(e)}
{/await}
