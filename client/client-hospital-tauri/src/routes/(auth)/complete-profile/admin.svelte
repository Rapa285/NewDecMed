<script lang="ts">
	import type { CompleteProfileAdminSchema } from '$lib/types';
	import { cn } from '$lib/utils';
	import { Label } from 'bits-ui';
	import type { Infer, SuperValidated } from 'sveltekit-superforms';
	import { CompleteProfileAdminState } from './admin-state.svelte';

	type Props = {
		completeProfileAdminFormData: SuperValidated<Infer<CompleteProfileAdminSchema>>;
	};

	let { completeProfileAdminFormData }: Props = $props();

	let completeProfileAdminState = new CompleteProfileAdminState({
		completeProfileAdminForm: completeProfileAdminFormData
	});

	const {
		form: completeProfileAdminForm,
		enhance: completeProfileAdminFormEnhance,
		constraints: completeProfileAdminFormConstraints,
		errors: completeProfileAdminFormErrors
	} = completeProfileAdminState.completeProfileAdminFormMeta;
</script>

<form
	method="post"
	class="flex flex-col gap-2 bg-zinc-50 border border-zinc-200 rounded-md p-2 w-full"
	use:completeProfileAdminFormEnhance
>
	<div
		class={cn(
			'flex flex-col w-full border border-zinc-200',
			$completeProfileAdminFormErrors.name && 'border-red-200'
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
			bind:value={$completeProfileAdminForm.name}
			{...$completeProfileAdminFormConstraints.name}
		/>
		{#if $completeProfileAdminFormErrors.name}
			<span class="px-2 py-1 border-t border-zinc-200 text-xs font-medium text-red-500 bg-red-50"
				>{$completeProfileAdminFormErrors.name[0]}</span
			>
		{/if}
	</div>
	<button type="submit" class="button-dark mt-2">Continue</button>
</form>
