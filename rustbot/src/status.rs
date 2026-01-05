use axum::{response::Html, routing::get, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerStatus {
    pub total: usize,
    pub gathering: usize,
    pub idle: usize,
    pub building: usize,
}

impl Default for WorkerStatus {
    fn default() -> Self {
        Self {
            total: 0,
            gathering: 0,
            idle: 0,
            building: 0,
        }
    }
}

pub type SharedStatus = Arc<Mutex<WorkerStatus>>;

pub fn create_shared_status() -> SharedStatus {
    Arc::new(Mutex::new(WorkerStatus::default()))
}

async fn status_handler(state: axum::extract::State<SharedStatus>) -> Html<String> {
    let status = state.lock().unwrap();

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>RustBot Status</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }}
        .container {{
            background: rgba(255, 255, 255, 0.1);
            border-radius: 15px;
            padding: 30px;
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.37);
        }}
        h1 {{
            text-align: center;
            margin-bottom: 30px;
            font-size: 2.5em;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }}
        .stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 20px;
            margin-top: 20px;
        }}
        .stat-card {{
            background: rgba(255, 255, 255, 0.15);
            padding: 20px;
            border-radius: 10px;
            text-align: center;
            transition: transform 0.3s;
        }}
        .stat-card:hover {{
            transform: translateY(-5px);
        }}
        .stat-number {{
            font-size: 3em;
            font-weight: bold;
            margin: 10px 0;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }}
        .stat-label {{
            font-size: 1.2em;
            opacity: 0.9;
        }}
        .refresh-note {{
            text-align: center;
            margin-top: 30px;
            opacity: 0.7;
            font-style: italic;
        }}
    </style>
    <script>
        // Auto-refresh every 2 seconds
        setTimeout(function(){{ location.reload(); }}, 2000);
    </script>
</head>
<body>
    <div class="container">
        <h1>🤖 RustBot Worker Status</h1>
        <div class="stats">
            <div class="stat-card">
                <div class="stat-label">Total Workers</div>
                <div class="stat-number">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Gathering</div>
                <div class="stat-number">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Idle</div>
                <div class="stat-number">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Building</div>
                <div class="stat-number">{}</div>
            </div>
        </div>
        <div class="refresh-note">
            Page auto-refreshes every 2 seconds
        </div>
    </div>
</body>
</html>"#,
        status.total, status.gathering, status.idle, status.building
    );

    Html(html)
}

pub async fn start_server(status: SharedStatus) {
    let app = Router::new()
        .route("/", get(status_handler))
        .with_state(status);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Status server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
