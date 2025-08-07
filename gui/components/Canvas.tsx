// https://medium.com/@pdx.lucasm/canvas-with-react-js-32e133c05258

import { useEffect, useRef } from 'react';

export function Canvas(props: {
	fps: number;
	draw: (ctx: CanvasRenderingContext2D) => void;
	width?: number;
	height?: number;
	className: string | undefined;
}) {
	console.log('rerendered canvas');

	const { draw, fps } = props;

	const canvasRef = useRef<HTMLCanvasElement | null>(null);

	useEffect(() => {
		let animationFrameId = 0;
		const canvas = canvasRef.current!;
		const ctx = canvas.getContext('2d')!;

		canvas.width = canvas.offsetWidth;
		canvas.height = canvas.offsetHeight;

		const interval = 1000 / fps;

		let now;
		let then = Date.now();

		let delta;

		function renderWithFps() {
			animationFrameId = requestAnimationFrame(renderWithFps);

			now = Date.now();
			delta = now - then;

			if (delta > interval) {
				then = now - (delta % interval);
				draw(ctx);
			}
		}
		renderWithFps();

		return () => {
			cancelAnimationFrame(animationFrameId);
			console.log('END');
		};
	}, [draw, fps]);
	return (
		<canvas
			ref={canvasRef}
			width={props.width}
			height={props.height}
			// TODO: ! refactor
			className={props.className}
		/>
	);
}
