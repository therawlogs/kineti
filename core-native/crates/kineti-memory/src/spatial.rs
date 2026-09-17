//! # Geospatial Memory & Geofencing Substrate (`spatial`)
//!
//! Handles:
//! - High-precision geographic coordinates and Haversine distance calculations.
//! - Parking recall: records parked vehicle location and provides walking guidance.
//! - Transit guidance: calculates distance, direction, and ETA to registered landmarks.
//! - Geofence state transitions (`Entering`, `Exiting`, `Inside`, `Outside`).
//! - Historical visit walks with privacy tombstone integration.

use kineti_core::current_epoch_millis;
use std::collections::HashMap;
use std::sync::RwLock;

/// Geographic coordinate with accuracy and timestamp.
#[derive(Debug, Clone, PartialEq)]
pub struct GeoCoordinate {
    /// Latitude in degrees (-90.0 to +90.0).
    pub lat: f64,
    /// Longitude in degrees (-180.0 to +180.0).
    pub lon: f64,
    /// Estimated horizontal accuracy in meters.
    pub accuracy_m: f32,
    /// Fix timestamp (Unix ms).
    pub timestamp_ms: u64,
}

impl GeoCoordinate {
    /// Computes great-circle distance in meters between two coordinates using the Haversine formula.
    pub fn distance_to(&self, other: &GeoCoordinate) -> f64 {
        const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

        let lat1 = self.lat.to_radians();
        let lat2 = other.lat.to_radians();
        let delta_lat = (other.lat - self.lat).to_radians();
        let delta_lon = (other.lon - self.lon).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_METERS * c
    }
}

/// Geofence region category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeofenceCategory {
    /// Home residence.
    Home,
    /// Work office.
    Work,
    /// Airport or transit hub.
    Airport,
    /// Fitness center or gym.
    Gym,
    /// Parking structure or space.
    Parking,
    /// Custom user-defined landmark.
    Custom,
}

/// Named circular geofence boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedGeofence {
    /// Unique geofence identifier.
    pub id: String,
    /// Human-readable label (e.g. "Home", "SFO Airport").
    pub name: String,
    /// Category classification.
    pub category: GeofenceCategory,
    /// Center coordinate.
    pub center: GeoCoordinate,
    /// Boundary radius in meters.
    pub radius_meters: f64,
}

/// State of a user relative to a geofence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeofenceTransition {
    /// User crossed into the geofence boundary.
    Entering,
    /// User crossed out of the geofence boundary.
    Exiting,
    /// User remained inside.
    Inside,
    /// User remained outside.
    Outside,
}

/// Recorded parked vehicle location event.
#[derive(Debug, Clone, PartialEq)]
pub struct ParkedLocation {
    /// Exact coordinate of vehicle.
    pub coordinate: GeoCoordinate,
    /// Optional parking garage, level, or parking space note.
    pub note: Option<String>,
    /// Timestamp when vehicle was parked (Unix ms).
    pub parked_at_ms: u64,
}

/// Spatial Memory Engine.
#[derive(Debug, Default)]
pub struct SpatialMemoryEngine {
    geofences: RwLock<HashMap<String, NamedGeofence>>,
    last_known_location: RwLock<Option<GeoCoordinate>>,
    parked_location: RwLock<Option<ParkedLocation>>,
    previous_geofence_states: RwLock<HashMap<String, bool>>, // geofence_id -> was_inside
}

impl SpatialMemoryEngine {
    /// Creates a new spatial memory engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a named geofence.
    pub fn register_geofence(&self, geofence: NamedGeofence) {
        let mut map = self.geofences.write().unwrap();
        map.insert(geofence.id.clone(), geofence);
    }

    /// Records or updates the user's latest location fix and detects geofence boundary transitions.
    pub fn update_location(&self, coord: GeoCoordinate) -> Vec<(NamedGeofence, GeofenceTransition)> {
        let geofences = self.geofences.read().unwrap();
        let mut prev_states = self.previous_geofence_states.write().unwrap();
        let mut transitions = Vec::new();

        for (id, fence) in geofences.iter() {
            let dist = coord.distance_to(&fence.center);
            let is_inside = dist <= fence.radius_meters;
            let was_inside = prev_states.get(id).copied().unwrap_or(false);

            let transition = match (was_inside, is_inside) {
                (false, true) => GeofenceTransition::Entering,
                (true, false) => GeofenceTransition::Exiting,
                (true, true) => GeofenceTransition::Inside,
                (false, false) => GeofenceTransition::Outside,
            };

            prev_states.insert(id.clone(), is_inside);
            if transition == GeofenceTransition::Entering || transition == GeofenceTransition::Exiting {
                transitions.push((fence.clone(), transition));
            }
        }

        *self.last_known_location.write().unwrap() = Some(coord);
        transitions
    }

    /// Records vehicle parking location with optional note.
    pub fn record_parked_vehicle(&self, coord: GeoCoordinate, note: Option<String>) {
        let now = current_epoch_millis();
        let parked = ParkedLocation {
            coordinate: coord,
            note,
            parked_at_ms: now,
        };
        *self.parked_location.write().unwrap() = Some(parked);
    }

    /// Recalls parked vehicle location with distance from current location and elapsed time.
    pub fn recall_parking(&self) -> Option<(ParkedLocation, Option<f64>, u64)> {
        let parked = self.parked_location.read().unwrap().clone()?;
        let last_loc = self.last_known_location.read().unwrap().clone();
        let distance_m = last_loc.as_ref().map(|l| l.distance_to(&parked.coordinate));
        let elapsed_secs = (current_epoch_millis().saturating_sub(parked.parked_at_ms)) / 1000;

        Some((parked, distance_m, elapsed_secs))
    }

    /// Provides transit guidance distance to a registered landmark (e.g. "Home", "Work").
    pub fn transit_guidance_to(&self, landmark_name: &str) -> Option<(f64, u32)> {
        let last_loc = self.last_known_location.read().unwrap().clone()?;
        let geofences = self.geofences.read().unwrap();
        let landmark = geofences.values().find(|g| g.name.eq_ignore_ascii_case(landmark_name))?;

        let distance_meters = last_loc.distance_to(&landmark.center);
        // Average driving speed estimate: 45 km/h = 12.5 m/s
        let estimated_minutes = ((distance_meters / 12.5) / 60.0).ceil() as u32;

        Some((distance_meters, estimated_minutes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance_calculation() {
        // Distance between SF Ferry Building and Coit Tower ~ 1.5 km
        let ferry_building = GeoCoordinate {
            lat: 37.7955,
            lon: -122.3937,
            accuracy_m: 5.0,
            timestamp_ms: 1000,
        };
        let coit_tower = GeoCoordinate {
            lat: 37.8024,
            lon: -122.4058,
            accuracy_m: 5.0,
            timestamp_ms: 1000,
        };

        let distance = ferry_building.distance_to(&coit_tower);
        assert!((distance - 1300.0).abs() < 200.0, "Expected ~1300-1400m, got {}", distance);
    }

    #[test]
    fn test_parking_recall_flow() {
        let engine = SpatialMemoryEngine::new();
        let parked_coord = GeoCoordinate {
            lat: 37.7879,
            lon: -122.4075,
            accuracy_m: 3.0,
            timestamp_ms: current_epoch_millis(),
        };

        engine.record_parked_vehicle(parked_coord.clone(), Some("Level 3, Space 42B".to_string()));

        // Update current location 150m away
        let current_coord = GeoCoordinate {
            lat: 37.7890,
            lon: -122.4080,
            accuracy_m: 5.0,
            timestamp_ms: current_epoch_millis(),
        };
        let _ = engine.update_location(current_coord);

        let (parked, dist, _) = engine.recall_parking().expect("Recalls parking");
        assert_eq!(parked.note.as_deref(), Some("Level 3, Space 42B"));
        assert!(dist.is_some());
        assert!(dist.unwrap() > 50.0 && dist.unwrap() < 200.0);
    }

    #[test]
    fn test_geofence_arrival_and_departure_triggers() {
        let engine = SpatialMemoryEngine::new();
        let sfo_airport = NamedGeofence {
            id: "sfo_01".to_string(),
            name: "SFO Airport".to_string(),
            category: GeofenceCategory::Airport,
            center: GeoCoordinate {
                lat: 37.6213,
                lon: -122.3790,
                accuracy_m: 10.0,
                timestamp_ms: 1000,
            },
            radius_meters: 500.0,
        };
        engine.register_geofence(sfo_airport);

        // 1. User arrives at airport -> Entering
        let at_airport = GeoCoordinate {
            lat: 37.6215,
            lon: -122.3792,
            accuracy_m: 5.0,
            timestamp_ms: 1050,
        };
        let transitions1 = engine.update_location(at_airport);
        assert_eq!(transitions1.len(), 1);
        assert_eq!(transitions1[0].1, GeofenceTransition::Entering);

        // 2. User leaves airport -> Exiting
        let outside_airport = GeoCoordinate {
            lat: 37.7000,
            lon: -122.4000,
            accuracy_m: 5.0,
            timestamp_ms: 2000,
        };
        let transitions2 = engine.update_location(outside_airport);
        assert_eq!(transitions2.len(), 1);
        assert_eq!(transitions2[0].1, GeofenceTransition::Exiting);
    }
}
