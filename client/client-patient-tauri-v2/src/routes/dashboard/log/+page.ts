import { invoke } from '@tauri-apps/api/core';
import type { PageLoad } from './$types';
import type { InvokeGetAccessLog, SuccessResponse } from '$lib/types';

export const load: PageLoad = async () => {
	const accessLog = invoke('get_access_log') as Promise<SuccessResponse<InvokeGetAccessLog[]>>;

	return {
		accessLog
	};
};
