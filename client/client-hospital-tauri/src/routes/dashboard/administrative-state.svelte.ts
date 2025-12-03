import type { SuccessResponse, TauriAccessData } from '$lib/types';
import { tryCatchAsVal } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';

export class AdministrativeHomeState {
	tabs = ['read'];
	currentTab = $state(this.tabs[0]);

	constructor() {}

	get_read_access = async () => {
		const resInvokeGetReadAccess = await tryCatchAsVal(async () => {
			return (await invoke('get_read_access_administrative_personnel')) as SuccessResponse<
				TauriAccessData[]
			>;
		});

		if (!resInvokeGetReadAccess.success) {
			toast.error(resInvokeGetReadAccess.error);
			return [];
		}

		console.log(resInvokeGetReadAccess.data.data);

		return resInvokeGetReadAccess.data.data;
	};
}
