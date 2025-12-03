<script lang="ts">
	import Dialog from '$lib/components/dialog.svelte';
	import Select from '$lib/components/select.svelte';
	import type { AddPersonnelSchemaStep2 } from '$lib/types';
	import type { Infer, SuperValidated } from 'sveltekit-superforms';
	import { AdminHomeState } from './admin-state.svelte';
	import { Button, Label, PinInput, REGEXP_ONLY_DIGITS } from 'bits-ui';
	import { cn } from '$lib/utils';
	import { Copy } from '@lucide/svelte';

	type Props = {
		addPersonnelFormData: SuperValidated<Infer<AddPersonnelSchemaStep2>>;
	};

	let { addPersonnelFormData }: Props = $props();

	const adminHomeState = new AdminHomeState({ addPersonnelForm: addPersonnelFormData });
	const {
		form: addPersonnelForm,
		enhance: addPersonnelFormEnhance,
		constraints: addPersonnelFormConstraints,
		errors: addPersonnelFormErrors,
		reset: addPersonnelFormReset
	} = adminHomeState.addPersonnelFormMeta;
</script>

<div class="flex flex-col w-full flex-1">
	<div class="flex items-center justify-between">
		<h2 class="font-medium">Hospital Personnels</h2>
		<Dialog
			onOpenChange={(open) => {
				if (!open) {
					addPersonnelFormReset();
				}
			}}
			buttonText="+ Personnel"
			contentProps={{
				escapeKeydownBehavior: 'ignore',
				onInteractOutside: (e) => e.preventDefault()
			}}
			withCloseButton={true}
			bind:open={adminHomeState.addPersonnelDialogOpen}
		>
			{#snippet title()}
				Add Personnel
			{/snippet}
			<form
				id="addPersonnelForm"
				method="post"
				class="flex flex-col gap-2 mt-4"
				use:addPersonnelFormEnhance
			>
				<div class={cn('flex flex-col w-full gap-2')}>
					<Label.Root for="id" class="font-medium after:content-['*'] after:text-red-500"
						>ID</Label.Root
					>
					<input
						type="text"
						id="id"
						name="id"
						class="input-text"
						placeholder="xxx-xxxxxxxx"
						bind:value={$addPersonnelForm.id}
						{...$addPersonnelFormConstraints.id}
					/>
					{#if $addPersonnelFormErrors.id}
						<span
							class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
							>{$addPersonnelFormErrors.id[0]}</span
						>
					{/if}
				</div>
				<label for="role" class="font-medium after:content-['*'] after:text-red-500">Role</label>
				<Select items={adminHomeState.roles} bind:value={$addPersonnelForm.role} type="single" />
				<Button.Root type="submit" class="button-dark mt-2">Add</Button.Root>
				<Dialog
					buttonText="Add personnel"
					bind:open={adminHomeState.askPin}
					contentProps={{
						escapeKeydownBehavior: 'ignore',
						onInteractOutside: (e) => e.preventDefault()
					}}
					withCloseButton={true}
					withTrigger={false}
					closeButtonEvent={() => {
						adminHomeState.currentStep -= 1;
						$addPersonnelForm.pin = '';
					}}
				>
					{#snippet title()}
						Enter PIN
					{/snippet}
					<PinInput.Root
						maxlength={6}
						pattern={REGEXP_ONLY_DIGITS}
						name="confirmPin"
						class="flex items-center gap-2"
						bind:value={$addPersonnelForm.pin}
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
					{#if $addPersonnelFormErrors.pin}
						<span
							class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
							>{$addPersonnelFormErrors.pin[0]}</span
						>
					{/if}
					<Button.Root type="submit" class="button-dark mt-2" form="addPersonnelForm"
						>Enter</Button.Root
					>
				</Dialog>
			</form>
		</Dialog>
	</div>
	<div class="flex flex-col my-4 bg-white">
		{#await adminHomeState.refetchPersonnels}
			Loading..
		{:then personnels}
			{#if personnels && personnels.length > 0}
				{#each personnels as personnel, i (i)}
					<div
						class="flex flex-col p-2 border [&:not(:last-child)]:border-b-0 w-full border-zinc-200"
					>
						<div class="flex items-center justify-between gap-2">
							<p class="text-zinc-400 text-sm">{personnel.id}</p>
							<p
								class="px-2 py-0.5 border border-zinc-200 bg-zinc-50 text-xs rounded-lg text-zinc-400"
							>
								{personnel.role}
							</p>
						</div>
						<div class="flex items-center gap-2">
							<button
								onclick={() => {
									navigator.clipboard.writeText(personnel.activation_key);
								}}
								class="cursor-pointer"
							>
								<Copy size={14} />
							</button>
							<p>{personnel.activation_key}</p>
						</div>
					</div>
				{/each}
			{:else}
				<p>No personnels.</p>
			{/if}
		{/await}
	</div>
</div>
