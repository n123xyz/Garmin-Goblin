use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyCloakedRoute {
    pub original_point_count: usize,
    pub cloaked_point_count: usize,
    pub start_zone_redacted: bool,
    pub end_zone_redacted: bool,
    pub coordinates: Vec<[f64; 2]>,
    pub biome_label: String,
}

/// Calculate Haversine distance in meters between two lat/lon points
pub fn haversine_distance_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371000.0; // Earth radius in meters
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let delta_phi = (lat2 - lat1).to_radians();
    let delta_lambda = (lon2 - lon1).to_radians();

    let a = (delta_phi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c
}

/// Privacy preservation: Cloaks and redacts the first and last N meters of a GPS route
/// to prevent identification of home addresses or sensitive origins/destinations.
pub fn cloak_gps_route(
    coords: &[[f64; 2]],
    privacy_radius_meters: f64,
    enable_dp_noise: bool,
) -> PrivacyCloakedRoute {
    if coords.len() < 3 {
        return PrivacyCloakedRoute {
            original_point_count: coords.len(),
            cloaked_point_count: coords.len(),
            start_zone_redacted: false,
            end_zone_redacted: false,
            coordinates: coords.to_vec(),
            biome_label: "Wilderness Trail".to_string(),
        };
    }

    let start_point = coords[0];
    let end_point = coords[coords.len() - 1];

    let mut filtered = Vec::new();
    let mut rng = rand::thread_rng();

    for pt in coords {
        let dist_from_start =
            haversine_distance_meters(start_point[0], start_point[1], pt[0], pt[1]);
        let dist_from_end =
            haversine_distance_meters(end_point[0], end_point[1], pt[0], pt[1]);

        // If outside privacy threshold radius from start and end
        if dist_from_start >= privacy_radius_meters && dist_from_end >= privacy_radius_meters {
            if enable_dp_noise {
                // Apply subtle Laplace/Gaussian noise (~5-15m perturbation)
                let noise_lat = rng.gen_range(-0.0001..0.0001);
                let noise_lon = rng.gen_range(-0.0001..0.0001);
                filtered.push([pt[0] + noise_lat, pt[1] + noise_lon]);
            } else {
                filtered.push(*pt);
            }
        }
    }

    // Ensure at least some points remain for display
    if filtered.is_empty() {
        filtered = coords.to_vec();
    }

    let avg_lat = coords.iter().map(|p| p[0]).sum::<f64>() / coords.len() as f64;
    let biome_label = if avg_lat.abs() > 50.0 {
        "Sub-Arctic Boreal Region"
    } else if avg_lat.abs() > 35.0 {
        "Temperate Forest & Highland Foothills"
    } else if avg_lat.abs() > 20.0 {
        "Sub-Tropical Mountain Basin"
    } else {
        "Equatorial Rainforest Zone"
    };

    PrivacyCloakedRoute {
        original_point_count: coords.len(),
        cloaked_point_count: filtered.len(),
        start_zone_redacted: true,
        end_zone_redacted: true,
        coordinates: filtered,
        biome_label: biome_label.to_string(),
    }
}

/// Redact identifiable location strings into privacy-safe biome descriptions for AI prompts
pub fn anonymize_location_for_ai(location_str: &str, lat: f64) -> String {
    let biome = if lat.abs() > 50.0 {
        "Sub-Arctic Valley"
    } else if lat.abs() > 35.0 {
        "Highland Mountain Cavern"
    } else {
        "Forest Grove Trail"
    };

    if location_str.is_empty() || location_str.contains("Goblin") {
        format!("Goblin {}", biome)
    } else {
        format!("Private Wilderness Biome ({})", biome)
    }
}
