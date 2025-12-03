<script lang="ts">
	import Dialog from '$lib/components/dialog.svelte';
	import { Copy, Loader2, Plus, RefreshCcw } from '@lucide/svelte';
	import { HomeState } from './state.svelte';
	import { cn, copyToClipboard } from '$lib/utils';

	let { data } = $props();

	const homeState = new HomeState({
		addHospitalForm: data.addHospitalForm
	});

	const {
		constraints: addHospitalFormConstraints,
		delayed: addHospitalFormDelayed,
		enhance: addHospitalFormEnhance,
		errors: addHospitalFormErrors,
		form: addHospitalForm
	} = homeState.addHospitalFormMeta;
</script>

<div>
	<Dialog
		bind:open={homeState.isAddHospitalDialogOpen}
		contentProps={{
			escapeKeydownBehavior: 'ignore',
			interactOutsideBehavior: 'ignore'
		}}
	>
		{#snippet title()}
			<h2>Add Hospital</h2>
		{/snippet}

		<form method="post" class="flex w-full flex-col" use:addHospitalFormEnhance>
			<div class="my-4 flex flex-col gap-2">
				<div class="container-input-text">
					<label for="hospital-id" class="required-label">Hospital ID</label>
					<input
						id="hospital-id"
						type="text"
						class={cn('input-base', $addHospitalFormErrors.hospitalId && 'input-error')}
						placeholder="hos_dalgi"
						bind:value={$addHospitalForm.hospitalId}
						{...$addHospitalFormConstraints.hospitalId}
					/>
					{#if $addHospitalFormErrors.hospitalId}
						<span class="error-input-text">{$addHospitalFormErrors.hospitalId[0]}</span>
					{/if}
				</div>
				<div class="container-input-text">
					<label for="hospital-name" class="required-label">Hospital Name</label>
					<input
						id="hospital-name"
						type="text"
						class={cn('input-base', $addHospitalFormErrors.hospitalName && 'input-error')}
						placeholder="Dalgi Hospital"
						bind:value={$addHospitalForm.hospitalName}
						{...$addHospitalFormConstraints.hospitalName}
					/>
					{#if $addHospitalFormErrors.hospitalName}
						<span class="error-input-text">{$addHospitalFormErrors.hospitalName[0]}</span>
					{/if}
				</div>
			</div>
			<div class="flex w-full items-center justify-end gap-2">
				<button type="button" class="btn-cancel" onclick={homeState.closeAddHospitalDialog}
					>Cancel</button
				>
				<button type="submit" class="btn">
					{#if $addHospitalFormDelayed}
						<Loader2 class="animate-spin" />
					{:else}
						Create
					{/if}
				</button>
			</div>
		</form>
	</Dialog>
	<button class="btn" onclick={homeState.openAddHospitalDialog}><Plus size={16} /> Hospital</button>

	{#await data.hospitals}
		Loading...
	{:then hospitals}
		<div class="border-light my-3 flex flex-col">
			{#each hospitals as hospital (hospital.hospitalAdminCid)}
				<div class="flex flex-col border-zinc-200 bg-zinc-50 p-2 [&:not(:last-child)]:border-b">
					<h2 class="mb-1 font-medium">{hospital.hospitalName}</h2>
					<div class="flex items-center gap-2">
						<button onclick={() => copyToClipboard(hospital.hospitalAdminCid)}
							><Copy size={14} /></button
						>
						<p>{hospital.hospitalAdminCid}</p>
					</div>
					<div class="flex items-center gap-2">
						<button onclick={() => copyToClipboard(hospital.activationKey)}
							><Copy size={14} /></button
						>
						<p class="text-sm text-zinc-500">{hospital.activationKey}</p>
						<button
							onclick={() => {
								homeState.isLoadingUpdateActivationKey = true;
								homeState.updateActivationKey({ hospitalAdminCid: hospital.hospitalAdminCid });
							}}
						>
							{#if homeState.isLoadingUpdateActivationKey}
								<RefreshCcw size={14} class="animate-spin" />
							{:else}
								<RefreshCcw size={14} />
							{/if}
						</button>
					</div>
				</div>
			{/each}
		</div>
	{:catch e}
		<div class="my-4 flex items-center justify-center">
			<p class="text-zinc-400">{e}.</p>
		</div>
	{/await}
</div>
