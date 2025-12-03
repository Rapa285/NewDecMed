// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info

import { tryCatchAsVal } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import type { LayoutLoad } from './$types';
import type { Role, SuccessResponse } from '$lib/types';

export const prerender = true;
export const ssr = false;

export type LayoutLoadData = {
	redirect_to: string | null;
	role: Role | null;
};

export const load: LayoutLoad = async ({ url }) => {
	const resInvokeAuthState = await tryCatchAsVal(async () => {
		return (await invoke('auth_status')) as SuccessResponse<Role | null>;
	});

	const defaultData: LayoutLoadData = {
		redirect_to: null,
		role: null
	};

	console.log(resInvokeAuthState);

	if (!resInvokeAuthState.success) {
		const redirect_code = resInvokeAuthState.error.match(/\$<(\d+)>\$/);
		if (!redirect_code) {
			defaultData.redirect_to = '/activation';
		} else {
			switch (parseInt(redirect_code[1])) {
				case 0: {
					defaultData.redirect_to = '/activation';
					break;
				}
				case 1: {
					defaultData.redirect_to = '/signup';
					break;
				}
				case 2: {
					defaultData.redirect_to = '/signin';
					break;
				}
				case 3: {
					defaultData.redirect_to = '/complete-profile';
					break;
				}
				default: {
					defaultData.redirect_to = '/activation';
					break;
				}
			}
		}
	}

	if (!defaultData.redirect_to && !url.pathname.startsWith('/dashboard')) {
		defaultData.redirect_to = '/dashboard';
	}

	if (resInvokeAuthState.success) {
		defaultData.role = resInvokeAuthState.data.data;
	}

	console.log(defaultData);

	return defaultData;
};
