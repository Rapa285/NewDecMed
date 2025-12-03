<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Dialog, type WithoutChild } from 'bits-ui';

	type Props = Dialog.RootProps & {
		buttonText?: string;
		title: Snippet;
		contentProps?: WithoutChild<Dialog.ContentProps>;
		withCloseButton?: boolean;
		withTrigger?: boolean;
		closeButtonEvent?: () => void;
	};

	let {
		open = $bindable(false),
		children,
		buttonText = 'Open',
		contentProps,
		title,
		withCloseButton = false,
		withTrigger = true,
		closeButtonEvent = () => {},
		...restProps
	}: Props = $props();
</script>

<Dialog.Root bind:open {...restProps}>
	{#if withTrigger}
		<Dialog.Trigger class="bg-zinc-800 text-zinc-200 px-4 py-1 rounded-lg">
			{buttonText}
		</Dialog.Trigger>
	{/if}
	<Dialog.Portal>
		<Dialog.Overlay class="bg-zinc-800/40 fixed inset-0 z-50" />
		<Dialog.Content
			{...contentProps}
			class="outline-hidden fixed left-1/2 top-1/2 z-50 w-full max-w-xl rounded-md translate-x-[-50%] translate-y-[-50%] flex flex-col border border-zinc-200 bg-white p-4"
		>
			<Dialog.Title class="font-medium text-xl">
				{@render title()}
			</Dialog.Title>
			{@render children?.()}
			{#if withCloseButton}
				<Dialog.Close class="py-1 px-4 mt-2 rounded-lg bg-zinc-100" onclick={closeButtonEvent}
					>Cancel</Dialog.Close
				>
			{/if}
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
