// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)

import type { SuccessResponse } from '$lib/types';
import { tryCatchAsVal } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import type { LayoutLoad } from './$types';

// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;

export type LayoutLoadData = {
	redirect_to: string | null;
};

export const load: LayoutLoad = async ({ url }) => {
	const resInvokeAuthState = await tryCatchAsVal(async () => {
		return (await invoke('auth_status')) as SuccessResponse<null>;
	});

	const defaultData: LayoutLoadData = {
		redirect_to: null
	};

	console.log(resInvokeAuthState);

	if (!resInvokeAuthState.success) {
		defaultData.redirect_to = '/signin';
		if (url.pathname === '/signup') {
			defaultData.redirect_to = '/signup';
		}

		const redirect_code = resInvokeAuthState.error.match(/\$<(\d+)>\$/);

		if (redirect_code && parseInt(redirect_code[1]) === 1) {
			defaultData.redirect_to = '/complete-profile';
		}
	}

	if (!defaultData.redirect_to && !url.pathname.startsWith('/dashboard')) {
		defaultData.redirect_to = '/dashboard';
	}

	console.log(defaultData);

	return defaultData;
};
