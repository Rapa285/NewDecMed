import type { InvokeGetMedicalRecordResponseData, SuccessResponse } from '$lib/types';
import { tryCatchAsVal } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';

type Props = {
	accessToken: string;
	index: number;
	patientIotaAddress: string;
};

export class EmrReadState {
	accessToken = $state<string>('');
	index = $state<number>(0);
	patientIotaAddress = $state('');

	constructor({ accessToken, index, patientIotaAddress }: Props) {
		this.accessToken = accessToken;
		this.index = index;
		this.patientIotaAddress = patientIotaAddress;
	}

	getMedicalRecord = async (
		accessToken: string | null,
		index: number | null,
		patientIotaAddress: string
	) => {
		const resInvokeGetMedicalRecord = await tryCatchAsVal(async () => {
			return (await invoke('get_medical_record', {
				accessToken,
				index,
				patientIotaAddress
			})) as SuccessResponse<InvokeGetMedicalRecordResponseData>;
		});

		console.log(resInvokeGetMedicalRecord);

		if (!resInvokeGetMedicalRecord.success) {
			toast.error(resInvokeGetMedicalRecord.error);
			throw new Error(resInvokeGetMedicalRecord.error);
		}

		return resInvokeGetMedicalRecord.data.data;
	};

	fetchMedicalRecord = $derived(
		this.getMedicalRecord(this.accessToken, this.index, this.patientIotaAddress)
	);
}
