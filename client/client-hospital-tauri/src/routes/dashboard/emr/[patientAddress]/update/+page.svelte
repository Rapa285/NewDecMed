<script lang="ts">
	import Error from '$lib/components/error.svelte';
	import { EmrUpdateState } from './state.svelte.js';

	let { data } = $props();

	const emrUpdateState = new EmrUpdateState({
		accessToken: data.accessToken,
		index: data.medicalMetadataIndex,
		patientIotaAddress: data.patientIotaAddress,
		patientPrePublicKey: data.patientPrePublicKey,
		updateMedicalRecordForm: data.updateMedicalRecordForm
	});

	const {
		form: updateMedicalRecordForm,
		enhance: updateMedicalRecordFormEnhance,
		errors: updateMedicalRecordFormErrors
	} = emrUpdateState.updateMedicalRecordFormMeta;
</script>

<h2 class="text-lg font-montserrat font-semibold">Update EMR Entry</h2>

{#await emrUpdateState.fetchMedicalRecord}
	Loading..
{:then record}
	<div class="grid grid-cols-[150px_1fr] items-center my-4">
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
	<form
		method="post"
		class="flex flex-col p-4 bg-white border border-zinc-200 rounded-md my-4 gap-2"
		use:updateMedicalRecordFormEnhance
	>
		<div class="flex flex-col gap-1">
			<label for="anamnesis" class="font-medium text-sm after:content-['*'] after:text-red-500 py-2"
				>Anamnesis</label
			>
			<textarea
				id="anamnesis"
				name="anamnesis"
				bind:value={$updateMedicalRecordForm.anamnesis}
				class="border border-zinc-300 p-2 focus:outline-none focus:ring-3 ring-zinc-500 rounded-md"
			></textarea>
			{#if $updateMedicalRecordFormErrors.anamnesis}
				<span class="px-2 py-1 text-xs font-medium text-red-500 bg-red-50"
					>{$updateMedicalRecordFormErrors.anamnesis[0]}</span
				>
			{/if}
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
				bind:value={$updateMedicalRecordForm.physicalCheck}
				class="border border-zinc-300 p-2 focus:outline-none focus:ring-3 ring-zinc-500 rounded-md"
			></textarea>
			{#if $updateMedicalRecordFormErrors.physicalCheck}
				<span class="px-2 py-1 text-xs font-medium text-red-500 bg-red-50"
					>{$updateMedicalRecordFormErrors.physicalCheck[0]}</span
				>
			{/if}
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
				bind:value={$updateMedicalRecordForm.psychologicalCheck}
				class="border border-zinc-300 p-2 focus:outline-none focus:ring-3 ring-zinc-500 rounded-md"
			></textarea>
			{#if $updateMedicalRecordFormErrors.psychologicalCheck}
				<span class="px-2 py-1 text-xs font-medium text-red-500 bg-red-50"
					>{$updateMedicalRecordFormErrors.psychologicalCheck[0]}</span
				>
			{/if}
		</div>
		<div class="flex flex-col gap-1">
			<label for="diagnose" class="font-medium text-sm after:content-['*'] after:text-red-500 py-2"
				>Diagnose</label
			>
			<textarea
				id="diagnose"
				name="diagnose"
				bind:value={$updateMedicalRecordForm.diagnose}
				class="border border-zinc-300 p-2 focus:outline-none focus:ring-3 ring-zinc-500 rounded-md"
			></textarea>
			{#if $updateMedicalRecordFormErrors.diagnose}
				<span class="px-2 py-1 text-xs font-medium text-red-500 bg-red-50"
					>{$updateMedicalRecordFormErrors.diagnose[0]}</span
				>
			{/if}
		</div>
		<div class="flex flex-col gap-1">
			<label for="therapy" class="font-medium text-sm after:content-['*'] after:text-red-500 py-2"
				>Therapy</label
			>
			<textarea
				id="therapy"
				name="therapy"
				bind:value={$updateMedicalRecordForm.therapy}
				class="border border-zinc-300 p-2 focus:outline-none focus:ring-3 ring-zinc-500 rounded-md"
			></textarea>
			{#if $updateMedicalRecordFormErrors.therapy}
				<span class="px-2 py-1 text-xs font-medium text-red-500 bg-red-50"
					>{$updateMedicalRecordFormErrors.therapy[0]}</span
				>
			{/if}
		</div>

		<button
			class="bg-zinc-800 px-4 py-2 rounded-md text-zinc-50 max-w-max mt-2 cursor-pointer"
			type="submit">Update</button
		>
	</form>
{:catch e}
	{#if e === '404'}
		{#await emrUpdateState.fetchPatientAdministrativeData}
			Loading...
		{:then record}
			<div class="grid grid-cols-[150px_1fr] items-center my-4">
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
		{/await}
		<div class="bg-white p-4 border border-zinc-200 rounded-md my-4">
			<p>No medical record found</p>
		</div>
	{:else}
		<Error error={e} />
	{/if}
	<a
		href={`/dashboard/emr/${data.patientIotaAddress}/create?accessToken=${data.accessToken}&patientPrePublicKey=${data.patientPrePublicKey}`}
		class="bg-zinc-800 text-zinc-50 border border-zinc-200 px-4 py-2 rounded-md max-w-max"
		>Create medical record</a
	>
{/await}
