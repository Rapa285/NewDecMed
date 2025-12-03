<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Dialog, type WithoutChild } from 'bits-ui';

	type Props = Dialog.RootProps & {
		title: Snippet;
		contentProps?: WithoutChild<Dialog.ContentProps>;
		withCloseButton?: boolean;
		trigger?: Snippet;
		closeButtonEvent?: () => void;
	};

	let {
		open = $bindable(false),
		children,
		contentProps,
		title,
		trigger,
		withCloseButton = false,
		closeButtonEvent = () => {},
		...restProps
	}: Props = $props();
</script>

<Dialog.Root bind:open {...restProps}>
	{#if trigger}
		<Dialog.Trigger class="max-w-max rounded-lg bg-zinc-800 px-4 py-1 text-zinc-200">
			{@render trigger()}
		</Dialog.Trigger>
	{/if}
	<Dialog.Portal>
		<Dialog.Overlay class="fixed inset-0 z-50 bg-zinc-800/40" />
		<Dialog.Content
			{...contentProps}
			class="fixed top-1/2 left-1/2 z-50 flex w-full max-w-xl translate-x-[-50%] translate-y-[-50%] flex-col rounded-md border border-zinc-200 bg-white p-4 outline-hidden"
		>
			<Dialog.Title class="text-xl font-medium">
				{@render title()}
			</Dialog.Title>
			{@render children?.()}
			{#if withCloseButton}
				<Dialog.Close class="mt-2 rounded-lg bg-zinc-100 px-4 py-1" onclick={closeButtonEvent}
					>Cancel</Dialog.Close
				>
			{/if}
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
