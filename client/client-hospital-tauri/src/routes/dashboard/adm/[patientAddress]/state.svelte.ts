import type { InvokeGetPatientAdministrativeDataResponseData, SuccessResponse } from '$lib/types';
import { tryCatchAsVal } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';

type Props = {
	accessToken: string;
	patientIotaAddress: string;
};

export class AdmReadState {
	accessToken = $state<string>('');
	patientIotaAddress = $state('');

	constructor({ accessToken, patientIotaAddress }: Props) {
		this.accessToken = accessToken;
		this.patientIotaAddress = patientIotaAddress;
	}

	getPatientAdministrativeData = async (accessToken: string | null, patientIotaAddress: string) => {
		const resInvokeGetPatientAdministrativeData = await tryCatchAsVal(async () => {
			return (await invoke('get_administrative_data', {
				accessToken,
				patientIotaAddress
			})) as SuccessResponse<InvokeGetPatientAdministrativeDataResponseData>;
		});

		console.log(resInvokeGetPatientAdministrativeData);

		if (!resInvokeGetPatientAdministrativeData.success) {
			toast.error(resInvokeGetPatientAdministrativeData.error);
			throw new Error(resInvokeGetPatientAdministrativeData.error);
		}

		return resInvokeGetPatientAdministrativeData.data.data;
	};

	fetchPatientAdministrativeData = $derived(
		this.getPatientAdministrativeData(this.accessToken, this.patientIotaAddress)
	);
}
