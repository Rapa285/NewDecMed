import { invoke } from '@tauri-apps/api/core';
import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import type { TryCatchAsValReturn } from './types';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

export async function copyToClipboard(str: string) {
	navigator.clipboard.writeText(str);
}

export async function reset() {
	await invoke('reset');
}

export async function tryCatchAsVal<T>(func: () => Promise<T>): Promise<TryCatchAsValReturn<T>> {
	try {
		const result = await func();
		return { success: true, data: result };
	} catch (e) {
		return { success: false, error: e as string };
	}
}

export async function waitMs(timeMs: number) {
	await new Promise((resolve) => setTimeout(resolve, timeMs));
}
