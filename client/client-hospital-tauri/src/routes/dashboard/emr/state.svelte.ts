import { getContext, setContext } from 'svelte';

export class EmrState {
	accessToken = $state<string | null>(null);
	patientPrePublicKey = $state<string | null>(null);
}

export const setEmrState = (emrState: EmrState) => {
	setContext('$EMR_STATE', emrState);
};
export const getEmrState = (): EmrState => {
	return getContext('$EMR_STATE');
};
