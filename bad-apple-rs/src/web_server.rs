use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Html,
    routing::get,
    Router,
};
use tokio::sync::broadcast;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use webbrowser;

pub async fn start_server(port: u16, tx: broadcast::Sender<String>) {
    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(move |ws| ws_handler(ws, tx)))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🚀 Web Streamer starting at http://{}", addr);
    
    // Open browser in a separate thread so it doesn't block the server loop
    let url = format!("http://{}", addr);
    tokio::task::spawn_blocking(move || {
        let _ = webbrowser::open(&url);
    });

    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(r##"
<!DOCTYPE html>
<html>
<head>
    <title>Bad Apple!! Real-time 4K Stream</title>
    <style>
        body { 
            background: #000; 
            color: #0f0; 
            font-family: 'Courier New', Courier, monospace; 
            display: flex; 
            flex-direction: column; 
            align-items: center; 
            justify-content: center; 
            height: 100vh; 
            margin: 0; 
            overflow: hidden; 
        }
        #canvas-container {
            position: relative;
            box-shadow: 0 0 30px rgba(0, 255, 0, 0.3);
            border: 2px solid #111;
        }
        canvas {
            display: block;
            image-rendering: pixelated;
        }
        .controls {
            position: fixed;
            bottom: 20px;
            left: 50%;
            transform: translateX(-50%);
            display: flex;
            gap: 15px;
            background: rgba(0, 20, 0, 0.8);
            padding: 10px 20px;
            border-radius: 5px;
            border: 1px solid #0f0;
            z-index: 100;
        }
        button, select {
            background: black;
            color: #0f0;
            border: 1px solid #0f0;
            padding: 5px 10px;
            cursor: pointer;
            font-family: inherit;
        }
        button:hover {
            background: #0f0;
            color: black;
        }
        .status {
            position: fixed;
            top: 20px;
            right: 20px;
            font-size: 14px;
            text-shadow: 0 0 5px #0f0;
        }
        /* Scanline Overlay */
        .scanlines {
            position: absolute;
            top: 0; left: 0; right: 0; bottom: 0;
            background: linear-gradient(rgba(18, 16, 16, 0) 50%, rgba(0, 0, 0, 0.25) 50%),
                        linear-gradient(90deg, rgba(255, 0, 0, 0.06), rgba(0, 255, 0, 0.02), rgba(0, 0, 255, 0.06));
            background-size: 100% 4px, 3px 100%;
            pointer-events: none;
            mix-blend-mode: overlay;
        }
    </style>
</head>
<body>
    <div class="status" id="status">Connecting to Rust Backend...</div>
    
    <div id="canvas-container">
        <canvas id="bad-apple-canvas"></canvas>
        <div id="scanline-overlay" class="scanlines"></div>
    </div>

    <div class="controls">
        <button id="toggle-emoji">Emoji Mode: OFF</button>
        <select id="color-filter">
            <option value="#0f0">Classic Green</option>
            <option value="#ffb000">Amber (VGA)</option>
            <option value="#00ffff">Cyan (Night)</option>
            <option value="#ffffff">True White</option>
        </select>
        <button id="toggle-scanlines">Toggle Scanlines</button>
    </div>

    <script>
        const canvas = document.getElementById('bad-apple-canvas');
        const ctx = canvas.getContext('2d');
        const status = document.getElementById('status');
        const overlay = document.getElementById('scanline-overlay');
        
        let emojiMode = false;
        let colorFilter = "#0f0";
        
        const ws = new WebSocket(`ws://${location.host}/ws`);

        document.getElementById('toggle-emoji').onclick = (e) => {
            emojiMode = !emojiMode;
            e.target.innerText = `Emoji Mode: ${emojiMode ? 'ON' : 'OFF'}`;
        };

        document.getElementById('color-filter').onchange = (e) => {
            colorFilter = e.target.value;
            status.style.color = colorFilter;
        };

        document.getElementById('toggle-scanlines').onclick = () => {
            overlay.style.display = overlay.style.display === 'none' ? 'block' : 'none';
        };

        ws.onopen = () => {
            status.innerText = "● LIVE STREAM CONNECTED";
            status.style.color = colorFilter;
        };

        ws.onmessage = (event) => {
            const lines = event.data.split('\n');
            if (lines.length < 2) return;

            const charWidth = 8;
            const charHeight = 12;
            const width = lines[0].length;
            const height = lines.length;

            if (canvas.width !== width * charWidth) {
                canvas.width = width * charWidth;
                canvas.height = height * charHeight;
                canvas.style.width = (width * charWidth) + 'px';
                canvas.style.height = (height * charHeight) + 'px';
            }

            ctx.fillStyle = "black";
            ctx.fillRect(0, 0, canvas.width, canvas.height);
            
            ctx.font = "bold 12px 'Courier New', monospace";
            ctx.fillStyle = colorFilter;

            for (let y = 0; y < height; y++) {
                const line = lines[y];
                for (let x = 0; x < width; x++) {
                    const char = line[x];
                    if (!char || char === ' ') continue;

                    if (emojiMode) {
                        // Emoji Mode: Map ASCII to 🍎/🍏/⬛
                        const emoji = char === '#' ? '🍎' : (char === '^' ? '🍏' : '⬜');
                        ctx.fillText(emoji, x * charWidth, y * charHeight + 10);
                    } else {
                        // High Performance Text Rendering
                        ctx.fillText(char, x * charWidth, y * charHeight + 10);
                    }
                }
            }
            
            // Add Global Glow
            canvas.style.filter = `drop-shadow(0 0 5px ${colorFilter})`;
        };

        ws.onclose = () => {
            status.innerText = "○ STREAM DISCONNECTED";
            status.style.color = "#f00";
        };
    </script>
</body>
</html>
"##)
}

async fn ws_handler(ws: WebSocketUpgrade, tx: broadcast::Sender<String>) -> impl axum::response::IntoResponse {
    let rx = tx.subscribe();
    ws.on_upgrade(move |socket| handle_socket(socket, rx))
}

async fn handle_socket(mut socket: WebSocket, mut rx: broadcast::Receiver<String>) {
    while let Ok(frame) = rx.recv().await {
        if socket.send(Message::Text(frame.into())).await.is_err() {
            break;
        }
    }
}
