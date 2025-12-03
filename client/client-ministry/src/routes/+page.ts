import { addHospitalSchema } from '$lib/schema';
import { tryCatchAsVal } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import type { PageLoad } from './$types';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { toast } from 'svelte-sonner';
import { type InvokeGetHospitalsResponseData, type SuccessResponse } from '$lib/types';

export const load: PageLoad = async () => {
	const addHospitalForm = await superValidate(zod(addHospitalSchema));

	const getHospitals = async () => {
		const resInvokeGetHospitals = await tryCatchAsVal(async () => {
			return (await invoke('get_hospitals', {
				payload: {
					cursor: null,
					size: null
				}
			})) as SuccessResponse<InvokeGetHospitalsResponseData[]>;
		});

		if (!resInvokeGetHospitals.success) {
			toast.error(resInvokeGetHospitals.error);
			throw 'No hospital registered';
		}

		if (resInvokeGetHospitals.data.data.length === 0) {
			throw 'No hospital registered';
		}

		return resInvokeGetHospitals.data.data;
	};

	return {
		addHospitalForm,
		hospitals: getHospitals()
	};
};
