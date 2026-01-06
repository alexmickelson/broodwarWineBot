use serde::{Deserialize, Serialize};

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

pub fn generate_map_svg(map_data: &MapData) -> String {
    if map_data.width == 0 || map_data.height == 0 {
        return String::from("<svg></svg>");
    }

    // Scale down the map for display (each walk tile is 1 pixel in SVG)
    let scale = 3; // pixels per walk tile
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
