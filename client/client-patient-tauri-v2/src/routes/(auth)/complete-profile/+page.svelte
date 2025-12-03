<script lang="ts">
	import DateField from '$lib/components/date-field.svelte';
	import { completeProfileSchema } from '$lib/schema';
	import type { SuccessResponse } from '$lib/types.js';
	import { cn, tryCatchAsVal, waitMs } from '$lib/utils';
	import { Loader2 } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Label } from 'bits-ui';
	import { toast } from 'svelte-sonner';
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { type DateValue } from '@internationalized/date';
	import Select from '$lib/components/select.svelte';

	let { data } = $props();

	let dateOfBirth = $state<DateValue>();

	const {
		form: completeProfileForm,
		enhance: completeProfileFormEnhance,
		constraints: completeProfileFormConstraints,
		errors: completeProfileFormErrors,
		delayed: completeProfileFormDelayed
	} = superForm(data.completeProfileForm, {
		delayMs: 100,
		validators: zodClient(completeProfileSchema),
		dataType: 'json',
		SPA: true,
		onUpdate: async ({ result, form, cancel }) => {
			if (result.type === 'success') {
				const resultInvokeUpdateProfile = await tryCatchAsVal(async () => {
					return (await invoke('update_profile', {
						data: {
							name: form.data.name,
							birthPlace: form.data.birthPlace,
							dateOfBirth: form.data.dateOfBirth,
							gender: form.data.gender,
							religion: form.data.religion,
							education: form.data.education,
							occupation: form.data.occupation,
							maritalStatus: form.data.maritalStatus
						}
					})) as SuccessResponse<null>;
				});

				if (!resultInvokeUpdateProfile.success) {
					cancel();
					toast.error(resultInvokeUpdateProfile.error);
					return;
				}

				// wait for 2 seconds for transaction be submitted
				await waitMs(2 * 1000);

				toast.success('Profile updated successfully');
			}
		}
	});

	$effect(() => {
		completeProfileForm.update((prev) => {
			prev.dateOfBirth = dateOfBirth?.toString() ?? '';
			return prev;
		});
	});
</script>

<div class="flex flex-col max-w-md w-full flex-1 items-center justify-center p-4">
	<h2 class="font-montserrat text-xl font-medium my-4">Complete Your Profile</h2>
	<form
		method="post"
		class="flex flex-col gap-2 bg-zinc-50 border border-zinc-200 rounded-md p-2 w-full"
		use:completeProfileFormEnhance
	>
		<div
			class={cn(
				'flex flex-col w-full border border-zinc-200',
				$completeProfileFormErrors.name && 'border-red-200'
			)}
		>
			<Label.Root
				for="name"
				class="font-medium text-sm after:content-['*'] after:text-red-500 p-2 border-b border-zinc-200"
				>Name</Label.Root
			>
			<input
				type="text"
				id="name"
				name="name"
				class="p-2 outline-0 bg-white"
				placeholder="xxx-xxxxxxxx"
				bind:value={$completeProfileForm.name}
				{...$completeProfileFormConstraints.name}
			/>
			{#if $completeProfileFormErrors.name}
				<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
					>{$completeProfileFormErrors.name[0]}</span
				>
			{/if}
		</div>
		<div
			class={cn(
				'flex flex-col w-full border border-zinc-200',
				$completeProfileFormErrors.birthPlace && 'border-red-200'
			)}
		>
			<Label.Root
				for="birthPlace"
				class="font-medium text-sm after:content-['*'] after:text-red-500 p-2 border-b border-zinc-200"
				>Birth Place</Label.Root
			>
			<input
				type="text"
				id="birthPlace"
				name="birthPlace"
				class="p-2 outline-0 bg-white"
				placeholder="xxx-xxxxxxxx"
				bind:value={$completeProfileForm.birthPlace}
				{...$completeProfileFormConstraints.birthPlace}
			/>
			{#if $completeProfileFormErrors.birthPlace}
				<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
					>{$completeProfileFormErrors.birthPlace[0]}</span
				>
			{/if}
		</div>
		<div
			class={cn(
				'flex flex-col w-full border border-zinc-200',
				$completeProfileFormErrors.dateOfBirth && 'border-red-200'
			)}
		>
			<Label.Root
				for="dateOfBirth"
				class="font-medium text-sm after:content-['*'] after:text-red-500 p-2 border-b border-zinc-200"
				>Date of Birth</Label.Root
			>
			<DateField bind:value={dateOfBirth} />
			{#if $completeProfileFormErrors.dateOfBirth}
				<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
					>{$completeProfileFormErrors.dateOfBirth[0]}</span
				>
			{/if}
		</div>
		<div
			class={cn(
				'flex flex-col w-full border border-zinc-200',
				$completeProfileFormErrors.gender && 'border-red-200'
			)}
		>
			<Label.Root
				for="gender"
				class="font-medium text-sm after:content-['*'] after:text-red-500 p-2 border-b border-zinc-200"
				>Gender</Label.Root
			>
			<Select
				items={[
					{ value: 'Male', label: 'Male' },
					{ value: 'Female', label: 'Female' }
				]}
				bind:value={$completeProfileForm.gender}
				type="single"
			/>
			{#if $completeProfileFormErrors.gender}
				<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
					>{$completeProfileFormErrors.gender[0]}</span
				>
			{/if}
		</div>
		<div
			class={cn(
				'flex flex-col w-full border border-zinc-200',
				$completeProfileFormErrors.religion && 'border-red-200'
			)}
		>
			<Label.Root
				for="religion"
				class="font-medium text-sm after:content-['*'] after:text-red-500 p-2 border-b border-zinc-200"
				>Religion</Label.Root
			>
			<input
				type="text"
				id="religion"
				name="religion"
				class="p-2 outline-0 bg-white"
				placeholder="xxx-xxxxxxxx"
				bind:value={$completeProfileForm.religion}
				{...$completeProfileFormConstraints.religion}
			/>
			{#if $completeProfileFormErrors.religion}
				<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
					>{$completeProfileFormErrors.religion[0]}</span
				>
			{/if}
		</div>
		<div
			class={cn(
				'flex flex-col w-full border border-zinc-200',
				$completeProfileFormErrors.education && 'border-red-200'
			)}
		>
			<Label.Root
				for="education"
				class="font-medium text-sm after:content-['*'] after:text-red-500 p-2 border-b border-zinc-200"
				>Education</Label.Root
			>
			<input
				type="text"
				id="education"
				name="education"
				class="p-2 outline-0 bg-white"
				placeholder="xxx-xxxxxxxx"
				bind:value={$completeProfileForm.education}
				{...$completeProfileFormConstraints.education}
			/>
			{#if $completeProfileFormErrors.education}
				<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
					>{$completeProfileFormErrors.education[0]}</span
				>
			{/if}
		</div>
		<div
			class={cn(
				'flex flex-col w-full border border-zinc-200',
				$completeProfileFormErrors.occupation && 'border-red-200'
			)}
		>
			<Label.Root
				for="occupation"
				class="font-medium text-sm after:content-['*'] after:text-red-500 p-2 border-b border-zinc-200"
				>Occupation</Label.Root
			>
			<input
				type="text"
				id="occupation"
				name="occupation"
				class="p-2 outline-0 bg-white"
				placeholder="xxx-xxxxxxxx"
				bind:value={$completeProfileForm.occupation}
				{...$completeProfileFormConstraints.occupation}
			/>
			{#if $completeProfileFormErrors.occupation}
				<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
					>{$completeProfileFormErrors.occupation[0]}</span
				>
			{/if}
		</div>
		<div
			class={cn(
				'flex flex-col w-full border border-zinc-200',
				$completeProfileFormErrors.maritalStatus && 'border-red-200'
			)}
		>
			<Label.Root
				for="maritalStatus"
				class="font-medium text-sm after:content-['*'] after:text-red-500 p-2 border-b border-zinc-200"
				>Marital Status</Label.Root
			>
			<Select
				items={[
					{ value: 'Single', label: 'Single' },
					{ value: 'Married', label: 'Married' },
					{ value: 'Widowed', label: 'Widowed' },
					{ value: 'Divorced', label: 'Divorced' }
				]}
				bind:value={$completeProfileForm.maritalStatus}
				type="single"
			/>
			{#if $completeProfileFormErrors.maritalStatus}
				<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
					>{$completeProfileFormErrors.maritalStatus[0]}</span
				>
			{/if}
		</div>
		<button type="submit" class="button-dark mt-2 flex items-center justify-center">
			{#if $completeProfileFormDelayed}
				<Loader2 class="animate-spin" />
			{:else}
				Continue
			{/if}
		</button>
	</form>
</div>
