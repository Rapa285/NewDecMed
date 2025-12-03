import type { SuccessResponse, TauriAccessData } from '$lib/types';
import { tryCatchAsVal } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';

export class MedicalHomeState {
	tabs = ['read', 'update'];
	currentTab = $state(this.tabs[0]);

	constructor() {}

	get_read_access = async () => {
		const resInvokeGetReadAccess = await tryCatchAsVal(async () => {
			return (await invoke('get_read_access_medical_personnel')) as SuccessResponse<
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

	get_update_access = async () => {
		const resInvokeGetUpdateAccess = await tryCatchAsVal(async () => {
			return (await invoke('get_update_access_medical_personnel')) as SuccessResponse<
				TauriAccessData[]
			>;
		});

		console.log('update-access', resInvokeGetUpdateAccess);

		if (!resInvokeGetUpdateAccess.success) {
			toast.error(resInvokeGetUpdateAccess.error);

			return [];
		}

		return resInvokeGetUpdateAccess.data.data;
	};
}
