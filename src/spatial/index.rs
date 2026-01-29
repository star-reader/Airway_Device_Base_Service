use geo::Point;
use rstar::{RTree, RTreeObject, AABB};

/// Spatial index entry
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialEntry {
    pub id: String,
    pub point: [f64; 2],
}

impl RTreeObject for SpatialEntry {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.point)
    }
}

/// In-memory spatial index using R-Tree
pub struct SpatialIndex {
    tree: RTree<SpatialEntry>,
}

impl SpatialIndex {
    /// Create a new spatial index
    pub fn new(entries: Vec<SpatialEntry>) -> Self {
        let tree = RTree::bulk_load(entries);
        Self { tree }
    }

    /// Find entries within a radius (in degrees)
    pub fn find_within_radius(&self, point: Point<f64>, radius: f64) -> Vec<&SpatialEntry> {
        let search_point = [point.x(), point.y()];
        let bbox = AABB::from_corners(
            [search_point[0] - radius, search_point[1] - radius],
            [search_point[0] + radius, search_point[1] + radius],
        );
        
        self.tree
            .locate_in_envelope(&bbox)
            .filter(|entry| {
                let dx = entry.point[0] - search_point[0];
                let dy = entry.point[1] - search_point[1];
                (dx * dx + dy * dy).sqrt() <= radius
            })
            .collect()
    }

    /// Find nearest entry (using linear search for simplicity)
    pub fn find_nearest(&self, point: Point<f64>) -> Option<&SpatialEntry> {
        let search_point = [point.x(), point.y()];
        self.tree
            .iter()
            .min_by(|a, b| {
                let dist_a = {
                    let dx = a.point[0] - search_point[0];
                    let dy = a.point[1] - search_point[1];
                    dx * dx + dy * dy
                };
                let dist_b = {
                    let dx = b.point[0] - search_point[0];
                    let dy = b.point[1] - search_point[1];
                    dx * dx + dy * dy
                };
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Find k nearest entries
    pub fn find_k_nearest(&self, point: Point<f64>, k: usize) -> Vec<&SpatialEntry> {
        let search_point = [point.x(), point.y()];
        let mut entries: Vec<_> = self.tree
            .iter()
            .map(|entry| {
                let dx = entry.point[0] - search_point[0];
                let dy = entry.point[1] - search_point[1];
                let dist = dx * dx + dy * dy;
                (entry, dist)
            })
            .collect();
        
        entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        entries.into_iter().take(k).map(|(entry, _)| entry).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_index() {
        let entries = vec![
            SpatialEntry {
                id: "1".to_string(),
                point: [0.0, 0.0],
            },
            SpatialEntry {
                id: "2".to_string(),
                point: [1.0, 1.0],
            },
            SpatialEntry {
                id: "3".to_string(),
                point: [2.0, 2.0],
            },
        ];

        let index = SpatialIndex::new(entries);

        // Find nearest to origin
        let nearest = index.find_nearest(Point::new(0.0, 0.0));
        assert!(nearest.is_some());
        assert_eq!(nearest.unwrap().id, "1");
    }

    #[test]
    fn test_find_within_radius() {
        let entries = vec![
            SpatialEntry {
                id: "1".to_string(),
                point: [0.0, 0.0],
            },
            SpatialEntry {
                id: "2".to_string(),
                point: [0.1, 0.1],
            },
            SpatialEntry {
                id: "3".to_string(),
                point: [10.0, 10.0],
            },
        ];

        let index = SpatialIndex::new(entries);

        let results = index.find_within_radius(Point::new(0.0, 0.0), 1.0);
        assert_eq!(results.len(), 2); // Should find 1 and 2, but not 3
    }
}
