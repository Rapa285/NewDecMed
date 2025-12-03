import {
	ADMIN_ROLE,
	ADMINISTRATIVE_PERSONNEL_ROLE,
	AUTH_CONTEXT_DEFAULT_KEY,
	MEDICAL_PERSONNEL_ROLE
} from '$lib/constants';
import type { AuthContext } from '$lib/interfaces';
import type { NavLink, Role } from '$lib/types';
import { invoke } from '@tauri-apps/api/core';
import { getContext, setContext } from 'svelte';

class Auth implements AuthContext {
	role = $state<Role | null>(null);
	isRegistered = $state<boolean | undefined>(undefined);
	constructor() {}

	getRegisterStatus = async () => {
		this.isRegistered = (await invoke('is_signed_up')) as boolean;
	};
	getNav = () => {
		const defaultNav: NavLink[] = [
			{
				label: 'Home',
				link: `/dashboard`,
				pageTitle: 'Home'
			},
			{
				label: 'Profile',
				link: `/dashboard/profile`,
				pageTitle: 'Profile'
			}
		];

		switch (this.role) {
			case ADMIN_ROLE: {
				return [...defaultNav];
			}
			case MEDICAL_PERSONNEL_ROLE: {
				return [...defaultNav];
			}
			case ADMINISTRATIVE_PERSONNEL_ROLE: {
				return [...defaultNav];
			}
			default: {
				return defaultNav;
			}
		}
	};
}

export const getAuthContext = (key = AUTH_CONTEXT_DEFAULT_KEY) => {
	return getContext<Auth>(key);
};

export const createAuthContext = (key = AUTH_CONTEXT_DEFAULT_KEY) => {
	const authContext = new Auth();
	return setContext(key, authContext);
};
