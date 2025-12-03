<script lang="ts">
	import { Tabs } from 'bits-ui';
	import { ChevronRight, Loader2 } from '@lucide/svelte';
	import { AdministrativeHomeState } from './administrative-state.svelte';

	const administrativeHomeState = new AdministrativeHomeState();
</script>

<Tabs.Root bind:value={administrativeHomeState.currentTab} class="w-full">
	<Tabs.List class="w-full flex justify-center mb-4">
		<div
			class="flex items-center max-w-max justify-center gap-2 p-2 rounded-md bg-white border border-zinc-200"
		>
			<Tabs.Trigger
				value={administrativeHomeState.tabs[0]}
				class="data-[state=active]:bg-zinc-100 hover:bg-zinc-100 cursor-pointer px-3 py-1 rounded-md"
				>Read</Tabs.Trigger
			>
		</div>
	</Tabs.List>
	<Tabs.Content value={administrativeHomeState.tabs[0]}>
		<div class="bg-white border border-zinc-200 rounded-md">
			{#await administrativeHomeState.get_read_access()}
				<div class="p-4">
					<div
						class="animate-pulse bg-zinc-100 w-full shadow h-20 flex items-center justify-center rounded-md"
					>
						<Loader2 class="animate-spin" />
					</div>
				</div>
			{:then readAccess}
				{#if readAccess && readAccess.length > 0}
					{#each readAccess as access, i (i)}
						<a
							href={`/dashboard/adm/${access.patientIotaAddress}?accessToken=${access.accessToken}`}
							class="p-2 [&:not(:last-child)]:border-b border-zinc-200 flex items-center gap-2"
						>
							<div
								class="size-8 rounded-full flex items-center justify-center bg-zinc-50 border border-zinc-200 shrink-0"
							>
								<p class="text-xs font-medium">{i + 1}</p>
							</div>
							<p class="flex-1 flex">{access.patientName}</p>
							<span class="flex items-center justify-center">
								<ChevronRight />
							</span>
						</a>
					{/each}
				{:else}
					<div class="p-2">
						<p>No access found.</p>
					</div>
				{/if}
			{/await}
		</div>
	</Tabs.Content>
</Tabs.Root>
