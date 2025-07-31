'use client';
import { usePluginListener } from '@/hooks/usePluginListener';

export default function Home() {
	usePluginListener((message) => {
		console.log(message);
	});

	return <div>hi</div>;
}
