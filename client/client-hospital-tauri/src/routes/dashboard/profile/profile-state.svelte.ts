import type { GetProfileData, SuccessResponse } from '$lib/types';
import { tryCatchAsVal } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';
import { getAuthContext } from '../../(context)/auth-context.svelte';
import { invalidateAll } from '$app/navigation';
import { BaseDirectory, create, exists, mkdir } from '@tauri-apps/plugin-fs';
import QRCode from 'qrcode';

export class ProfileState {
	authContext = getAuthContext();
	profile = $state<GetProfileData>();
	qr = $state('');
	qrFileName = $derived(`qr-${this.profile?.id ?? 'unknown'}`);

	signout = async () => {
		const resInvokeSignout = await tryCatchAsVal(async () => {
			return (await invoke('signout')) as SuccessResponse<null>;
		});

		if (!resInvokeSignout.success) {
			toast.error('Signout failed');
			return;
		}

		this.authContext.role = null;
		invalidateAll();
	};

	getProfile = async () => {
		const resInvokeGetProfile = await tryCatchAsVal(async () => {
			return (await invoke('get_profile')) as SuccessResponse<GetProfileData>;
		});

		if (!resInvokeGetProfile.success) {
			toast.error(resInvokeGetProfile.error);
			throw new Error(resInvokeGetProfile.error);
		}

		this.profile = resInvokeGetProfile.data.data;

		QRCode.toString(
			`${this.profile.iotaAddress ?? ''}@${this.profile.prePublicKey ?? ''}`,
			{
				type: 'svg'
			},
			(err, string) => {
				if (err) throw err;
				this.qr = string;
			}
		);

		return resInvokeGetProfile.data.data;
	};

	downloadQr = async () => {
		if (
			!(await exists('decmed-hospital/', {
				baseDir: BaseDirectory.Home
			}))
		) {
			await mkdir('decmed-hospital', {
				baseDir: BaseDirectory.Home
			});
		}

		const res = await QRCode.toDataURL(
			`${this.profile?.iotaAddress ?? ''}@${this.profile?.prePublicKey ?? ''}`,
			{
				type: 'image/png'
			}
		);
		const response = await fetch(res);
		const arrayBuffer = await response.arrayBuffer();
		const buff = new Uint8Array(arrayBuffer);

		const file = await create(`decmed-hospital/${this.qrFileName}.png`, {
			baseDir: BaseDirectory.Home
		});
		await file.write(buff);
		await file.close();

		toast.success(`QR Code downloaded to ~/decmed-hospital/${this.qrFileName}.png`);
	};
}
