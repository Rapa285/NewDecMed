import { SIGNIN_TOTAL_STEP } from '$lib/constants';
import { signInSchemas } from '$lib/schema';
import type { SignInSchemaStep3, SuccessResponse } from '$lib/types';
import { invoke } from '@tauri-apps/api/core';
import { superForm, type Infer, type SuperForm, type SuperValidated } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { getAuthContext } from '../../(context)/auth-context.svelte';
import { tryCatchAsVal } from '$lib/utils';
import { toast } from 'svelte-sonner';

type Constructor = {
	signInForm: SuperValidated<Infer<SignInSchemaStep3>>;
};

export class SignInState {
	currentStep = $state<number>(1);
	signInFormMeta: SuperForm<Infer<SignInSchemaStep3>>;
	authContext = getAuthContext();

	constructor({ signInForm }: Constructor) {
		$effect(() => {
			this.signInFormMeta.options.validators = zod(signInSchemas[this.currentStep - 1]);
		});

		this.signInFormMeta = superForm(signInForm, {
			validators: false,
			dataType: 'json',
			SPA: true,
			onSubmit: async ({ cancel, formData }) => {
				if (this.currentStep === SIGNIN_TOTAL_STEP) return;
				cancel();

				const result = await this.signInFormMeta.validateForm({ update: true });
				let valid = true;

				if (result.valid) {
					switch (this.currentStep) {
						case 1: {
							const pin = formData.get('pin') as string;
							const resInvokeValidatePin = await tryCatchAsVal(async () => {
								return (await invoke('validate_pin', {
									pin,
									authType: 'Signin'
								})) as SuccessResponse<null>;
							});

							valid = resInvokeValidatePin.success;

							if (!resInvokeValidatePin.success) {
								this.signInFormMeta.errors.update((val) => {
									val.pin = [resInvokeValidatePin.error];
									return val;
								});
							}

							break;
						}
						case 2: {
							const confirmPin = formData.get('confirmPin') as string;
							const resInvokeValidateConfirmPin = await tryCatchAsVal(async () => {
								return (await invoke('validate_confirm_pin', {
									confirmPin,
									authType: 'Signin'
								})) as SuccessResponse<null>;
							});

							valid = resInvokeValidateConfirmPin.success;

							if (!resInvokeValidateConfirmPin.success) {
								this.signInFormMeta.errors.update((val) => {
									val.confirmPin = [resInvokeValidateConfirmPin.error];
									return val;
								});
							}

							break;
						}
					}
				} else {
					valid = false;
				}

				if (valid) this.currentStep += 1;
			},
			onUpdate: async ({ form, result, cancel }) => {
				if (result.type === 'success') {
					const resInvokeSignin = await tryCatchAsVal(async () => {
						return (await invoke('signin', {
							seedWords: form.data.seedWords
						})) as SuccessResponse<null>;
					});

					if (!resInvokeSignin.success) {
						cancel();
						toast.error(resInvokeSignin.error);
					}
				}
			}
		});
	}
}
