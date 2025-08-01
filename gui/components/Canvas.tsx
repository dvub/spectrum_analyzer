// https://medium.com/@pdx.lucasm/canvas-with-react-js-32e133c05258

import { useEffect, useRef } from 'react';

export function Canvas(props: {
	draw: (ctx: CanvasRenderingContext2D) => void;
}) {
	const { draw } = props;

	const canvasRef = useRef<HTMLCanvasElement | null>(null);

	useEffect(() => {
		let animationFrameId = 0;
		const canvas = canvasRef.current!;
		const ctx = canvas.getContext('2d')!;

		function render() {
			draw(ctx);
			animationFrameId = requestAnimationFrame(render);
		}
		render();

		return () => cancelAnimationFrame(animationFrameId);
	}, [draw]);
	return (
		<div>
			<canvas ref={canvasRef} width={600} height={600} />
		</div>
	);
}
