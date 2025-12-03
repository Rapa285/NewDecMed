<script lang="ts">
	import { SIGNUP_TOTAL_STEP } from '$lib/constants';
	import { cn } from '$lib/utils';
	import { Button, Label, PinInput, REGEXP_ONLY_DIGITS } from 'bits-ui';
	import { SignUpState } from './state.svelte';
	import SuperDebug from 'sveltekit-superforms';

	let { data } = $props();
	const signUpState = new SignUpState({ signUpForm: data.signUpForm });
	const {
		form: signUpForm,
		enhance: signUpFormEnhance,
		constraints: signUpFormConstraints,
		errors: signUpFormErrors
	} = signUpState.signUpFormMeta;
</script>

<div class="flex flex-1 flex-col w-full">
	<div class="flex flex-col p-4 w-full border rounded-t-lg border-zinc-200">
		<h2 class="font-montserrat font-bold text-2xl">DecMed</h2>
		<p class="text-sm">Decentralized EMR Management System</p>
	</div>
	<div class="flex flex-1 w-full">
		<div
			class="flex flex-col bg-zinc-50 border border-t-0 rounded-bl-lg border-zinc-200 p-4 items-center"
		>
			{#each new Array(SIGNUP_TOTAL_STEP) as _, i (i)}
				<div
					class={cn(
						'size-10 rounded-full bg-white border border-zinc-200 flex items-center justify-center',
						signUpState.currentStep >= i + 1 && 'bg-zinc-800 text-zinc-100'
					)}
				>
					<span>{i + 1}</span>
				</div>
				{#if i < SIGNUP_TOTAL_STEP - 1}
					<div class="flex flex-1 flex-col border-l border-zinc-200"></div>
				{/if}
			{/each}
		</div>
		<div
			class="flex flex-1 flex-col w-full border border-l-0 border-t-0 border-zinc-200 rounded-br-lg p-4"
		>
			<div class="flex flex-col max-w-2xl w-full mx-auto flex-1">
				<form method="post" use:signUpFormEnhance class="flex flex-col flex-1 w-full">
					<div class="flex-1 flex flex-col justify-center w-full gap-4">
						<h3 class="font-medium">Register</h3>
						<!-- <SuperDebug data={$signUpForm} /> -->
						{#if signUpState.currentStep === 1}
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
						{#if signUpState.currentStep === 2}
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
						{#if signUpState.currentStep === 3}
							<p>Seed Words</p>
							<p>{signUpState.mnemonic}</p>
							<button class="border p-2" type="button" onclick={signUpState.copyMnemonic}
								>copy</button
							>
						{/if}
						{#if signUpState.currentStep === 4}
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
					</div>
					<div class="flex items-center justify-center flex-col gap-2">
						<Button.Root type="submit" class="button-dark mt-2">Next</Button.Root>
					</div>
				</form>
			</div>
		</div>
	</div>
</div>
