import { Spectrum } from '@/components/Spectrum';

export default function Home() {
	return (
		<Spectrum
			width={400}
			height={400}
			fill={true}
			antiAliasing={false}
			style='rgb(0,255,0)'
			fps={30}
			className='border-2 border-blue-500 w-full h-[30%]'
		/>
	);
}
