'use client';

import { Message } from '@/bindings/Message';
import { usePluginListener } from '@/hooks/usePluginListener';

import { useCallback, useRef } from 'react';
import { Canvas } from './Canvas';

const FPS = 30;
export function Spectrum() {
	const spectrum = useRef<number[]>([]);
	const listener = useCallback((m: Message) => {
		if (m.type !== 'drawData') {
			return;
		}
		if (m.data.type !== 'spectrum') {
			return;
		}
		const spectrumData = m.data.data;
		spectrum.current = spectrumData;
	}, []);
	usePluginListener(listener);

	function draw(ctx: CanvasRenderingContext2D) {
		const m: Message = {
			type: 'drawRequest',
			data: {
				type: 'spectrum',
				data: FPS,
			},
		};
		window.plugin.send(JSON.stringify(m));

		ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
		ctx.strokeStyle = 'black';
		ctx.lineWidth = 1;
		const spectrumArray = spectrum.current;

		ctx.beginPath();
		for (let i = 0; i < spectrumArray.length; i++) {
			const current = spectrumArray[i];
			const x =
				(Math.log(i + 1) / Math.log(spectrumArray.length)) *
				ctx.canvas.width;
			const y = ctx.canvas.height - current * 1000;

			ctx.lineTo(x, y);
		}
		ctx.stroke();
	}

	return (
		<div>
			<Canvas draw={draw} fps={FPS} />
		</div>
	);
}
