'use client';

import { Message } from '@/bindings/Message';
import { usePluginListener } from '@/hooks/usePluginListener';

import { useCallback, useRef } from 'react';
import { Canvas } from './Canvas';

export function Spectrum() {
	console.log('This component rerendered');

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
		ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);

		ctx.strokeStyle = 'black';
		ctx.lineWidth = 1;
		const spectruma = spectrum.current;



		ctx.beginPath();
		for (let i = 0; i < spectruma.length; i++) {
			const current = spectruma[i];
			const x = (i / spectruma.length) * ctx.canvas.width;
			const y = current * 10000

			ctx.lineTo(x, y);
		}
		ctx.stroke();
	}

	return (
		<div>
			<Canvas draw={draw} />
		</div>
	);
}
