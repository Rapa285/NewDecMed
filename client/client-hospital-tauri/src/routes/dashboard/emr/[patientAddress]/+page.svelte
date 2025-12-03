<script lang="ts">
	import { EmrReadState } from './state.svelte.js';

	let { data } = $props();

	let emrReadState = new EmrReadState({
		accessToken: data.accessToken,
		index: data.index,
		patientIotaAddress: data.patientIotaAddress
	});
</script>

<h2 class="text-lg font-montserrat font-semibold">EMR of</h2>

{#if data.accessToken}
	{#await emrReadState.fetchMedicalRecord}
		Loading...
	{:then record}
		<h3 class="font-medium text-md my-4">Administrative Data</h3>
		<div class="grid grid-cols-[150px_1fr] items-center mb-4">
			<div class="p-2 bg-white border border-b-0 border-zinc-200">
				<span>NIK</span>
			</div>
			<div class="p-2 border border-zinc-200 border-b-0 border-l-0">
				<span>{record.administrativeData.id}</span>
			</div>
			<div class="p-2 bg-white border border-b-0 border-zinc-200">
				<span>Name</span>
			</div>
			<div class="p-2 border border-zinc-200 border-b-0 border-l-0">
				<span>{record.administrativeData.name}</span>
			</div>
			<div class="p-2 bg-white border border-b-0 border-zinc-200">
				<span>Birth Place</span>
			</div>
			<div class="p-2 border border-zinc-200 border-b-0 border-l-0">
				<span>{record.administrativeData.birth_place}</span>
			</div>
			<div class="p-2 bg-white border border-b-0 border-zinc-200">
				<span>Date of Birth</span>
			</div>
			<div class="p-2 border border-zinc-200 border-b-0 border-l-0">
				<span>{record.administrativeData.date_of_birth}</span>
			</div>
			<div class="p-2 bg-white border border-b-0 border-zinc-200">
				<span>Gender</span>
			</div>
			<div class="p-2 border border-zinc-200 border-b-0 border-l-0">
				<span>{record.administrativeData.gender}</span>
			</div>
			<div class="p-2 bg-white border border-b-0 border-zinc-200">
				<span>Religion</span>
			</div>
			<div class="p-2 border border-zinc-200 border-b-0 border-l-0">
				<span>{record.administrativeData.religion}</span>
			</div>
			<div class="p-2 bg-white border border-b-0 border-zinc-200">
				<span>Education</span>
			</div>
			<div class="p-2 border border-zinc-200 border-b-0 border-l-0">
				<span>{record.administrativeData.education}</span>
			</div>
			<div class="p-2 bg-white border border-b-0 border-zinc-200">
				<span>Occupation</span>
			</div>
			<div class="p-2 border border-zinc-200 border-b-0 border-l-0">
				<span>{record.administrativeData.occupation}</span>
			</div>
			<div class="p-2 bg-white border border-zinc-200">
				<span>Marital Status</span>
			</div>
			<div class="p-2 border border-zinc-200 border-l-0">
				<span>{record.administrativeData.marital_status}</span>
			</div>
		</div>
		<h3 class="font-medium text-md my-4">Medical Data</h3>
		<div class="grid grid-cols-[150px_1fr] items-center">
			<div class="p-2 bg-white border border-b-0 border-zinc-200">
				<span>Index</span>
			</div>
			<div class="p-2 border border-zinc-200 border-b-0 border-l-0">
				<span>{record.currentIndex}</span>
			</div>
			<div class="p-2 bg-white border border-zinc-200">
				<span>Created At</span>
			</div>
			<div class="p-2 border border-zinc-200 border-l-0">
				<span
					>{new Date(record.createdAt).toLocaleDateString('id-ID', {
						year: 'numeric',
						month: 'short',
						day: 'numeric',
						hour: '2-digit',
						minute: '2-digit',
						hourCycle: 'h24'
					})}</span
				>
			</div>
		</div>

		<div class="flex flex-col gap-1">
			<label for="anamnesis" class="font-medium text-sm after:content-['*'] after:text-red-500 py-2"
				>Anamnesis</label
			>
			<textarea
				id="anamnesis"
				name="anamnesis"
				disabled
				value={record.medicalData.anamnesis}
				class="border border-zinc-300 p-2 focus:outline-none focus:ring-3 ring-zinc-500 rounded-md"
			></textarea>
		</div>
		<div class="flex flex-col gap-1">
			<label
				for="physicalCheck"
				class="font-medium text-sm after:content-['*'] after:text-red-500 py-2"
				>Physical Check</label
			>
			<textarea
				id="physicalCheck"
				name="physicalCheck"
				disabled
				value={record.medicalData.physical_check}
				class="border border-zinc-300 p-2 focus:outline-none focus:ring-3 ring-zinc-500 rounded-md"
			></textarea>
		</div>
		<div class="flex flex-col gap-1">
			<label
				for="psychologicalCheck"
				class="font-medium text-sm after:content-['*'] after:text-red-500 py-2"
				>Psychological Check</label
			>
			<textarea
				id="psychologicalCheck"
				name="psychologicalCheck"
				disabled
				value={record.medicalData.psychological_check}
				class="border border-zinc-300 p-2 focus:outline-none focus:ring-3 ring-zinc-500 rounded-md"
			></textarea>
		</div>
		<div class="flex flex-col gap-1">
			<label for="diagnose" class="font-medium text-sm after:content-['*'] after:text-red-500 py-2"
				>Diagnose</label
			>
			<textarea
				id="diagnose"
				name="diagnose"
				disabled
				value={record.medicalData.diagnose}
				class="border border-zinc-300 p-2 focus:outline-none focus:ring-3 ring-zinc-500 rounded-md"
			></textarea>
		</div>
		<div class="flex flex-col gap-1">
			<label for="therapy" class="font-medium text-sm after:content-['*'] after:text-red-500 py-2"
				>Therapy</label
			>
			<textarea
				id="therapy"
				name="therapy"
				disabled
				value={record.medicalData.therapy}
				class="border border-zinc-300 p-2 focus:outline-none focus:ring-3 ring-zinc-500 rounded-md"
			></textarea>
		</div>

		<div class="flex items-center mt-4">
			{#if record.prevIndex !== null && record.prevIndex !== undefined}
				<div class="flex-1 justify-start flex items-center">
					<a
						href={`/dashboard/emr/${data.patientIotaAddress}?accessToken=${data.accessToken}&index=${record.prevIndex}`}
						class="max-w-max bg-zinc-800 text-zinc-100 px-4 rounded-md"
						onclick={() => {
							emrReadState.index = record.prevIndex as number;
						}}>Prev</a
					>
				</div>
			{/if}
			{#if record.nextIndex !== null && record.prevIndex !== undefined}
				<div class="flex-1 justify-end flex items-center">
					<a
						href={`/dashboard/emr/${data.patientIotaAddress}?accessToken=${data.accessToken}&index=${record.nextIndex}`}
						class="max-w-max bg-zinc-800 text-zinc-100 px-4 rounded-md"
						onclick={() => {
							emrReadState.index = record.nextIndex as number;
						}}>Next</a
					>
				</div>
			{/if}
		</div>
	{:catch e}
		<div class="bg-zinc-100 p-4 border border-zinc-200 rounded-md text-zinc-500">
			<p>No EMR found</p>
		</div>
		<!-- <Error error={e} /> -->
	{/await}
{:else}
	nothing
{/if}
