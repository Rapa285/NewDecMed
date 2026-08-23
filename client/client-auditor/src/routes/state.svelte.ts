import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';
import { tryCatchAsVal } from '$lib/utils';
import type { AppSettings, AuditLogEntry, LogRecord, LogsResponse } from '$lib/types';

export class AuditorHomeState {
	rows = $state<LogRecord[]>([]);
	nextCursor = $state<string | null>(null);
	hasNextPage = $state(false);
	isLoading = $state(false);
	isLoadingMore = $state(false);
	loadError = $state<string | null>(null);

	isSettingsOpen = $state(false);
	settings = $state<AppSettings>({ audit_trail_base_url: '', ipfs_gateway_base_url: '' });
	isSavingSettings = $state(false);

	selectedRecord = $state<LogRecord | null>(null);
	entries = $state<AuditLogEntry[]>([]);
	isLoadingEntries = $state(false);
	entriesError = $state<string | null>(null);

	loadSettings = async () => {
		const res = await tryCatchAsVal(async () => {
			return (await invoke('get_settings')) as AppSettings;
		});

		if (!res.success) {
			toast.error(res.error);
			return;
		}

		this.settings = res.data;
	};

	loadLogs = async ({ append = false }: { append?: boolean } = {}) => {
		if (append) {
			this.isLoadingMore = true;
		} else {
			this.isLoading = true;
		}
		this.loadError = null;

		const res = await tryCatchAsVal(async () => {
			return (await invoke('fetch_logs', {
				params: {
					cursor: append ? this.nextCursor : null,
					limit: 25
				}
			})) as LogsResponse;
		});

		if (!res.success) {
			this.loadError = res.error;
			toast.error(res.error);
		} else {
			this.rows = append ? [...this.rows, ...res.data.data] : res.data.data;
			this.nextCursor = res.data.next_cursor;
			this.hasNextPage = res.data.has_next_page;
		}

		this.isLoading = false;
		this.isLoadingMore = false;
	};

	openSettings = async () => {
		await this.loadSettings();
		this.isSettingsOpen = true;
	};

	saveSettings = async (settings: AppSettings) => {
		this.isSavingSettings = true;

		const res = await tryCatchAsVal(async () => {
			return await invoke('save_settings', { settings });
		});

		this.isSavingSettings = false;

		if (!res.success) {
			toast.error(res.error);
			return;
		}

		this.settings = settings;
		this.isSettingsOpen = false;
		toast.success('Settings saved');
		await this.loadLogs();
	};

	openEntries = async (record: LogRecord) => {
		this.selectedRecord = record;
		this.entries = [];
		this.entriesError = null;
		this.isLoadingEntries = true;

		const res = await tryCatchAsVal(async () => {
			return (await invoke('fetch_log_entries', {
				cid: record.metadata.ipfs_cid
			})) as AuditLogEntry[];
		});

		this.isLoadingEntries = false;

		if (!res.success) {
			this.entriesError = res.error;
			toast.error(res.error);
			return;
		}

		this.entries = res.data;
	};

	closeEntries = () => {
		this.selectedRecord = null;
		this.entries = [];
		this.entriesError = null;
	};
}
