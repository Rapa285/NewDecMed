<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { SIGNUP_TOTAL_STEP } from '$lib/constants';
	import { signUpSchemas } from '$lib/schema';
	import type { SuccessResponse } from '$lib/types.js';
	import { cn, tryCatchAsVal, waitMs } from '$lib/utils';
	import { Loader2 } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Button, Label, PinInput, REGEXP_ONLY_DIGITS } from 'bits-ui';
	import { toast } from 'svelte-sonner';
	import SuperDebug, { superForm } from 'sveltekit-superforms';
	import { zod } from 'sveltekit-superforms/adapters';

	let { data } = $props();

	let currentStep = $state<number>(1);
	let mnemonic = $state<string>('');

	const {
		constraints: signUpFormConstraints,
		delayed: signUpFormDelayed,
		enhance: signUpFormEnhance,
		errors: signUpFormErrors,
		form: signUpForm,
		options: signUpFormOptions,
		validateForm: signUpFormValidateForm
	} = superForm(data.signUpForm, {
		validators: false,
		dataType: 'json',
		delayMs: 100,
		SPA: true,
		onSubmit: async ({ cancel, formData }) => {
			if (currentStep === SIGNUP_TOTAL_STEP) return;
			cancel();

			const result = await signUpFormValidateForm({ update: true });
			let valid = true;

			if (result.valid) {
				switch (currentStep) {
					case 1: {
						const pin = formData.get('pin') as string;
						const resInvokeValidatePin = await tryCatchAsVal(async () => {
							return (await invoke('validate_pin', {
								pin,
								authType: 'Signup'
							})) as SuccessResponse<null>;
						});

						valid = resInvokeValidatePin.success;

						if (!resInvokeValidatePin.success) {
							signUpFormErrors.update((val) => {
								val.pin = [resInvokeValidatePin.error];
								return val;
							});

							break;
						}

						break;
					}
					case 2: {
						const confirmPin = formData.get('confirmPin') as string;
						const resInvokeValidateConfirmPin = await tryCatchAsVal(async () => {
							return (await invoke('validate_confirm_pin', {
								confirmPin,
								authType: 'Signup'
							})) as SuccessResponse<boolean>;
						});

						valid = resInvokeValidateConfirmPin.success;

						if (!resInvokeValidateConfirmPin.success) {
							signUpFormErrors.update((val) => {
								val.confirmPin = [resInvokeValidateConfirmPin.error];
								return val;
							});

							break;
						}

						break;
					}
					case 4: {
						const seedWords = formData.get('seedWords') as string;
						const resInvokeValidateSeedWords = await tryCatchAsVal(async () => {
							return (await invoke('validate_seed_words', {
								seedWords,
								authType: 'Signup'
							})) as SuccessResponse<boolean>;
						});

						valid = resInvokeValidateSeedWords.success;

						if (!resInvokeValidateSeedWords.success) {
							signUpFormErrors.update((val) => {
								val.seedWords = [resInvokeValidateSeedWords.error];
								return val;
							});
							break;
						}

						break;
					}
				}
			} else {
				valid = false;
			}

			if (valid) currentStep += 1;

			if (currentStep == 3) {
				const resInvokeGenerateMnemonic = await tryCatchAsVal(async () => {
					return (await invoke('generate_mnemonic')) as SuccessResponse<string>;
				});

				if (!resInvokeGenerateMnemonic.success) {
					toast.error(resInvokeGenerateMnemonic.error);
					invalidateAll();
				} else {
					mnemonic = resInvokeGenerateMnemonic.data.data;
				}
			}
		},
		onUpdate: async ({ form, result, cancel }) => {
			let valid = false;
			if (result.type === 'success') {
				const resInvokeSignup = await tryCatchAsVal(async () => {
					return (await invoke('signup', { id: form.data.nik })) as SuccessResponse<null>;
				});

				if (!resInvokeSignup.success) {
					cancel();
					toast.error(resInvokeSignup.error);
				} else {
					valid = true;
				}
			}

			if (!valid) {
				signUpFormErrors.update((val) => {
					val.nik = ['NIK already registered'];
					return val;
				});
			} else {
				await waitMs(2 * 1000);
			}
		}
	});

	const copyMnemonic = async () => {
		await navigator.clipboard.writeText(mnemonic);
	};

	$effect(() => {
		signUpFormOptions.validators = zod(signUpSchemas[currentStep - 1]);
	});
</script>

<div class="flex flex-1 flex-col w-full">
	<div class="flex flex-col p-4 w-full border rounded-t-lg border-zinc-200">
		<h2 class="font-montserrat font-bold text-2xl">DecMed</h2>
		<p class="text-sm">Decentralized EMR Management System</p>
	</div>
	<div class="flex flex-col flex-1 w-full">
		<div class="flex bg-zinc-50 border border-t-0 rounded-bl-lg border-zinc-200 p-4 items-center">
			{#each new Array(SIGNUP_TOTAL_STEP) as _, i (i)}
				<div
					class={cn(
						'size-10 rounded-full bg-white border border-zinc-200 flex items-center justify-center',
						currentStep >= i + 1 && 'bg-zinc-800 text-zinc-100'
					)}
				>
					<span>{i + 1}</span>
				</div>
				{#if i < SIGNUP_TOTAL_STEP - 1}
					<div class="flex flex-1 flex-col border-t border-zinc-200"></div>
				{/if}
			{/each}
		</div>
		<div
			class="flex flex-1 flex-col w-full border border-l-0 border-t-0 border-zinc-200 rounded-br-lg p-4"
		>
			<div class="flex flex-col max-w-2xl w-full mx-auto flex-1">
				<form method="post" use:signUpFormEnhance class="flex flex-col flex-1 w-full">
					<div class="flex-1 flex flex-col justify-center w-full gap-4">
						<h3 class="font-medium">Sign Up</h3>
						<!-- <SuperDebug data={$signUpForm} /> -->
						{#if currentStep === 1}
							<p>Enter PIN:</p>

							<PinInput.Root
								maxlength={6}
								pattern={REGEXP_ONLY_DIGITS}
								name="pin"
								class="flex items-center gap-2"
								bind:value={$signUpForm.pin}
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
												<div
													class="pointer-events-none absolute inset-0 flex items-center justify-center"
												>
													<div class="h-6 w-2 bg-blue-500"></div>
												</div>
											{/if}
										</PinInput.Cell>
									{/each}
								{/snippet}
							</PinInput.Root>
							{#if $signUpFormErrors.pin}
								<span
									class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
									>{$signUpFormErrors.pin[0]}</span
								>
							{/if}
						{/if}
						{#if currentStep === 2}
							<p>Re-Enter PIN:</p>

							<PinInput.Root
								maxlength={6}
								pattern={REGEXP_ONLY_DIGITS}
								name="confirmPin"
								class="flex items-center gap-2"
								bind:value={$signUpForm.confirmPin}
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
												<div
													class="pointer-events-none absolute inset-0 flex items-center justify-center"
												>
													<div class="h-6 w-2 bg-blue-500"></div>
												</div>
											{/if}
										</PinInput.Cell>
									{/each}
								{/snippet}
							</PinInput.Root>
							{#if $signUpFormErrors.confirmPin}
								<span
									class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
									>{$signUpFormErrors.confirmPin[0]}</span
								>
							{/if}
						{/if}
						{#if currentStep === 3}
							<p>Seed Words</p>
							<p>{mnemonic}</p>
							<button class="border p-2" type="button" onclick={copyMnemonic}>copy</button>
						{/if}
						{#if currentStep === 4}
							<div
								class={cn(
									'flex flex-col w-full border border-zinc-200',
									$signUpFormErrors.seedWords && 'border-red-200'
								)}
							>
								<Label.Root
									for="seedWords"
									class="font-medium text-sm after:content-['*'] after:text-red-500 p-2 border-b border-zinc-200"
									>seedWords</Label.Root
								>
								<input
									type="text"
									id="seedWords"
									name="seedWords"
									class="p-2 outline-0 bg-white"
									placeholder="xxx-xxxxxxxx"
									bind:value={$signUpForm.seedWords}
									{...$signUpFormConstraints.seedWords}
								/>
								{#if $signUpFormErrors.seedWords}
									<span
										class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
										>{$signUpFormErrors.seedWords[0]}</span
									>
								{/if}
							</div>
						{/if}
						{#if currentStep === 5}
							<div
								class={cn(
									'flex flex-col w-full border border-zinc-200',
									$signUpFormErrors.nik && 'border-red-200'
								)}
							>
								<Label.Root
									for="nik"
									class="font-medium text-sm after:content-['*'] after:text-red-500 p-2 border-b border-zinc-200"
									>NIK</Label.Root
								>
								<input
									type="text"
									id="nik"
									name="nik"
									class="p-2 outline-0 bg-white"
									placeholder="xxxxxxxxxxxxxxxx"
									bind:value={$signUpForm.nik}
									{...$signUpFormConstraints.nik}
								/>
								{#if $signUpFormErrors.nik}
									<span
										class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
										>{$signUpFormErrors.nik[0]}</span
									>
								{/if}
							</div>
						{/if}
					</div>
					<div class="flex items-center justify-center flex-col gap-2">
						<Button.Root type="submit" class="button-dark mt-2 flex items-center justify-center">
							{#if currentStep == 5 && $signUpFormDelayed}
								<Loader2 class="animate-spin" />
							{:else}
								Next
							{/if}
						</Button.Root>
						<p>
							Doesn't have an account? <a href="/signin" class="underline underline-offset-4"
								>Signin</a
							>
						</p>
					</div>
				</form>
			</div>
		</div>
	</div>
</div>
