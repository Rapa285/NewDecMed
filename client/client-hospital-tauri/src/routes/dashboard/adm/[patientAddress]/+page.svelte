<script lang="ts">
	import Error from '$lib/components/error.svelte';
	import { AdmReadState } from './state.svelte.js';

	let { data } = $props();

	let admReadState = new AdmReadState({
		accessToken: data.accessToken,
		patientIotaAddress: data.patientIotaAddress
	});
</script>

{#if data.accessToken}
	{#await admReadState.fetchPatientAdministrativeData}
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
	{:catch e}
		<Error error={e} />
	{/await}
{:else}
	nothing
{/if}
