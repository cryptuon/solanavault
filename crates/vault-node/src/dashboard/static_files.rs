//! # Static File Handler
//!
//! Serves embedded static files for the Vue.js SPA.

use axum::{
    body::Body,
    http::{header, Request, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use rust_embed::RustEmbed;

/// Embedded frontend assets
#[derive(RustEmbed)]
#[folder = "dashboard-frontend/dist"]
struct Assets;

/// Handle static file requests
pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Try to serve the requested file
    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(Body::from(content.data.into_owned()))
            .unwrap();
    }

    // Fall back to index.html for SPA routing
    match Assets::get("index.html") {
        Some(content) => {
            let html = String::from_utf8_lossy(&content.data).to_string();
            Html(html).into_response()
        }
        None => {
            // If no frontend is built yet, return a helpful message
            Html(get_placeholder_html()).into_response()
        }
    }
}

/// Placeholder HTML when frontend is not built
fn get_placeholder_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SolanaVault Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a2e;
            color: #eee;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }
        .container { max-width: 800px; text-align: center; }
        h1 { color: #00d9ff; margin-bottom: 20px; }
        p { color: #888; margin-bottom: 15px; line-height: 1.6; }
        .api-section {
            background: #16213e;
            border-radius: 8px;
            padding: 20px;
            margin: 20px 0;
            text-align: left;
        }
        .api-section h2 { color: #00d9ff; margin-bottom: 15px; font-size: 1.2em; }
        .endpoint {
            background: #0f3460;
            padding: 8px 12px;
            border-radius: 4px;
            margin: 8px 0;
            font-family: monospace;
        }
        .endpoint a { color: #00d9ff; text-decoration: none; }
        .endpoint a:hover { text-decoration: underline; }
        code {
            background: #0f3460;
            padding: 2px 6px;
            border-radius: 3px;
            font-size: 0.9em;
        }
        .build-instructions {
            background: #1e3a5f;
            border-left: 4px solid #00d9ff;
            padding: 15px;
            margin: 20px 0;
            text-align: left;
        }
        .build-instructions pre {
            background: #0f3460;
            padding: 10px;
            border-radius: 4px;
            overflow-x: auto;
            margin-top: 10px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>SolanaVault Node Dashboard</h1>
        <p>The web dashboard frontend has not been built yet.</p>

        <div class="api-section">
            <h2>Available API Endpoints</h2>
            <div class="endpoint"><a href="/api/health">/api/health</a> - Health check</div>
            <div class="endpoint"><a href="/api/stats">/api/stats</a> - All node statistics</div>
            <div class="endpoint"><a href="/api/storage">/api/storage</a> - Storage metrics</div>
            <div class="endpoint"><a href="/api/network">/api/network</a> - Network metrics</div>
            <div class="endpoint"><a href="/api/economics">/api/economics</a> - Economics metrics</div>
            <div class="endpoint"><a href="/api/history">/api/history</a> - Metrics history for charts</div>
            <div class="endpoint">/ws - WebSocket for real-time updates</div>
        </div>

        <div class="build-instructions">
            <strong>To build the Vue.js frontend:</strong>
            <pre>cd crates/vault-node/dashboard-frontend
npm install
npm run build</pre>
            <p style="margin-top: 10px; color: #888;">
                Then rebuild the vault-node with <code>cargo build -p vault-node --features dashboard</code>
            </p>
        </div>
    </div>
</body>
</html>"#.to_string()
}
