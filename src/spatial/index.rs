use geo::Point;
use rstar::{RTree, RTreeObject, AABB};

/// Spatial index entry
#[derive(Debug, Clone)]
pub struct SpatialEntry {
    pub id: String,
    pub point: Point<f64>,
}

impl RTreeObject for SpatialEntry {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.point.x(), self.point.y()])
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
        self.tree
            .locate_within_distance([point.x(), point.y()], radius * radius)
            .collect()
    }

    /// Find nearest entry
    pub fn find_nearest(&self, point: Point<f64>) -> Option<&SpatialEntry> {
        self.tree.nearest_neighbor(&[point.x(), point.y()])
    }

    /// Find k nearest entries
    pub fn find_k_nearest(&self, point: Point<f64>, k: usize) -> Vec<&SpatialEntry> {
        self.tree
            .nearest_neighbor_iter(&[point.x(), point.y()])
            .take(k)
            .collect()
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
                point: Point::new(0.0, 0.0),
            },
            SpatialEntry {
                id: "2".to_string(),
                point: Point::new(1.0, 1.0),
            },
            SpatialEntry {
                id: "3".to_string(),
                point: Point::new(2.0, 2.0),
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
                point: Point::new(0.0, 0.0),
            },
            SpatialEntry {
                id: "2".to_string(),
                point: Point::new(0.1, 0.1),
            },
            SpatialEntry {
                id: "3".to_string(),
                point: Point::new(10.0, 10.0),
            },
        ];

        let index = SpatialIndex::new(entries);

        let results = index.find_within_radius(Point::new(0.0, 0.0), 1.0);
        assert_eq!(results.len(), 2); // Should find 1 and 2, but not 3
    }
}
