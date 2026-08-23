export type LogMetadata = {
	version: string;
	log_sequence_number: number;
	rotation_timestamp: string;
	ipfs_cid: string;
	file_hash: string;
	first_record_hash: string;
	final_record_hash: string;
	record_count: number;
	prev_tx_digest: string | null;
};

export type LogRecord = {
	object_id: string;
	metadata: LogMetadata;
};

export type LogsResponse = {
	data: LogRecord[];
	next_cursor: string | null;
	has_next_page: boolean;
};

export type FetchLogsParams = {
	cursor: string | null;
	limit: number | null;
};

export type AuditLogEntry = {
	record_id: string;
	timestamp: string;
	prev_record_hash: string | null;
	record_hash: string;
	[key: string]: unknown;
};

export type AppSettings = {
	audit_trail_base_url: string;
	ipfs_gateway_base_url: string;
};

export type TryCatchAsValSuccess<T> = { success: true; data: T };
export type TryCatchAsValError = { success: false; error: string };
export type TryCatchAsValReturn<T> = TryCatchAsValSuccess<T> | TryCatchAsValError;
