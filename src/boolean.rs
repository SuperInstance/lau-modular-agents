//! Boolean algebra: complemented distributive lattice.
//!
//! A Boolean algebra is a distributive lattice with a complement operation
//! satisfying a ∨ ¬a = 1 and a ∧ ¬a = 0.

use serde::{Serialize, Deserialize};
use crate::lattice::{FiniteLattice, powerset_lattice};

/// Complement information for an element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Complement {
    pub element: String,
    pub complement: String,
}

/// A Boolean algebra: a complemented distributive lattice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BooleanAlgebra {
    lattice: FiniteLattice,
    complements: Vec<Complement>,
}

impl BooleanAlgebra {
    /// Construct from a finite lattice, verifying it forms a Boolean algebra.
    pub fn new(lattice: FiniteLattice) -> Result<Self, String> {
        // First check distributive
        crate::distributive::check_distributive_law(&lattice)
            .map_err(|e| format!("Not distributive: {:?}", e))?;

        let top = lattice.top().ok_or("No top element")?.to_string();
        let bottom = lattice.bottom().ok_or("No bottom element")?.to_string();

        let labels: Vec<&str> = lattice.elements().iter().map(|e| e.label.as_str()).collect();

        let mut complements = Vec::new();
        for &a in &labels {
            let mut found = Vec::new();
            for &b in &labels {
                if let (Some(j), Some(m)) = (lattice.join(a, b), lattice.meet(a, b)) {
                    if j == top && m == bottom {
                        found.push(b.to_string());
                    }
                }
            }
            if found.len() != 1 {
                return Err(format!("Element {} has {} complements (need exactly 1)", a, found.len()));
            }
            complements.push(Complement {
                element: a.to_string(),
                complement: found[0].clone(),
            });
        }

        Ok(Self { lattice, complements })
    }

    /// Create a Boolean algebra from a powerset of size n.
    pub fn powerset(n: usize) -> Result<Self, String> {
        Self::new(powerset_lattice(n))
    }

    pub fn inner(&self) -> &FiniteLattice {
        &self.lattice
    }

    /// Get the complement of an element.
    pub fn complement(&self, a: &str) -> Option<&str> {
        self.complements.iter()
            .find(|c| c.element == a)
            .map(|c| c.complement.as_str())
    }

    /// De Morgan's laws: ¬(a ∨ b) = ¬a ∧ ¬b and ¬(a ∧ b) = ¬a ∨ ¬b.
    pub fn verify_de_morgan(&self) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();
        let labels: Vec<&str> = self.lattice.elements().iter().map(|e| e.label.as_str()).collect();

        for &a in &labels {
            for &b in &labels {
                let na = match self.complement(a) {
                    Some(c) => c,
                    None => continue,
                };
                let nb = match self.complement(b) {
                    Some(c) => c,
                    None => continue,
                };

                // ¬(a ∨ b) = ¬a ∧ ¬b
                if let (Some(a_join_b), Some(na_meet_nb)) = (self.lattice.join(a, b), self.lattice.meet(na, nb)) {
                    if let Some(n_a_join_b) = self.complement(a_join_b) {
                        if n_a_join_b != na_meet_nb {
                            violations.push(format!(
                                "De Morgan 1 fail: ¬({}∨{})={} ≠ ¬{}∧¬{}={}",
                                a, b, n_a_join_b, a, b, na_meet_nb
                            ));
                        }
                    }
                }

                // ¬(a ∧ b) = ¬a ∨ ¬b
                if let (Some(a_meet_b), Some(na_join_nb)) = (self.lattice.meet(a, b), self.lattice.join(na, nb)) {
                    if let Some(n_a_meet_b) = self.complement(a_meet_b) {
                        if n_a_meet_b != na_join_nb {
                            violations.push(format!(
                                "De Morgan 2 fail: ¬({}∧{})={} ≠ ¬{}∨¬{}={}",
                                a, b, n_a_meet_b, a, b, na_join_nb
                            ));
                        }
                    }
                }
            }
        }

        if violations.is_empty() { Ok(()) } else { Err(violations) }
    }

    /// Involution: ¬(¬a) = a.
    pub fn verify_involution(&self) -> bool {
        for c in &self.complements {
            match self.complement(&c.complement) {
                Some(cc) if cc == c.element => {}
                _ => return false,
            }
        }
        true
    }

    /// The number of atoms (elements that cover bottom).
    pub fn atom_count(&self) -> usize {
        let bottom = match self.lattice.bottom() {
            Some(b) => b,
            None => return 0,
        };
        self.lattice.covers_of(bottom).len()
    }

    /// Number of elements.
    pub fn size(&self) -> usize {
        self.lattice.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_powerset_1() {
        let ba = BooleanAlgebra::powerset(1).unwrap();
        assert_eq!(ba.size(), 2);
        assert!(ba.verify_de_morgan().is_ok());
        assert!(ba.verify_involution());
    }

    #[test]
    fn test_boolean_powerset_2() {
        let ba = BooleanAlgebra::powerset(2).unwrap();
        assert_eq!(ba.size(), 4);
        assert!(ba.verify_de_morgan().is_ok());
    }

    #[test]
    fn test_boolean_powerset_3() {
        let ba = BooleanAlgebra::powerset(3).unwrap();
        assert_eq!(ba.size(), 8);
        assert!(ba.verify_de_morgan().is_ok());
    }

    #[test]
    fn test_boolean_complement_involution() {
        let ba = BooleanAlgebra::powerset(3).unwrap();
        assert!(ba.verify_involution());
    }

    #[test]
    fn test_boolean_complement_specific() {
        let ba = BooleanAlgebra::powerset(2).unwrap();
        // In powerset of {a,b}, complement of {a} is {b}
        let comp_a = ba.complement("a").unwrap();
        assert_eq!(comp_a, "b");
    }

    #[test]
    fn test_boolean_atom_count() {
        let ba = BooleanAlgebra::powerset(3).unwrap();
        assert_eq!(ba.atom_count(), 3);
    }

    #[test]
    fn test_boolean_atom_count_2() {
        let ba = BooleanAlgebra::powerset(2).unwrap();
        assert_eq!(ba.atom_count(), 2);
    }

    #[test]
    fn test_boolean_chain_not_boolean() {
        let chain = crate::lattice::chain_lattice(3);
        let result = BooleanAlgebra::new(chain);
        assert!(result.is_err());
    }

    #[test]
    fn test_de_morgan_laws() {
        let ba = BooleanAlgebra::powerset(3).unwrap();
        assert!(ba.verify_de_morgan().is_ok());
    }

    #[test]
    fn test_boolean_size_is_power_of_2() {
        for n in 1..=4 {
            let ba = BooleanAlgebra::powerset(n).unwrap();
            assert_eq!(ba.size(), 1 << n);
        }
    }
}
