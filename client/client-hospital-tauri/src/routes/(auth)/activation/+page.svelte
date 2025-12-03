<script lang="ts">
	import { Label, Button } from 'bits-ui';
	import { superForm } from 'sveltekit-superforms';
	import { zod } from 'sveltekit-superforms/adapters';
	import { activationSchema } from '$lib/schema';
	import { cn, tryCatchAsVal } from '$lib/utils';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';
	import type { InvokeGlobalAdminAddActivationKeyData, SuccessResponse } from '$lib/types.js';

	let { data } = $props();

	const {
		form: activationForm,
		errors: activationFormErrors,
		constraints: activationFormConstrains,
		enhance: activationFormEnhance
	} = superForm(data.activationForm, {
		SPA: true,
		validators: zod(activationSchema),
		onUpdate: async ({ form, result, cancel }) => {
			if (result.type === 'success') {
				console.log(form.data);
				const resInvokeActivateApp = await tryCatchAsVal(async () => {
					return (await invoke('activate_app', {
						activationKey: form.data.activationKey,
						id: form.data.id
					})) as SuccessResponse<null>;
				});

				if (!resInvokeActivateApp.success) {
					console.log(resInvokeActivateApp.error);
					toast.error(resInvokeActivateApp.error);
					cancel();
					return;
				}
			}
		}
	});

	async function addActivationKey() {
		const resInvokeGlobalAdminAddActivationKey = await tryCatchAsVal(async () => {
			return (await invoke(
				'global_admin_add_activation_key'
			)) as SuccessResponse<InvokeGlobalAdminAddActivationKeyData>;
		});

		if (!resInvokeGlobalAdminAddActivationKey.success) {
			console.log(resInvokeGlobalAdminAddActivationKey.error);
			toast.error(resInvokeGlobalAdminAddActivationKey.error);
			return;
		}

		toast.success('Success');
		console.log(resInvokeGlobalAdminAddActivationKey.data);
	}
</script>

<div class="flex flex-col border border-zinc-200 items-center max-w-md w-full">
	<div class="flex flex-col p-4 w-full border-b border-zinc-200">
		<h2 class="font-montserrat font-bold text-2xl">DecMed</h2>
		<p class="text-sm">Decentralized EMR Management System</p>
	</div>
	<form method="post" use:activationFormEnhance class="flex flex-col w-full bg-stone-50 p-3 gap-3">
		<div
			class={cn(
				'flex flex-col w-full border border-zinc-200',
				$activationFormErrors.id && 'border-red-200'
			)}
		>
			<Label.Root
				for="id"
				class="font-medium text-sm after:content-['*'] after:text-red-500 p-2 border-b border-zinc-200"
				>CID</Label.Root
			>
			<input
				type="text"
				id="id"
				name="id"
				class="p-2 outline-0 bg-white"
				placeholder="xxx-xxxxxxxx"
				bind:value={$activationForm.id}
				{...$activationFormConstrains.id}
			/>
			{#if $activationFormErrors.id}
				<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
					>{$activationFormErrors.id[0]}</span
				>
			{/if}
		</div>
		<div
			class={cn(
				'flex flex-col w-full border border-zinc-200',
				$activationFormErrors.activationKey && 'border-red-200'
			)}
		>
			<Label.Root
				for="activationKey"
				class="font-medium text-sm after:content-['*'] after:text-red-500 p-2 border-b border-zinc-200"
				>Activation Key</Label.Root
			>
			<input
				type="text"
				id="activationKey"
				name="activationKey"
				class="p-2 outline-0 bg-white"
				placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
				bind:value={$activationForm.activationKey}
				{...$activationFormConstrains.activationKey}
			/>
			{#if $activationFormErrors.activationKey}
				<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
					>{$activationFormErrors.activationKey[0]}</span
				>
			{/if}
		</div>

		<Button.Root type="submit" class="button-dark mt-2">Activate</Button.Root>
	</form>
</div>
