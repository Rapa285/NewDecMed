<script lang="ts">
	import type { CompleteProfilePersonnelSchema } from '$lib/types';
	import { cn } from '$lib/utils';
	import { Label } from 'bits-ui';
	import type { Infer, SuperValidated } from 'sveltekit-superforms';
	import { CompleteProfilePersonnelState } from './personnel-state.svelte';

	type Props = {
		completeProfilePersonnelFormData: SuperValidated<Infer<CompleteProfilePersonnelSchema>>;
	};

	let { completeProfilePersonnelFormData }: Props = $props();

	const completeProfilePersonnelState = new CompleteProfilePersonnelState({
		completeProfilePersonnelForm: completeProfilePersonnelFormData
	});

	const {
		form: completeProfilePersonnelForm,
		enhance: completeProfilePersonnelFormEnhance,
		constraints: completeProfilePersonnelFormConstraints,
		errors: completeProfilePersonnelFormErrors
	} = completeProfilePersonnelState.completeProfilePersonnelFormMeta;
</script>

<form
	method="post"
	class="flex flex-col gap-2 bg-zinc-50 border border-zinc-200 rounded-md p-2 w-full"
	use:completeProfilePersonnelFormEnhance
>
	<div
		class={cn(
			'flex flex-col w-full border border-zinc-200',
			$completeProfilePersonnelFormErrors.name && 'border-red-200'
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
			bind:value={$completeProfilePersonnelForm.name}
			{...$completeProfilePersonnelFormConstraints.name}
		/>
		{#if $completeProfilePersonnelFormErrors.name}
			<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
				>{$completeProfilePersonnelFormErrors.name[0]}</span
			>
		{/if}
	</div>
	<button type="submit" class="button-dark mt-2">Continue</button>
</form>
