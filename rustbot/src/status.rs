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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitInfo {
    pub x: i32,
    pub y: i32,
    pub unit_type: String,
    pub is_ally: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub x: i32,
    pub y: i32,
    pub resource_type: String,
    pub amount: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapData {
    pub width: usize,
    pub height: usize,
    pub walkability: Vec<Vec<bool>>, // true if walkable
    pub explored: Vec<Vec<bool>>,    // true if explored
    pub units: Vec<UnitInfo>,
    pub resources: Vec<ResourceInfo>,
}

impl Default for MapData {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            walkability: Vec::new(),
            explored: Vec::new(),
            units: Vec::new(),
            resources: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct GameStatus {
    pub worker_status: WorkerStatus,
    pub map_data: MapData,
}

pub type SharedStatus = Arc<Mutex<GameStatus>>;

pub fn create_shared_status() -> SharedStatus {
    Arc::new(Mutex::new(GameStatus::default()))
}

fn generate_map_svg(map_data: &MapData) -> String {
    if map_data.width == 0 || map_data.height == 0 {
        return String::from("<svg></svg>");
    }

    // Scale down the map for display (each walk tile is 1 pixel in SVG)
    let scale = 4; // Scale factor for better visibility
    let svg_width = map_data.width * scale;
    let svg_height = map_data.height * scale;

    let mut svg = format!(
        r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
        svg_width, svg_height, svg_width, svg_height
    );

    // Draw background (unexplored areas)
    svg.push_str(&format!(
        "<rect width=\"{}\" height=\"{}\" fill=\"#000000\"/>",
        svg_width, svg_height
    ));

    // Draw explored and walkability
    for y in 0..map_data.height {
        for x in 0..map_data.width {
            let is_explored = map_data
                .explored
                .get(y)
                .and_then(|row| row.get(x))
                .copied()
                .unwrap_or(false);
            let is_walkable = map_data
                .walkability
                .get(y)
                .and_then(|row| row.get(x))
                .copied()
                .unwrap_or(false);

            if is_explored {
                let color = if is_walkable {
                    "#2a4a2a" // Dark green for walkable explored terrain
                } else {
                    "#4a4a4a" // Gray for unwalkable terrain
                };

                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
                    x * scale,
                    y * scale,
                    scale,
                    scale,
                    color
                ));
            }
        }
    }

    // Draw resources
    for resource in &map_data.resources {
        let color = match resource.resource_type.as_str() {
            "minerals" => "#00FFFF", // Cyan for minerals
            "gas" => "#00FF00",      // Green for gas
            _ => "#FFFF00",          // Yellow for other
        };

        svg.push_str(&format!(
            r#"<circle cx="{}" cy="{}" r="{}" fill="{}" opacity="0.8"/>"#,
            resource.x / 8 * (scale as i32) + (scale / 2) as i32,
            resource.y / 8 * (scale as i32) + (scale / 2) as i32,
            scale * 2,
            color
        ));
    }

    // Draw units
    for unit in &map_data.units {
        let color = if unit.is_ally {
            "#0000FF" // Blue for allies
        } else {
            "#FF0000" // Red for enemies
        };

        svg.push_str(&format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" stroke=\"#FFFFFF\" stroke-width=\"0.5\"/>",
            unit.x / 8 * (scale as i32),
            unit.y / 8 * (scale as i32),
            scale,
            color
        ));
    }

    svg.push_str("</svg>");
    svg
}

async fn status_handler(state: axum::extract::State<SharedStatus>) -> Html<String> {
    let status = state.lock().unwrap();
    let map_svg = generate_map_svg(&status.map_data);

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
            max-width: 1400px;
            margin: 20px auto;
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
        h2 {{
            text-align: center;
            margin-top: 30px;
            margin-bottom: 20px;
            font-size: 1.8em;
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
        .map-container {{
            background: rgba(0, 0, 0, 0.3);
            border-radius: 10px;
            padding: 20px;
            margin-top: 30px;
            display: flex;
            justify-content: center;
            align-items: center;
            overflow: auto;
        }}
        .map-container svg {{
            border: 2px solid rgba(255, 255, 255, 0.3);
            background: #000000;
            max-width: 100%;
            height: auto;
        }}
        .legend {{
            display: flex;
            justify-content: center;
            gap: 30px;
            margin-top: 20px;
            flex-wrap: wrap;
        }}
        .legend-item {{
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        .legend-color {{
            width: 20px;
            height: 20px;
            border-radius: 50%;
            border: 2px solid white;
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
        <h1>🤖 RustBot Status Dashboard</h1>
        
        <h2>Worker Status</h2>
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

        <h2>Map Visualization</h2>
        <div class="legend">
            <div class="legend-item">
                <div class="legend-color" style="background: #2a4a2a;"></div>
                <span>Walkable Terrain</span>
            </div>
            <div class="legend-item">
                <div class="legend-color" style="background: #4a4a4a;"></div>
                <span>Unwalkable Terrain</span>
            </div>
            <div class="legend-item">
                <div class="legend-color" style="background: #000000;"></div>
                <span>Unexplored</span>
            </div>
            <div class="legend-item">
                <div class="legend-color" style="background: #0000FF;"></div>
                <span>Allied Units</span>
            </div>
            <div class="legend-item">
                <div class="legend-color" style="background: #FF0000;"></div>
                <span>Enemy Units</span>
            </div>
            <div class="legend-item">
                <div class="legend-color" style="background: #00FFFF;"></div>
                <span>Minerals</span>
            </div>
            <div class="legend-item">
                <div class="legend-color" style="background: #00FF00;"></div>
                <span>Gas</span>
            </div>
        </div>
        <div class="map-container">
            {}
        </div>

        <div class="refresh-note">
            Page auto-refreshes every 2 seconds
        </div>
    </div>
</body>
</html>"#,
        status.worker_status.total,
        status.worker_status.gathering,
        status.worker_status.idle,
        status.worker_status.building,
        map_svg
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
