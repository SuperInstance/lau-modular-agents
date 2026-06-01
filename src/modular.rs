//! Modular lattice: if a ≤ b then a ∨ (x ∧ b) = (a ∨ x) ∧ b.
//!
//! A lattice is modular iff it does NOT contain the pentagon N₅ as a sublattice.
//! Dedekind's theorem: a lattice is modular iff the modular law holds.

use crate::lattice::FiniteLattice;

/// Check if a lattice satisfies the modular law for all triples.
pub fn check_modular_law(lattice: &FiniteLattice) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    let labels: Vec<&str> = lattice.elements().iter().map(|e| e.label.as_str()).collect();

    for &a in &labels {
        for &b in &labels {
            for &x in &labels {
                if lattice.leq(a, b) {
                    // Modular law: a ∨ (x ∧ b) = (a ∨ x) ∧ b
                    if let (Some(x_meet_b), Some(a_join_x)) = (lattice.meet(x, b), lattice.join(a, x)) {
                        if let (Some(lhs), Some(rhs)) = (lattice.join(a, x_meet_b), lattice.meet(a_join_x, b)) {
                            if lhs != rhs {
                                violations.push(format!(
                                    "Modular law violation: a={}, b={}, x={} | a∨(x∧b)={} ≠ (a∨x)∧b={}",
                                    a, b, x, lhs, rhs
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    if violations.is_empty() { Ok(()) } else { Err(violations) }
}

/// A lattice that has been verified to be modular.
#[derive(Debug, Clone)]
pub struct ModularLattice {
    lattice: FiniteLattice,
}

impl ModularLattice {
    /// Create a ModularLattice, verifying the modular law.
    pub fn new(lattice: FiniteLattice) -> Result<Self, Vec<String>> {
        check_modular_law(&lattice)?;
        Ok(Self { lattice })
    }

    /// Create without verification (use when known to be modular).
    pub fn new_unchecked(lattice: FiniteLattice) -> Self {
        Self { lattice }
    }

    pub fn inner(&self) -> &FiniteLattice {
        &self.lattice
    }

    pub fn into_inner(self) -> FiniteLattice {
        self.lattice
    }

    /// The diamond isomorphism theorem for modular lattices:
    /// If a ≤ b, then [a, b] ≅ [a ∨ x, b ∨ x] via the map y ↦ y ∨ x.
    pub fn diamond_isomorphism(&self, a: &str, b: &str, x: &str) -> Vec<(String, String)> {
        let mut iso = Vec::new();
        let labels: Vec<&str> = self.lattice.elements().iter().map(|e| e.label.as_str()).collect();

        if !self.lattice.leq(a, b) {
            return iso;
        }

        for &y in &labels {
            if self.lattice.leq(a, y) && self.lattice.leq(y, b) {
                if let Some(mapped) = self.lattice.join(y, x) {
                    iso.push((y.to_string(), mapped.to_string()));
                }
            }
        }
        iso
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lattice::{chain_lattice, diamond_m3_lattice, pentagon_n5_lattice, powerset_lattice};

    #[test]
    fn test_chain_is_modular() {
        let chain = chain_lattice(5);
        assert!(check_modular_law(&chain).is_ok());
    }

    #[test]
    fn test_diamond_m3_is_modular() {
        let m3 = diamond_m3_lattice();
        assert!(check_modular_law(&m3).is_ok());
    }

    #[test]
    fn test_pentagon_n5_not_modular() {
        let n5 = pentagon_n5_lattice();
        assert!(check_modular_law(&n5).is_err());
    }

    #[test]
    fn test_modular_lattice_construction() {
        let chain = chain_lattice(3);
        let modular = ModularLattice::new(chain);
        assert!(modular.is_ok());
    }

    #[test]
    fn test_modular_lattice_rejects_n5() {
        let n5 = pentagon_n5_lattice();
        let modular = ModularLattice::new(n5);
        assert!(modular.is_err());
    }

    #[test]
    fn test_powerset_is_modular() {
        // Powerset lattice is distributive, hence modular
        let p = powerset_lattice(3);
        assert!(check_modular_law(&p).is_ok());
    }

    #[test]
    fn test_diamond_isomorphism_theorem() {
        let chain = chain_lattice(4);
        let modular = ModularLattice::new(chain).unwrap();
        let iso = modular.diamond_isomorphism("c0", "c3", "c1");
        // [c0, c3] maps via ∨c1 to [c1, c3]
        assert!(!iso.is_empty());
    }

    #[test]
    fn test_modular_law_chain_specific() {
        // In a chain, modular law always holds: if a ≤ b, then a∨(x∧b) = (a∨x)∧b
        let chain = chain_lattice(6);
        assert!(check_modular_law(&chain).is_ok());
    }

    #[test]
    fn test_modular_law_violation_details() {
        let n5 = pentagon_n5_lattice();
        let result = check_modular_law(&n5);
        assert!(result.is_err());
        let violations = result.unwrap_err();
        assert!(!violations.is_empty());
        // The violation should mention the modular law
        assert!(violations[0].contains("Modular law violation"));
    }
}
