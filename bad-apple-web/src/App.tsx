import { useState, useEffect, useRef } from 'react';
import init, { AsciiEngine } from 'bad-apple-wasm';
import './index.css';

function App() {
  const [url, setUrl] = useState('https://www.youtube.com/watch?v=FtutLA63Cp8');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [streamUrl, setStreamUrl] = useState('');

  const [colorMode, setColorMode] = useState(false);
  const [bloom, setBloom] = useState(false);
  const [scanlines, setScanlines] = useState(false);
  const [noise, setNoise] = useState(false);

  const [engine, setEngine] = useState<AsciiEngine | null>(null);

  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const preRef = useRef<HTMLPreElement>(null);

  const [isPlaying, setIsPlaying] = useState(false);

  // Initialize WASM
  useEffect(() => {
    init().then(() => {
      console.log('WASM Initialized!');
      setEngine(new AsciiEngine());
    });
  }, []);

  const handleFetchStream = async () => {
    setLoading(true);
    setError('');
    setIsPlaying(false);

    try {
      // Use Vercel Serverless API in production, otherwise use full URL for local testing
      const apiUrl = import.meta.env.MODE === 'production'
        ? '/api/get-stream'
        : '/api/get-stream'; // Or 'http://localhost:3001/api/get-stream' if running separate Express server

      const res = await fetch(apiUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url })
      });

      const data = await res.json();
      if (!res.ok) throw new Error(data.error || 'Failed to fetch stream');

      setStreamUrl(data.streamUrl);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  // Video loop for processing frames
  useEffect(() => {
    let animationId: number;
    let lastTime = performance.now();
    let frameRateLimit = 1000 / 30; // Max 30 FPS processing for smooth UI

    const processFrame = () => {
      const now = performance.now();

      if (videoRef.current && canvasRef.current && engine && isPlaying && videoRef.current.readyState >= 2) {
        if (now - lastTime >= frameRateLimit) {
          const ctx = canvasRef.current.getContext('2d', { willReadFrequently: true });
          if (ctx) {
            const width = canvasRef.current.width;
            const height = canvasRef.current.height;

            // Draw video frame to hidden canvas
            ctx.drawImage(videoRef.current, 0, 0, width, height);

            // Extract Image Data
            const imageData = ctx.getImageData(0, 0, width, height);

            try {
              // Call WASM Rust Engine to render ASCII
              const asciiResult = engine.render_frame(
                new Uint8Array(imageData.data.buffer),
                width,
                height,
                colorMode,
                bloom,
                scanlines,
                noise
              );

              if (preRef.current) {
                // Apply HTML since we return ANSI escape codes for colors
                if (colorMode) {
                  const htmlRender = ansiToHtml(asciiResult);
                  preRef.current.innerHTML = htmlRender;
                } else {
                  preRef.current.textContent = asciiResult;
                }
              }
            } catch (e) {
              console.error('WASM error:', e);
            }
            lastTime = now;
          }
        }
      }
      animationId = requestAnimationFrame(processFrame);
    };

    animationId = requestAnimationFrame(processFrame);
    return () => cancelAnimationFrame(animationId);
  }, [isPlaying, engine, colorMode, bloom, scanlines, noise]);


  return (
    <div className="app-container">
      <div className="cyberpunk-dashboard">
        <h1 className="glitch-title" data-text="BAD APPLE WASM ENGINE">BAD APPLE WASM ENGINE</h1>

        <div className="control-panel">
          <input
            className="neon-input"
            type="text"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="Enter YouTube URL..."
          />
          <button className="neon-button primary" onClick={handleFetchStream} disabled={loading || !engine}>
            {loading ? 'HACKING STREAM...' : 'INITIALIZE UPLINK'}
          </button>
        </div>

        {error && <div className="error-box">[ERROR: {error}]</div>}

        <div className="toggles-grid">
          <label className="cyber-checkbox">
            <input type="checkbox" checked={colorMode} onChange={(e) => setColorMode(e.target.checked)} />
            <span className="box"></span> TEXTURE MAPPING (COLOR)
          </label>
          <label className="cyber-checkbox">
            <input type="checkbox" checked={bloom} onChange={(e) => setBloom(e.target.checked)} />
            <span className="box"></span> BLOOM EFFECT
          </label>
          <label className="cyber-checkbox">
            <input type="checkbox" checked={scanlines} onChange={(e) => setScanlines(e.target.checked)} />
            <span className="box"></span> CRT SCANLINES
          </label>
          <label className="cyber-checkbox">
            <input type="checkbox" checked={noise} onChange={(e) => setNoise(e.target.checked)} />
            <span className="box"></span> ANALOG NOISE
          </label>
        </div>

        <div className="viewport">
          <div className="viewport-overlay" style={{ display: scanlines ? 'block' : 'none' }}></div>

          <pre
            ref={preRef}
            className="ascii-display"
            style={{ color: colorMode ? 'white' : '#0f0', textShadow: bloom ? '0 0 8px currentColor' : 'none' }}
          >
            {streamUrl ? "STANDING BY FOR VISUAL DATA..." : "AWAITING URL INPUT..."}
          </pre>

          {/* Hidden Elements used for stream decoding */}
          {streamUrl && (
            <video
              ref={videoRef}
              src={streamUrl}
              crossOrigin="anonymous"
              loop
              controls
              onPlay={() => setIsPlaying(true)}
              onPause={() => setIsPlaying(false)}
              className="hidden-video"
              style={{ position: 'absolute', bottom: 10, left: 10, width: '200px', zIndex: 100, opacity: 0.8 }}
            />
          )}
          {/* Internal render resolution is 160x80 (Output is 160x40 ASCII chars due to half-block) */}
          <canvas ref={canvasRef} width={160} height={80} style={{ display: 'none' }} />
        </div>
      </div>
    </div>
  );
}

// Ultra-fast ANSI to HTML minimal parser for True Color mode
function ansiToHtml(ansiStr: string): string {
  let html = '';
  const regex = /\x1b\[([0-9;]+)m(.*?)(?=\x1b\[|$)/g;

  if (ansiStr.indexOf('\x1b[') === -1) return ansiStr;

  let match;

  html += ansiStr.substring(0, ansiStr.indexOf('\x1b['));

  while ((match = regex.exec(ansiStr)) !== null) {
    const code = match[1];
    const text = match[2];

    if (code === '0') {
      html += `</span>${text}`;
    } else {
      const parts = code.split(';');
      if (parts[0] === '38' && parts[1] === '2') {
        // Foreground color
        html += `<span style="color: rgb(${parts[2]},${parts[3]},${parts[4]})">${text}`;
      } else if (parts[0] === '48' && parts[1] === '2') {
        // Background color (very basic implementation for demonstration)
        html += `<span style="background-color: rgb(${parts[2]},${parts[3]},${parts[4]})">${text}`;
      }
    }
    // unused: lastIndex = regex.lastIndex;
  }

  return html;
}

export default App;
