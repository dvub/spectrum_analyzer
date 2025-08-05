'use client';

import { Message } from '@/bindings/Message';
import { usePluginListener } from '@/hooks/usePluginListener';

import { useCallback, useRef } from 'react';
import { Canvas } from './Canvas';

const FPS = 30;
export function Spectrum() {
	const coordinatesToDraw = useRef<[number, number][]>([]);
	const listener = useCallback((m: Message) => {
		if (m.type !== 'drawData') {
			return;
		}
		if (m.data.type !== 'spectrum') {
			return;
		}
		const spectrumData = m.data.data;
		coordinatesToDraw.current = spectrumData;
	}, []);
	usePluginListener(listener);

	function draw(ctx: CanvasRenderingContext2D) {
		const m: Message = {
			type: 'drawRequest',
			data: {
				type: 'spectrum',
			},
		};
		window.plugin.send(JSON.stringify(m));

		ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
		ctx.strokeStyle = 'black';
		ctx.lineWidth = 1;
		const spectrumArray = coordinatesToDraw.current;

		ctx.beginPath();
		for (let i = 0; i < spectrumArray.length; i++) {
			const [x, y] = spectrumArray[i];

			ctx.lineTo(
				Math.floor(x * ctx.canvas.width),
				Math.floor((1.0 - y) * ctx.canvas.height)
			);
		}
		ctx.stroke();
	}

	return (
		<div>
			<Canvas draw={draw} fps={FPS} />
		</div>
	);
}
