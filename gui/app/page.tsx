'use client';

import { Spectrum } from '@/components/Spectrum';
export default function Home() {
	return (
		<div className='w-screen h-screen'>
			<Spectrum
				width={10}
				fill={false}
				antiAliasing={true}
				style='rgb(0,0,0)'
				fps={50}
				className='w-full h-full'
			/>
		</div>
	);
}
