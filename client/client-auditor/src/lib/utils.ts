import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import type { TryCatchAsValReturn } from './types';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

export async function copyToClipboard(str: string) {
	navigator.clipboard.writeText(str);
}

export async function tryCatchAsVal<T>(func: () => Promise<T>): Promise<TryCatchAsValReturn<T>> {
	try {
		const result = await func();
		return { success: true, data: result };
	} catch (e) {
		return { success: false, error: e as string };
	}
}

export function formatTimestamp(isoString: string) {
	try {
		return new Date(isoString).toLocaleString();
	} catch {
		return isoString;
	}
}

export function truncateMiddle(value: string | null | undefined, length = 14) {
	if (!value) return '—';
	if (value.length <= length) return value;
	const head = Math.ceil(length / 2);
	const tail = Math.floor(length / 2);
	return `${value.slice(0, head)}…${value.slice(value.length - tail)}`;
}
