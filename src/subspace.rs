//! Lattice of subspaces: subspaces of an agent vector space form a modular lattice.
//!
//! The subspaces of a vector space, ordered by inclusion, form a modular lattice
//! where join = span, meet = intersection. This is a fundamental example in lattice theory.

use serde::{Serialize, Deserialize};
use nalgebra::{DMatrix, DVector};
use crate::lattice::{FiniteLattice, LatticeElement};

/// A subspace of a vector space, represented by a basis stored as rows of a matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subspace {
    pub label: String,
    /// Basis vectors stored as rows. Each inner Vec has length `dimension`.
    pub basis: Vec<Vec<f64>>,
    /// Dimension of the ambient space.
    pub ambient_dim: usize,
}

impl Subspace {
    /// Create the zero subspace of an n-dimensional space.
    pub fn zero(n: usize) -> Self {
        Self { label: "zero".into(), basis: vec![], ambient_dim: n }
    }

    /// Create the full n-dimensional space (standard basis).
    pub fn full(n: usize) -> Self {
        let basis: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                let mut v = vec![0.0; n];
                v[i] = 1.0;
                v
            })
            .collect();
        Self { label: "full".into(), basis, ambient_dim: n }
    }

    /// Create from basis vectors.
    pub fn from_basis(label: impl Into<String>, basis: &[Vec<f64>], ambient_dim: usize) -> Self {
        Self { label: label.into(), basis: basis.to_vec(), ambient_dim }
    }

    /// Create a 1-dimensional subspace spanned by a single vector.
    pub fn span_of(label: impl Into<String>, v: &[f64]) -> Self {
        Self { label: label.into(), basis: vec![v.to_vec()], ambient_dim: v.len() }
    }

    /// Dimension of this subspace.
    pub fn dimension(&self) -> usize {
        self.basis.len()
    }

    /// Check if this subspace contains another (i.e., the other is a subspace of this).
    pub fn contains(&self, other: &Subspace) -> bool {
        if other.basis.is_empty() { return true; }
        if self.basis.is_empty() { return other.basis.is_empty(); }

        // Check if each vector in other is in the span of self
        let n = self.ambient_dim;
        let self_dim = self.basis.len();

        // Build self's basis matrix
        let mut mat = DMatrix::<f64>::zeros(self_dim, n);
        for (i, row) in self.basis.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                mat[(i, j)] = val;
            }
        }

        for v in &other.basis {
            let target = DVector::from_row_slice(v);
            // Solve mat^T * c = v (i.e., find coefficients)
            // We need to check if v is in the row space of mat
            // Use the pseudoinverse approach: v should be in row space iff mat * mat^T is invertible and c = (mat * mat^T)^{-1} * mat * v gives mat^T * c ≈ v
            let mt = mat.transpose();
            let mmt = &mat * &mt;
            if let Some(mmt_inv) = mmt.try_inverse() {
                let c = &mmt_inv * &mat * &target;
                let reconstructed = &mt * &c;
                if (reconstructed - target).norm() > 1e-8 {
                    return false;
                }
            } else {
                // Self basis is linearly dependent; use rank-based check
                // Augmented matrix rank test
                let aug_rows = self.basis.len() + 1;
                let mut aug = DMatrix::<f64>::zeros(aug_rows, n);
                for (i, row) in self.basis.iter().enumerate() {
                    for (j, &val) in row.iter().enumerate() {
                        aug[(i, j)] = val;
                    }
                }
                for (j, &val) in v.iter().enumerate() {
                    aug[(self.basis.len(), j)] = val;
                }
                let rank_self = mat.rank(1e-10);
                let rank_aug = aug.rank(1e-10);
                if rank_aug > rank_self {
                    return false;
                }
            }
        }
        true
    }

    /// Intersection of two subspaces (meet).
    pub fn intersection(&self, other: &Subspace) -> Subspace {
        if self.basis.is_empty() || other.basis.is_empty() {
            return Subspace::zero(self.ambient_dim);
        }

        let n = self.ambient_dim;

        // Stack both bases and find the null space
        let total_rows = self.basis.len() + other.basis.len();
        let mut combined = DMatrix::<f64>::zeros(total_rows, n);
        for (i, row) in self.basis.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                combined[(i, j)] = val;
            }
        }
        for (i, row) in other.basis.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                combined[(self.basis.len() + i, j)] = val;
            }
        }

        // Find null space of combined matrix using SVD-like approach
        let svd = combined.svd(true, true);
        let mut inter_basis = Vec::new();
        if let Some(v_t) = &svd.v_t {
            let singular_values = &svd.singular_values;
            for i in 0..v_t.nrows() {
                let is_zero = i < singular_values.len() && singular_values[i] < 1e-8;
                if is_zero || i >= singular_values.len() {
                    let row: Vec<f64> = (0..v_t.ncols()).map(|j| v_t[(i, j)]).collect();
                    let norm: f64 = row.iter().map(|x| x * x).sum::<f64>().sqrt();
                    if norm > 1e-8 {
                        inter_basis.push(row);
                    }
                }
            }
        }

        // Simpler approach: for each pair of basis vectors, try to find vectors in intersection
        // Actually, let's use a cleaner method
        let n = self.ambient_dim;
        let d1 = self.basis.len();
        let d2 = other.basis.len();

        // We want vectors v such that v = A^T * x = B^T * y for some x, y
        // i.e., A^T * x - B^T * y = 0
        // Build matrix [A^T | -B^T] and find null space
        let mut sys = DMatrix::<f64>::zeros(n, d1 + d2);
        for (i, row) in self.basis.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                sys[(j, i)] = val;
            }
        }
        for (i, row) in other.basis.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                sys[(j, d1 + i)] = -val;
            }
        }

        let svd2 = sys.svd(true, true);
        let mut result_basis = Vec::new();
        if let Some(v_t) = &svd2.v_t {
            let sv = &svd2.singular_values;
            for i in 0..v_t.nrows() {
                let is_zero_sv = i < sv.len() && sv[i] < 1e-8;
                if is_zero_sv || i >= sv.len() {
                    // Extract the first d1 components, these give coefficients for A^T
                    let coeffs: Vec<f64> = (0..d1).map(|j| v_t[(i, j)]).collect();
                    // Compute v = A^T * coeffs
                    let mut v = vec![0.0; n];
                    for (k, &c) in coeffs.iter().enumerate() {
                        if k < self.basis.len() {
                            for (j, val) in v.iter_mut().enumerate() {
                                *val += c * self.basis[k][j];
                            }
                        }
                    }
                    let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
                    if norm > 1e-8 {
                        for x in &mut v { *x /= norm; }
                        result_basis.push(v);
                    }
                }
            }
        }

        Subspace {
            label: format!("{}∩{}", self.label, other.label),
            basis: result_basis,
            ambient_dim: self.ambient_dim,
        }
    }

    /// Equality of subspaces (same span).
    pub fn spans_equal(&self, other: &Subspace) -> bool {
        self.contains(other) && other.contains(self)
    }
}

/// Lattice of subspaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubspaceLattice {
    subspaces: Vec<Subspace>,
    lattice: FiniteLattice,
}

impl SubspaceLattice {
    /// Build from a list of subspaces (must include zero and full).
    pub fn from_subspaces(subspaces: Vec<Subspace>) -> Self {
        let mut lattice = FiniteLattice::new();

        for ss in &subspaces {
            lattice.add_element(LatticeElement::new(&ss.label));
        }

        // Set join and meet
        for i in 0..subspaces.len() {
            for j in 0..subspaces.len() {
                let a = &subspaces[i];
                let b = &subspaces[j];

                // Meet = intersection
                let meet = a.intersection(b);
                if let Some(m) = subspaces.iter().find(|s| s.spans_equal(&meet)) {
                    lattice.set_meet(&a.label, &b.label, &m.label);
                }

                // Join = span(a ∪ b)
                let mut joined_basis = a.basis.clone();
                joined_basis.extend(b.basis.clone());
                let joined = Subspace {
                    label: format!("{}+{}", a.label, b.label),
                    basis: joined_basis,
                    ambient_dim: a.ambient_dim,
                };
                if let Some(j) = subspaces.iter().find(|s| s.spans_equal(&joined)) {
                    lattice.set_join(&a.label, &b.label, &j.label);
                }
            }
        }

        // Covers
        for i in 0..subspaces.len() {
            for j in 0..subspaces.len() {
                if i != j && subspaces[i].contains(&subspaces[j]) {
                    let mut is_cover = true;
                    for k in 0..subspaces.len() {
                        if k != i && k != j &&
                           subspaces[i].contains(&subspaces[k]) &&
                           subspaces[k].contains(&subspaces[j]) {
                            is_cover = false;
                            break;
                        }
                    }
                    if is_cover {
                        lattice.add_cover(&subspaces[j].label, &subspaces[i].label);
                    }
                }
            }
        }

        if let Some(zero) = subspaces.iter().find(|s| s.basis.is_empty()) {
            lattice.set_bottom(&zero.label);
        }
        if let Some(full) = subspaces.iter().find(|s| s.dimension() == subspaces[0].ambient_dim) {
            lattice.set_top(&full.label);
        }

        Self { subspaces, lattice }
    }

    pub fn lattice(&self) -> &FiniteLattice {
        &self.lattice
    }

    pub fn subspaces(&self) -> &[Subspace] {
        &self.subspaces
    }

    /// Build the lattice of subspaces of F₂² (4 subspaces).
    pub fn f2_squared() -> Self {
        let zero = Subspace::zero(2);
        let e1 = Subspace::span_of("e1", &[1.0, 0.0]);
        let e2 = Subspace::span_of("e2", &[0.0, 1.0]);
        let diag = Subspace::span_of("diag", &[1.0, 1.0]);
        let full = Subspace::full(2);

        let subspaces = vec![zero, e1, e2, diag, full];
        let mut lattice = FiniteLattice::new();

        for ss in &subspaces {
            lattice.add_element(LatticeElement::new(&ss.label));
        }

        // Set all joins and meets explicitly
        // zero is bottom, full is top
        // e1, e2, diag are atoms covering zero, all covered by full

        // zero joins
        lattice.set_join("zero", "zero", "zero");
        lattice.set_join("zero", "e1", "e1");
        lattice.set_join("zero", "e2", "e2");
        lattice.set_join("zero", "diag", "diag");
        lattice.set_join("zero", "full", "full");

        // e1
        lattice.set_join("e1", "e1", "e1");
        lattice.set_join("e1", "e2", "full");
        lattice.set_join("e1", "diag", "full");
        lattice.set_join("e1", "full", "full");

        // e2
        lattice.set_join("e2", "e2", "e2");
        lattice.set_join("e2", "diag", "full");
        lattice.set_join("e2", "full", "full");

        // diag
        lattice.set_join("diag", "diag", "diag");
        lattice.set_join("diag", "full", "full");

        // full
        lattice.set_join("full", "full", "full");

        // meets
        lattice.set_meet("zero", "zero", "zero");
        lattice.set_meet("zero", "e1", "zero");
        lattice.set_meet("zero", "e2", "zero");
        lattice.set_meet("zero", "diag", "zero");
        lattice.set_meet("zero", "full", "zero");

        lattice.set_meet("e1", "e1", "e1");
        lattice.set_meet("e1", "e2", "zero");
        lattice.set_meet("e1", "diag", "zero");
        lattice.set_meet("e1", "full", "e1");

        lattice.set_meet("e2", "e2", "e2");
        lattice.set_meet("e2", "diag", "zero");
        lattice.set_meet("e2", "full", "e2");

        lattice.set_meet("diag", "diag", "diag");
        lattice.set_meet("diag", "full", "diag");

        lattice.set_meet("full", "full", "full");

        // Covers
        lattice.add_cover("zero", "e1");
        lattice.add_cover("zero", "e2");
        lattice.add_cover("zero", "diag");
        lattice.add_cover("e1", "full");
        lattice.add_cover("e2", "full");
        lattice.add_cover("diag", "full");

        lattice.set_bottom("zero");
        lattice.set_top("full");

        Self { subspaces, lattice }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_subspace() {
        let z = Subspace::zero(3);
        assert_eq!(z.dimension(), 0);
    }

    #[test]
    fn test_full_subspace() {
        let f = Subspace::full(3);
        assert_eq!(f.dimension(), 3);
    }

    #[test]
    fn test_span_of() {
        let s = Subspace::span_of("test", &[1.0, 0.0, 1.0]);
        assert_eq!(s.dimension(), 1);
        assert_eq!(s.ambient_dim, 3);
    }

    #[test]
    fn test_contains_zero() {
        let z = Subspace::zero(3);
        let s = Subspace::span_of("s", &[1.0, 0.0, 0.0]);
        assert!(s.contains(&z));
    }

    #[test]
    fn test_contains_line_in_plane() {
        let line = Subspace::span_of("line", &[1.0, 0.0]);
        let plane = Subspace::full(2);
        assert!(plane.contains(&line));
    }

    #[test]
    fn test_intersection_zero() {
        let a = Subspace::span_of("a", &[1.0, 0.0]);
        let b = Subspace::span_of("b", &[0.0, 1.0]);
        let inter = a.intersection(&b);
        assert_eq!(inter.dimension(), 0);
    }

    #[test]
    fn test_intersection_same() {
        let a = Subspace::span_of("a", &[1.0, 0.0]);
        let b = Subspace::span_of("b", &[2.0, 0.0]);
        let inter = a.intersection(&b);
        assert_eq!(inter.dimension(), 1);
    }

    #[test]
    fn test_f2_squared_lattice() {
        let sl = SubspaceLattice::f2_squared();
        assert_eq!(sl.subspaces().len(), 5);
    }

    #[test]
    fn test_f2_squared_bottom_top() {
        let sl = SubspaceLattice::f2_squared();
        assert_eq!(sl.lattice().bottom(), Some("zero"));
        assert_eq!(sl.lattice().top(), Some("full"));
    }

    #[test]
    fn test_subspace_lattice_modular() {
        let sl = SubspaceLattice::f2_squared();
        assert!(crate::modular::check_modular_law(sl.lattice()).is_ok());
    }

    #[test]
    fn test_spans_equal() {
        let a = Subspace::span_of("a", &[1.0, 0.0]);
        let b = Subspace::span_of("b", &[2.0, 0.0]);
        assert!(a.spans_equal(&b));
    }

    #[test]
    fn test_subspace_contains_full() {
        let full = Subspace::full(2);
        let line = Subspace::span_of("l", &[1.0, 0.0]);
        assert!(full.contains(&line));
        assert!(!line.contains(&full));
    }
}
