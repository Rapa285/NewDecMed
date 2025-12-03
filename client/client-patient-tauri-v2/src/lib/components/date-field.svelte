<script lang="ts">
	import { DateField, type WithoutChildrenOrChild } from 'bits-ui';

	let {
		value = $bindable(),
		placeholder = $bindable(),
		...restProps
	}: WithoutChildrenOrChild<DateField.RootProps> = $props();
</script>

<DateField.Root bind:value bind:placeholder locale="en" {...restProps}>
	<DateField.Input class="p-2">
		{#snippet children({ segments })}
			{#each segments as { part, value }}
				<DateField.Segment {part}>
					{value}
				</DateField.Segment>
			{/each}
		{/snippet}
	</DateField.Input>
</DateField.Root>
