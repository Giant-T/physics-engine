use nalgebra::{UnitVector3, Vector3};

pub struct Contact {
    pub point: Vector3<f64>,
    pub normal: UnitVector3<f64>,
    pub depth: f64,
}

#[derive(Debug, PartialEq)]
pub enum ContactDetection {
    PointTriangle,
    PointEdge,
    EdgeEdge,
}

// basé sur: http://www.r-5.org/files/books/computers/algo-list/realtime-3d/Christer_Ericson-Real-Time_Collision_Detection-EN.pdf
impl ContactDetection {
    pub fn tests_count(&self) -> usize {
        match self {
            ContactDetection::PointTriangle => 3,
            ContactDetection::PointEdge => 9,
            ContactDetection::EdgeEdge => 9,
        }
    }

    pub fn detect(
        &self,
        triangle_a: [Vector3<f64>; 3],
        triangle_b: [&Vector3<f64>; 3],
        epsilon: f64,
    ) -> Vec<Contact> {
        match self {
            Self::PointTriangle => {
                let mut contacts = Vec::with_capacity(3);
                for point in triangle_a {
                    if let Some(contact) = Self::point_triangle(
                        &point,
                        triangle_b[0],
                        triangle_b[1],
                        triangle_b[2],
                        epsilon,
                    ) {
                        contacts.push(contact);
                    }
                }
                contacts
            }
            Self::PointEdge => {
                let mut contacts = Vec::with_capacity(6);
                let triangle_b_edges = [
                    (triangle_b[0], triangle_b[1]),
                    (triangle_b[0], triangle_b[2]),
                    (triangle_b[1], triangle_b[2]),
                ];
                for point in triangle_a {
                    contacts.extend(
                        triangle_b_edges.iter().filter_map(|(start, end)| {
                            Self::point_edge(&point, start, end, epsilon)
                        }),
                    )
                }
                contacts
            }
            Self::EdgeEdge => {
                let mut contacts = Vec::with_capacity(6);
                let triangle_a_edges = [
                    (triangle_a[0], triangle_a[1]),
                    (triangle_a[0], triangle_a[2]),
                    (triangle_a[1], triangle_a[2]),
                ];
                let triangle_b_edges = [
                    (triangle_b[0], triangle_b[1]),
                    (triangle_b[0], triangle_b[2]),
                    (triangle_b[1], triangle_b[2]),
                ];
                for (start1, end1) in triangle_a_edges {
                    contacts.extend(triangle_b_edges.iter().filter_map(|(start2, end2)| {
                        Self::edge_edge(&start1, &end1, start2, end2, epsilon)
                    }));
                }
                contacts
            }
        }
    }

    pub fn point_triangle(
        point: &Vector3<f64>,
        a: &Vector3<f64>,
        b: &Vector3<f64>,
        c: &Vector3<f64>,
        epsilon: f64,
    ) -> Option<Contact> {
        let normal = UnitVector3::new_normalize((b - a).cross(&(c - a)));
        let dist = normal.dot(&(point - a));

        let area = (b - a).cross(&(c - a)).norm();

        if dist.abs() >= epsilon || area == 0.0 {
            return None;
        }

        let q = point - dist * *normal;

        let u = normal.dot(&(b - q).cross(&(c - q))) / area;
        let v = normal.dot(&(c - q).cross(&(a - q))) / area;
        let w = normal.dot(&(a - q).cross(&(b - q))) / area;

        let is_inside = u >= 0.0 && v >= 0.0 && w >= 0.0;

        if !is_inside {
            return None;
        }

        Some(Contact {
            point: q,
            normal,
            depth: dist.abs(),
        })
    }

    pub fn point_edge(
        point: &Vector3<f64>,
        start: &Vector3<f64>,
        end: &Vector3<f64>,
        epsilon: f64,
    ) -> Option<Contact> {
        let v = end - start;
        let t = (point - start).dot(&v).clamp(0.0, 1.0);

        let q = start + t * v;

        let dist = (point - q).norm();

        if dist >= epsilon {
            return None;
        }

        let normal = UnitVector3::new_normalize(point - q);

        Some(Contact {
            point: q,
            normal,
            depth: dist,
        })
    }

    pub fn edge_edge(
        start1: &Vector3<f64>,
        end1: &Vector3<f64>,
        start2: &Vector3<f64>,
        end2: &Vector3<f64>,
        epsilon: f64,
    ) -> Option<Contact> {
        let v = end1 - start1;
        let w = end2 - start2;

        let v_norm_squared = v.norm_squared();
        let w_norm_squared = w.norm_squared();

        if v_norm_squared <= f64::EPSILON && w_norm_squared <= f64::EPSILON {
            let dist = (start2 - start1).norm();
            if dist >= epsilon {
                return None;
            }

            return Some(Contact {
                point: *start1,
                normal: UnitVector3::new_normalize(start2 - start1),
                depth: dist,
            });
        }

        if v_norm_squared <= f64::EPSILON {
            return Self::point_edge(start1, start2, end2, epsilon);
        } else if w_norm_squared <= f64::EPSILON {
            return Self::point_edge(start2, start1, end1, epsilon);
        }

        let r = start1 - start2;
        let f = w.dot(&r);
        let c = v.dot(&r);
        let b = v.dot(&w);

        let denom = v_norm_squared * w_norm_squared - b * b;

        let mut s = if denom != 0.0 {
            ((b * f - c * w_norm_squared) / denom).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let mut t = (b * s + f) / w_norm_squared;

        if t < 0.0 {
            t = 0.0;
            s = (-c / v_norm_squared).clamp(0.0, 1.0);
        } else if t > 1.0 {
            t = 1.0;
            s = ((b - c) / v_norm_squared).clamp(0.0, 1.0);
        }

        let c1 = start1 + s * v;
        let c2 = start2 + t * w;

        let dist = (c2 - c1).norm();

        if dist >= epsilon {
            return None;
        }

        Some(Contact {
            point: c2,
            normal: UnitVector3::new_normalize(c2 - c1),
            depth: dist,
        })
    }
}
