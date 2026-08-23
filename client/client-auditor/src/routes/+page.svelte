<script lang="ts">
	import { onMount } from 'svelte';
	import { Loader2, RefreshCcw, Settings, X } from '@lucide/svelte';
	import { AuditorHomeState } from './state.svelte';
	import { cn, formatTimestamp, truncateMiddle } from '$lib/utils';

	const state = new AuditorHomeState();

	onMount(async () => {
		await state.loadSettings();
		await state.loadLogs();
	});
</script>

<div class="flex items-center justify-between border-b border-zinc-200 pb-2">
	<div>
		<h2 class="font-medium">Audit Trail — Log Metadata</h2>
		<p class="text-sm text-zinc-400">On-chain log batches, verifiable against IPFS.</p>
	</div>
	<div class="flex items-center gap-2">
		{#if state.isLoading}
			<span class="pill pill-loading">loading…</span>
		{:else if state.loadError}
			<span class="pill pill-error">error</span>
		{:else}
			<span class="pill">{state.rows.length} loaded</span>
		{/if}
		<button class="btn-cancel" onclick={state.openSettings} aria-label="Settings">
			<Settings size={16} />
		</button>
		<button class="btn" onclick={() => state.loadLogs()} disabled={state.isLoading}>
			<RefreshCcw size={16} class={cn(state.isLoading && 'animate-spin')} />
			Refresh
		</button>
	</div>
</div>

{#if state.loadError}
	<div class="my-3 rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-500">
		{state.loadError}
	</div>
{/if}

<div class="my-3 flex-1 overflow-auto border-light rounded-md">
	<table>
		<thead>
			<tr>
				<th>Seq</th>
				<th>Object ID</th>
				<th>Rotation Timestamp</th>
				<th>Records</th>
				<th>IPFS CID</th>
				<th>File Hash</th>
				<th>Prev TX Digest</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			{#if state.rows.length === 0 && !state.isLoading}
				<tr>
					<td colspan="8" class="py-8 text-center text-zinc-400">
						No log batches found yet.
					</td>
				</tr>
			{/if}
			{#each state.rows as record (record.object_id)}
				<tr>
					<td>{record.metadata.log_sequence_number}</td>
					<td class="mono" title={record.object_id}>{truncateMiddle(record.object_id, 18)}</td>
					<td>{formatTimestamp(record.metadata.rotation_timestamp)}</td>
					<td>{record.metadata.record_count}</td>
					<td class="mono" title={record.metadata.ipfs_cid}
						>{truncateMiddle(record.metadata.ipfs_cid, 18)}</td
					>
					<td class="mono" title={record.metadata.file_hash}
						>{truncateMiddle(record.metadata.file_hash, 16)}</td
					>
					<td class="mono" title={record.metadata.prev_tx_digest ?? ''}
						>{truncateMiddle(record.metadata.prev_tx_digest, 12)}</td
					>
					<td>
						<button class="btn-cancel" onclick={() => state.openEntries(record)}>Inspect</button>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

{#if state.hasNextPage}
	<div class="flex justify-center pb-2">
		<button
			class="btn-cancel"
			onclick={() => state.loadLogs({ append: true })}
			disabled={state.isLoadingMore}
		>
			{#if state.isLoadingMore}
				<Loader2 size={16} class="animate-spin" />
			{/if}
			Load more
		</button>
	</div>
{/if}

{#if state.isSettingsOpen}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-zinc-800/40">
		<div class="flex w-full max-w-md flex-col rounded-md border border-zinc-200 bg-white p-4">
			<div class="mb-3 flex items-center justify-between">
				<h3 class="font-medium">Settings</h3>
				<button class="btn-cancel" onclick={() => (state.isSettingsOpen = false)}>
					<X size={16} />
				</button>
			</div>
			<div class="container-input-text mb-3">
				<label for="audit-trail-base-url">Audit-trail base URL</label>
				<input
					id="audit-trail-base-url"
					type="text"
					class="input-base"
					placeholder="http://localhost:3000"
					bind:value={state.settings.audit_trail_base_url}
				/>
			</div>
			<div class="container-input-text mb-4">
				<label for="ipfs-gateway-base-url">IPFS gateway base URL</label>
				<input
					id="ipfs-gateway-base-url"
					type="text"
					class="input-base"
					placeholder="http://103.107.4.68:8080"
					bind:value={state.settings.ipfs_gateway_base_url}
				/>
			</div>
			<div class="flex items-center justify-end gap-2">
				<button class="btn-cancel" onclick={() => (state.isSettingsOpen = false)}>Cancel</button>
				<button
					class="btn"
					onclick={() => state.saveSettings(state.settings)}
					disabled={state.isSavingSettings}
				>
					{#if state.isSavingSettings}
						<Loader2 size={16} class="animate-spin" />
					{/if}
					Save
				</button>
			</div>
		</div>
	</div>
{/if}

{#if state.selectedRecord}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-zinc-800/40">
		<div
			class="flex max-h-[80vh] w-full max-w-2xl flex-col rounded-md border border-zinc-200 bg-white p-4"
		>
			<div class="mb-3 flex items-center justify-between">
				<div>
					<h3 class="font-medium">Log entries</h3>
					<p class="mono text-zinc-400">{state.selectedRecord.metadata.ipfs_cid}</p>
				</div>
				<button class="btn-cancel" onclick={state.closeEntries}>
					<X size={16} />
				</button>
			</div>
			<div class="flex-1 overflow-auto">
				{#if state.isLoadingEntries}
					<div class="flex items-center justify-center gap-2 py-8 text-zinc-400">
						<Loader2 size={16} class="animate-spin" />
						Fetching from IPFS…
					</div>
				{:else if state.entriesError}
					<div class="rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-500">
						{state.entriesError}
					</div>
				{:else if state.entries.length === 0}
					<p class="py-8 text-center text-zinc-400">No entries in this batch.</p>
				{:else}
					<div class="flex flex-col gap-2">
						{#each state.entries as entry (entry.record_hash)}
							<div class="border-light rounded-md p-2">
								<div class="mb-1 flex items-center justify-between">
									<span class="mono">{entry.record_id}</span>
									<span class="text-xs text-zinc-400">{formatTimestamp(entry.timestamp)}</span>
								</div>
								<pre class="mono overflow-x-auto whitespace-pre-wrap">{JSON.stringify(
										entry,
										null,
										2
									)}</pre>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
