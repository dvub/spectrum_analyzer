'use client';

import { Message } from '@/bindings/Message';
import { usePluginListener } from '@/hooks/usePluginListener';

import { useCallback, useEffect, useRef } from 'react';
import { Canvas } from './Canvas';

export function Spectrum(props: {
	fps: number;
	width?: number;
	height?: number;
	fill: boolean;
	antiAliasing: boolean;
	style: string | CanvasGradient | CanvasPattern;
	className?: string;
}) {
	const { fill, antiAliasing, style, width, height, fps } = props;

	useEffect(() => {
		const initMessage: Message = {
			type: 'spectrumAnalyzerConfigUpdate',
			data: {
				type: 'fps',
				data: fps,
			},
		};
		window.plugin.send(JSON.stringify(initMessage));
	}, [fps]);

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
		const requestMessage: Message = {
			type: 'drawRequest',
			data: {
				type: 'spectrum',
			},
		};
		window.plugin.send(JSON.stringify(requestMessage));

		const height = ctx.canvas.height;
		const width = ctx.canvas.width;

		ctx.strokeStyle = style;
		ctx.fillStyle = style;
		ctx.clearRect(0, 0, width, height);
		ctx.lineWidth = 1;

		//
		const spectrumArray = coordinatesToDraw.current;

		ctx.beginPath();
		for (let i = 0; i < spectrumArray.length; i++) {
			const [x, y] = spectrumArray[i];

			let scaledX = x * width;
			let scaledY = (1.0 - y) * height;

			if (!antiAliasing) {
				scaledX = Math.floor(scaledX);
				scaledY = Math.floor(scaledY);
			}
			ctx.lineTo(scaledX, scaledY);
		}

		if (fill) {
			ctx.lineTo(width, height);
			ctx.lineTo(0, height);
			ctx.fill();
		}

		ctx.stroke();
	}

	return (
		<Canvas
			draw={draw}
			fps={fps}
			width={width}
			height={height}
			className={props.className}
		/>
	);
}
