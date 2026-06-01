//! Distributive lattice: a ∧ (b ∨ c) = (a ∧ b) ∨ (a ∧ c).
//!
//! A lattice is distributive iff it contains neither the pentagon N₅ nor the diamond M₃
//! as a sublattice.

use crate::lattice::FiniteLattice;

/// Check if a lattice satisfies the distributive law for all triples.
pub fn check_distributive_law(lattice: &FiniteLattice) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    let labels: Vec<&str> = lattice.elements().iter().map(|e| e.label.as_str()).collect();

    for &a in &labels {
        for &b in &labels {
            for &c in &labels {
                // Distributive: a ∧ (b ∨ c) = (a ∧ b) ∨ (a ∧ c)
                if let (Some(b_join_c), Some(a_meet_b), Some(a_meet_c)) =
                    (lattice.join(b, c), lattice.meet(a, b), lattice.meet(a, c))
                {
                    if let (Some(lhs), Some(rhs)) =
                        (lattice.meet(a, b_join_c), lattice.join(a_meet_b, a_meet_c))
                    {
                        if lhs != rhs {
                            violations.push(format!(
                                "Distributive law violation: a={}, b={}, c={} | a∧(b∨c)={} ≠ (a∧b)∨(a∧c)={}",
                                a, b, c, lhs, rhs
                            ));
                        }
                    }
                }
            }
        }
    }

    if violations.is_empty() { Ok(()) } else { Err(violations) }
}

/// A lattice verified to be distributive.
#[derive(Debug, Clone)]
pub struct DistributiveLattice {
    lattice: FiniteLattice,
}

impl DistributiveLattice {
    pub fn new(lattice: FiniteLattice) -> Result<Self, Vec<String>> {
        check_distributive_law(&lattice)?;
        Ok(Self { lattice })
    }

    pub fn new_unchecked(lattice: FiniteLattice) -> Self {
        Self { lattice }
    }

    pub fn inner(&self) -> &FiniteLattice {
        &self.lattice
    }

    /// Check if the lattice is also a Boolean algebra (complemented distributive).
    /// Returns true if every element has a unique complement.
    pub fn is_boolean_algebra(&self) -> bool {
        let top = match self.lattice.top() {
            Some(t) => t,
            None => return false,
        };
        let bottom = match self.lattice.bottom() {
            Some(b) => b,
            None => return false,
        };

        let labels: Vec<&str> = self.lattice.elements().iter().map(|e| e.label.as_str()).collect();

        for &a in &labels {
            let mut complements = Vec::new();
            for &b in &labels {
                if let (Some(j), Some(m)) = (self.lattice.join(a, b), self.lattice.meet(a, b)) {
                    if j == top && m == bottom {
                        complements.push(b);
                    }
                }
            }
            if complements.len() != 1 {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lattice::{chain_lattice, diamond_m3_lattice, pentagon_n5_lattice, powerset_lattice};

    #[test]
    fn test_chain_is_distributive() {
        let chain = chain_lattice(5);
        assert!(check_distributive_law(&chain).is_ok());
    }

    #[test]
    fn test_powerset_is_distributive() {
        let p = powerset_lattice(3);
        assert!(check_distributive_law(&p).is_ok());
    }

    #[test]
    fn test_diamond_m3_not_distributive() {
        let m3 = diamond_m3_lattice();
        assert!(check_distributive_law(&m3).is_err());
    }

    #[test]
    fn test_pentagon_n5_not_distributive() {
        let n5 = pentagon_n5_lattice();
        assert!(check_distributive_law(&n5).is_err());
    }

    #[test]
    fn test_distributive_construction() {
        let p = powerset_lattice(2);
        let dist = DistributiveLattice::new(p);
        assert!(dist.is_ok());
    }

    #[test]
    fn test_distributive_rejects_m3() {
        let m3 = diamond_m3_lattice();
        let dist = DistributiveLattice::new(m3);
        assert!(dist.is_err());
    }

    #[test]
    fn test_chain_distributive_lattice() {
        let chain = chain_lattice(4);
        let dist = DistributiveLattice::new(chain).unwrap();
        // Chain is not a Boolean algebra (elements lack complements except top/bottom)
        assert!(!dist.is_boolean_algebra());
    }

    #[test]
    fn test_powerset_is_boolean() {
        let p = powerset_lattice(3);
        let dist = DistributiveLattice::new(p).unwrap();
        assert!(dist.is_boolean_algebra());
    }

    #[test]
    fn test_powerset_2_is_boolean() {
        let p = powerset_lattice(2);
        let dist = DistributiveLattice::new(p).unwrap();
        assert!(dist.is_boolean_algebra());
    }

    #[test]
    fn test_distributive_law_violation_count() {
        let m3 = diamond_m3_lattice();
        let result = check_distributive_law(&m3);
        assert!(result.is_err());
        assert!(!result.unwrap_err().is_empty());
    }
}
