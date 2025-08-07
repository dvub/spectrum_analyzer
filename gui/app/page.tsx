'use client';

import { Spectrum } from '@/components/Spectrum';
import { useEffect } from 'react';

export default function Home() {
	useEffect(() => {
		console.log(window.innerWidth);
	}, []);

	return (
		<div className='w-full h-full'>
			<Spectrum
				width={10}
				fill={false}
				antiAliasing={true}
				style='rgb(0,0,0)'
				fps={50}
				className='border-2 border-blue-500 w-full h-[50%]'
			/>
		</div>
	);
}
