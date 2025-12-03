<script lang="ts">
	import Dialog from '$lib/components/dialog.svelte';
	import { JPEG_MIME_TYPE, PNG_MIME_TYPE } from '$lib/constants.js';
	import { enterPinSchema, hospitalQrSchema } from '$lib/schema.js';
	import type { InvokeProcessQrResponse, SuccessResponse } from '$lib/types.js';
	import { tryCatchAsVal } from '$lib/utils';
	import { Loader, Loader2 } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Button, PinInput, REGEXP_ONLY_DIGITS } from 'bits-ui';
	import { toast } from 'svelte-sonner';
	import { fileProxy, superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';

	let { data } = $props();

	let isConfirmDialogOpen = $state(false);
	let isEnterPinDialogOpen = $state(false);
	let confirmDialogData = $state<InvokeProcessQrResponse>();

	const {
		form: hospitalQrForm,
		enhance: hospitalQrFormEnhance,
		errors: hospitalQrFormErrors,
		constraints: hospitalQrFormConstraints,
		delayed: hospitalQrFormDelayed
	} = superForm(data.hospitalQrForm, {
		SPA: true,
		validators: zodClient(hospitalQrSchema),
		delayMs: 100,
		onUpdate: async ({ form, result }) => {
			if (result.type === 'success') {
				let qrBytes = await form.data.qr.bytes();
				let resInvokeProcessQr = await tryCatchAsVal(async () => {
					return (await invoke('process_qr', {
						qrBytes
					})) as SuccessResponse<InvokeProcessQrResponse>;
				});

				if (resInvokeProcessQr.success) {
					isConfirmDialogOpen = true;
					confirmDialogData = resInvokeProcessQr.data.data;
					return;
				}

				toast.error(resInvokeProcessQr.error);
			}
		}
	});
	const qrFileProxy = fileProxy(hospitalQrForm, 'qr');

	const {
		form: enterPinForm,
		enhance: enterPinFormEnhance,
		errors: enterPinFormErrors,
		delayed: enterPinFormDelayed
	} = superForm(data.enterPinForm, {
		SPA: true,
		validators: zodClient(enterPinSchema),
		delayMs: 100,
		onUpdate: async ({ form, result, cancel }) => {
			if (result.type === 'success') {
				const resInvokeCreateAccess = await tryCatchAsVal(async () => {
					return (await invoke('create_access', { pin: form.data.pin })) as SuccessResponse<null>;
				});

				if (!resInvokeCreateAccess.success && resInvokeCreateAccess.error === 'Invalid PIN') {
					enterPinFormErrors.update((val) => {
						val.pin = [resInvokeCreateAccess.error];
						return val;
					});
					cancel();
					return;
				}

				if (resInvokeCreateAccess.success) {
					toast.success('Success to give access');
				}

				if (!resInvokeCreateAccess.success) {
					console.log(resInvokeCreateAccess.error);
					toast.error(resInvokeCreateAccess.error);
				}

				isEnterPinDialogOpen = false;
			}
		}
	});
</script>

<Dialog
	bind:open={isConfirmDialogOpen}
	contentProps={{
		escapeKeydownBehavior: 'ignore',
		onInteractOutside: (e) => e.preventDefault()
	}}
	withCloseButton={true}
	withTrigger={false}
	closeButtonEvent={() => {}}
>
	{#snippet title()}Confirm Access{/snippet}

	<div class="flex flex-col my-2">
		<p>Are you sure to give the following person access to your data?</p>
		<div class="grid grid-cols-[100px_1fr] p-2 border border-zinc-200 bg-zinc-50 rounded-md my-2">
			<p>Name:</p>
			<p>{confirmDialogData?.hospitalPersonnelName}</p>
			<p>Hospital:</p>
			<p>{confirmDialogData?.hospitalPersonnelHospitalName}</p>
		</div>
	</div>

	<Button.Root
		type="button"
		class="button-dark mt-2"
		onclick={() => {
			isConfirmDialogOpen = false;
			isEnterPinDialogOpen = true;
		}}>Confirm</Button.Root
	>
</Dialog>

<Dialog
	bind:open={isEnterPinDialogOpen}
	contentProps={{
		escapeKeydownBehavior: 'ignore',
		onInteractOutside: (e) => e.preventDefault()
	}}
	withCloseButton={true}
	withTrigger={false}
	closeButtonEvent={() => {}}
>
	{#snippet title()}
		Enter PIN
	{/snippet}
	<form use:enterPinFormEnhance>
		<PinInput.Root
			maxlength={6}
			pattern={REGEXP_ONLY_DIGITS}
			name="confirmPin"
			class="flex items-center gap-2"
			bind:value={$enterPinForm.pin}
		>
			{#snippet children({ cells })}
				{#each cells as cell}
					<PinInput.Cell
						{cell}
						class="size-10 border border-zinc-200 flex items-center justify-center relative"
					>
						{#if cell.char !== null}
							<div class="size-6 rounded-full bg-zinc-800"></div>
						{:else}
							<div class="size-6 rounded-full bg-zinc-100"></div>
						{/if}
						{#if cell.hasFakeCaret}
							<div class="pointer-events-none absolute inset-0 flex items-center justify-center">
								<div class="h-6 w-2 bg-blue-500"></div>
							</div>
						{/if}
					</PinInput.Cell>
				{/each}
			{/snippet}
		</PinInput.Root>
		{#if $enterPinFormErrors.pin}
			<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
				>{$enterPinFormErrors.pin[0]}</span
			>
		{/if}
		<Button.Root type="submit" class="button-dark mt-2 flex items-center justify-center">
			{#if $enterPinFormDelayed}
				<Loader2 class="animate-spin" />
			{:else}
				Enter
			{/if}
		</Button.Root>
	</form>
</Dialog>

<div class="flex flex-col">
	<h2 class="font-montserrat font-medium text-xl my-2">Scan Hospital QR</h2>
	<div class="bg-zinc-100 border border-zinc-200 rounded-md h-56 flex items-center justify-center">
		<p>Scan Placeholder</p>
	</div>
	<form
		method="post"
		class="flex flex-col gap-2 my-4"
		enctype="multipart/form-data"
		use:hospitalQrFormEnhance
		{...$hospitalQrFormConstraints.qr}
	>
		<label for="qr" class="font-medium">Input QR</label>
		<input
			id="qr"
			name="qr"
			type="file"
			class="border border-zinc-200 p-4 bg-zinc-100 placeholder-cyan-950 rounded-md"
			accept={`${PNG_MIME_TYPE}, ${JPEG_MIME_TYPE}`}
			bind:files={$qrFileProxy}
		/>
		{#if $hospitalQrFormErrors.qr}
			<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
				>{$hospitalQrFormErrors.qr[0]}</span
			>
		{/if}
		<button
			type="submit"
			class="button-dark disabled:bg-zinc-700"
			disabled={$hospitalQrFormDelayed}
		>
			{#if $hospitalQrFormDelayed}
				<Loader class="animate-spin" />
			{:else}
				Submit
			{/if}
		</button>
	</form>
</div>
