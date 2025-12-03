<script lang="ts">
	import { Check, ChevronDown } from '@lucide/svelte';
	import { Select, type WithoutChildren } from 'bits-ui';

	type Props = WithoutChildren<Select.RootProps> & {
		placeholder?: string;
		items: { value: string; label: string; disabled?: boolean }[];
		contentProps?: WithoutChildren<Select.ContentProps>;
		// any other specific component props if needed
	};

	let {
		value = $bindable(),
		items,
		contentProps,
		placeholder = 'Select',
		...restProps
	}: Props = $props();

	const selectedLabel = $derived(items.find((item) => item.value === value)?.label);
</script>

<!--
TypeScript Discriminated Unions + destructing (required for "bindable") do not
get along, so we shut typescript up by casting `value` to `never`, however,
from the perspective of the consumer of this component, it will be typed appropriately.
-->
<Select.Root bind:value={value as never} {...restProps}>
	<Select.Trigger
		class="flex border border-zinc-200 bg-white max-w-max px-4 py-2 rounded-md items-center gap-4"
	>
		{selectedLabel ? selectedLabel : placeholder}
		<span><ChevronDown /></span>
	</Select.Trigger>
	<Select.Portal>
		<Select.Content
			{...contentProps}
			class="z-[100] bg-white rounded-md border border-zinc-200 p-4"
		>
			<Select.ScrollUpButton>up</Select.ScrollUpButton>
			<Select.Viewport>
				{#each items as { value, label, disabled } (value)}
					<Select.Item
						{value}
						{label}
						{disabled}
						class="flex items-center justify-start p-2 hover:bg-zinc-100 hover:rounded-md"
					>
						{#snippet children({ selected })}
							<span class="w-10">
								{#if selected}
									<Check />
								{/if}
							</span>
							{label}
						{/snippet}
					</Select.Item>
				{/each}
			</Select.Viewport>
			<Select.ScrollDownButton>down</Select.ScrollDownButton>
		</Select.Content>
	</Select.Portal>
</Select.Root>
