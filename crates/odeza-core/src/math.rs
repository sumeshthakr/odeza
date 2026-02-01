//! Math utilities
//!
//! Re-exports from glam and additional math utilities for game development.

pub use glam::{
    Mat3, Mat4, Quat, Vec2, Vec3, Vec4,
    IVec2, IVec3, IVec4,
    UVec2, UVec3, UVec4,
    Affine3A,
};

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    /// Minimum corner
    pub min: Vec3,
    /// Maximum corner
    pub max: Vec3,
}

impl Aabb {
    /// Create an empty AABB
    pub const EMPTY: Self = Self {
        min: Vec3::splat(f32::INFINITY),
        max: Vec3::splat(f32::NEG_INFINITY),
    };

    /// Create an AABB from min and max corners
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an AABB from center and half-extents
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Create a unit AABB centered at origin
    pub fn unit() -> Self {
        Self::from_center_half_extents(Vec3::ZERO, Vec3::splat(0.5))
    }

    /// Get the center of the AABB
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the half-extents of the AABB
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Get the full size of the AABB
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Check if the AABB is empty
    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    /// Check if a point is inside the AABB
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    /// Check if this AABB intersects another
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    /// Expand the AABB to include a point
    pub fn expand_to_include(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Merge with another AABB
    pub fn merge(&self, other: &Aabb) -> Aabb {
        Aabb {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Transform the AABB by a matrix
    pub fn transform(&self, matrix: Mat4) -> Aabb {
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        let mut result = Aabb::EMPTY;
        for corner in corners {
            let transformed = matrix.transform_point3(corner);
            result.expand_to_include(transformed);
        }
        result
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self::EMPTY
    }
}

/// Bounding sphere
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingSphere {
    /// Center of the sphere
    pub center: Vec3,
    /// Radius of the sphere
    pub radius: f32,
}

impl BoundingSphere {
    /// Create a new bounding sphere
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    /// Create a unit sphere at origin
    pub fn unit() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: 1.0,
        }
    }

    /// Check if a point is inside the sphere
    pub fn contains_point(&self, point: Vec3) -> bool {
        (point - self.center).length_squared() <= self.radius * self.radius
    }

    /// Check if this sphere intersects another
    pub fn intersects(&self, other: &BoundingSphere) -> bool {
        let distance_sq = (other.center - self.center).length_squared();
        let radius_sum = self.radius + other.radius;
        distance_sq <= radius_sum * radius_sum
    }

    /// Check if this sphere intersects an AABB
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        // Find the closest point on the AABB to the sphere center
        let closest = Vec3::new(
            self.center.x.clamp(aabb.min.x, aabb.max.x),
            self.center.y.clamp(aabb.min.y, aabb.max.y),
            self.center.z.clamp(aabb.min.z, aabb.max.z),
        );
        
        self.contains_point(closest)
    }

    /// Create a bounding sphere from an AABB
    pub fn from_aabb(aabb: &Aabb) -> Self {
        let center = aabb.center();
        let radius = aabb.half_extents().length();
        Self { center, radius }
    }
}

impl Default for BoundingSphere {
    fn default() -> Self {
        Self::unit()
    }
}

/// Frustum for culling
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    /// Frustum planes (left, right, bottom, top, near, far)
    pub planes: [Plane; 6],
}

/// A plane in 3D space (ax + by + cz + d = 0)
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Normal vector
    pub normal: Vec3,
    /// Distance from origin
    pub distance: f32,
}

impl Plane {
    /// Create a new plane
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// Create a plane from a point and normal
    pub fn from_point_normal(point: Vec3, normal: Vec3) -> Self {
        let normal = normal.normalize();
        Self {
            normal,
            distance: -normal.dot(point),
        }
    }

    /// Get the signed distance from a point to the plane
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

impl Frustum {
    /// Create a frustum from a view-projection matrix
    pub fn from_matrix(matrix: Mat4) -> Self {
        let rows = [
            matrix.row(0),
            matrix.row(1),
            matrix.row(2),
            matrix.row(3),
        ];

        let mut planes = [Plane::new(Vec3::ZERO, 0.0); 6];

        // Left plane
        planes[0] = Self::normalize_plane(rows[3] + rows[0]);
        // Right plane
        planes[1] = Self::normalize_plane(rows[3] - rows[0]);
        // Bottom plane
        planes[2] = Self::normalize_plane(rows[3] + rows[1]);
        // Top plane
        planes[3] = Self::normalize_plane(rows[3] - rows[1]);
        // Near plane
        planes[4] = Self::normalize_plane(rows[3] + rows[2]);
        // Far plane
        planes[5] = Self::normalize_plane(rows[3] - rows[2]);

        Self { planes }
    }

    fn normalize_plane(plane: glam::Vec4) -> Plane {
        let length = Vec3::new(plane.x, plane.y, plane.z).length();
        if length > 0.0 {
            Plane {
                normal: Vec3::new(plane.x, plane.y, plane.z) / length,
                distance: plane.w / length,
            }
        } else {
            Plane::new(Vec3::ZERO, 0.0)
        }
    }

    /// Check if a point is inside the frustum
    pub fn contains_point(&self, point: Vec3) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(point) < 0.0 {
                return false;
            }
        }
        true
    }

    /// Check if an AABB intersects the frustum
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        for plane in &self.planes {
            // Find the positive vertex (furthest along the plane normal)
            let p = Vec3::new(
                if plane.normal.x >= 0.0 { aabb.max.x } else { aabb.min.x },
                if plane.normal.y >= 0.0 { aabb.max.y } else { aabb.min.y },
                if plane.normal.z >= 0.0 { aabb.max.z } else { aabb.min.z },
            );

            if plane.distance_to_point(p) < 0.0 {
                return false;
            }
        }
        true
    }

    /// Check if a sphere intersects the frustum
    pub fn intersects_sphere(&self, sphere: &BoundingSphere) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(sphere.center) < -sphere.radius {
                return false;
            }
        }
        true
    }
}

/// Ray for raycasting
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Ray origin
    pub origin: Vec3,
    /// Ray direction (normalized)
    pub direction: Vec3,
}

impl Ray {
    /// Create a new ray
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get a point along the ray at distance t
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Intersect with an AABB, returns (t_min, t_max) if hit
    pub fn intersect_aabb(&self, aabb: &Aabb) -> Option<(f32, f32)> {
        let inv_dir = Vec3::new(
            1.0 / self.direction.x,
            1.0 / self.direction.y,
            1.0 / self.direction.z,
        );

        let t1 = (aabb.min - self.origin) * inv_dir;
        let t2 = (aabb.max - self.origin) * inv_dir;

        let t_min = t1.min(t2);
        let t_max = t1.max(t2);

        let t_enter = t_min.x.max(t_min.y).max(t_min.z);
        let t_exit = t_max.x.min(t_max.y).min(t_max.z);

        if t_enter <= t_exit && t_exit >= 0.0 {
            Some((t_enter.max(0.0), t_exit))
        } else {
            None
        }
    }

    /// Intersect with a sphere, returns (t_min, t_max) if hit
    pub fn intersect_sphere(&self, sphere: &BoundingSphere) -> Option<(f32, f32)> {
        let oc = self.origin - sphere.center;
        let a = self.direction.length_squared();
        let half_b = oc.dot(self.direction);
        let c = oc.length_squared() - sphere.radius * sphere.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            None
        } else {
            let sqrt_d = discriminant.sqrt();
            let t1 = (-half_b - sqrt_d) / a;
            let t2 = (-half_b + sqrt_d) / a;
            Some((t1, t2))
        }
    }
}

/// Linear interpolation
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Inverse linear interpolation
pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if (b - a).abs() < f32::EPSILON {
        0.0
    } else {
        (value - a) / (b - a)
    }
}

/// Remap a value from one range to another
pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = inverse_lerp(from_min, from_max, value);
    lerp(to_min, to_max, t)
}

/// Smoothstep interpolation
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smoother step interpolation (Ken Perlin's version)
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_creation() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert_eq!(aabb.center(), Vec3::splat(0.5));
        assert_eq!(aabb.size(), Vec3::ONE);
    }

    #[test]
    fn test_aabb_contains_point() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains_point(Vec3::splat(0.5)));
        assert!(!aabb.contains_point(Vec3::splat(2.0)));
    }

    #[test]
    fn test_aabb_intersection() {
        let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let b = Aabb::new(Vec3::splat(0.5), Vec3::splat(1.5));
        let c = Aabb::new(Vec3::splat(2.0), Vec3::splat(3.0));
        
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_sphere_intersection() {
        let a = BoundingSphere::new(Vec3::ZERO, 1.0);
        let b = BoundingSphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);
        let c = BoundingSphere::new(Vec3::new(5.0, 0.0, 0.0), 1.0);
        
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_ray_aabb_intersection() {
        let ray = Ray::new(Vec3::new(-5.0, 0.5, 0.5), Vec3::X);
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        
        let hit = ray.intersect_aabb(&aabb);
        assert!(hit.is_some());
        
        let (t_min, _t_max) = hit.unwrap();
        let hit_point = ray.at(t_min);
        assert!((hit_point.x - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_ray_sphere_intersection() {
        let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::X);
        let sphere = BoundingSphere::new(Vec3::ZERO, 1.0);
        
        let hit = ray.intersect_sphere(&sphere);
        assert!(hit.is_some());
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_inverse_lerp() {
        assert_eq!(inverse_lerp(0.0, 10.0, 5.0), 0.5);
    }

    #[test]
    fn test_remap() {
        let result = remap(5.0, 0.0, 10.0, 0.0, 100.0);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_smoothstep() {
        assert_eq!(smoothstep(0.0, 1.0, 0.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 1.0), 1.0);
        assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < 0.001);
    }
}
