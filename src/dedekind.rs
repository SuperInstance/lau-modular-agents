//! Modular law and Dedekind's theorem.
//!
//! The modular law: if a ≤ b then a ∨ (x ∧ b) = (a ∨ x) ∧ b.
//!
//! Dedekind's theorem: A lattice is modular if and only if it does not contain
//! the pentagon N₅ as a sublattice.
//!
//! For subgroup lattices specifically, Dedekind proved that the lattice of
//! normal subgroups of any group is always modular.

use crate::lattice::FiniteLattice;

/// Verify the modular law for a specific triple (a, x, b) where a ≤ b.
pub fn dedekind_modular_law(lattice: &FiniteLattice, a: &str, x: &str, b: &str) -> Result<bool, String> {
    if !lattice.leq(a, b) {
        return Err(format!("{} is not ≤ {}", a, b));
    }

    let lhs = lattice.join(a, lattice.meet(x, b).ok_or(format!("No meet for {},{}", x, b))?)
        .ok_or(format!("No join for {},{}", a, lattice.meet(x, b).unwrap()))?;

    let rhs = lattice.meet(lattice.join(a, x).ok_or(format!("No join for {},{}", a, x))?, b)
        .ok_or(format!("No meet for {},{}", lattice.join(a, x).unwrap(), b))?;

    Ok(lhs == rhs)
}

/// Check if N₅ (the pentagon) is embedded in the lattice as a sublattice.
/// If N₅ is found, the lattice is NOT modular (by Dedekind's theorem).
pub fn contains_pentagon_n5(lattice: &FiniteLattice) -> Option<Vec<String>> {
    let labels: Vec<&str> = lattice.elements().iter().map(|e| e.label.as_str()).collect();

    // N₅: bot < a < c < top, bot < b < top, b incomparable to a and c
    // Key property: a ≤ c, but a ∨ (b ∧ c) ≠ (a ∨ b) ∧ c
    for &bot in &labels {
        for &a in &labels {
            if a == bot { continue; }
            for &c in &labels {
                if c == bot || c == a { continue; }
                if !lattice.leq(a, c) { continue; }

                for &b in &labels {
                    if b == bot || b == a || b == c { continue; }

                    // Check N₅ pattern: bot ≤ a ≤ c, bot ≤ b, b incomparable to a and c
                    if !lattice.leq(bot, b) { continue; }
                    if lattice.leq(a, b) || lattice.leq(b, a) { continue; }
                    if lattice.leq(c, b) || lattice.leq(b, c) { continue; }

                    // Check the modular law violation
                    if let (Some(x_and_c), Some(a_or_x)) = (lattice.meet(b, c), lattice.join(a, b)) {
                        if let (Some(lhs), Some(rhs)) = (lattice.join(a, x_and_c), lattice.meet(a_or_x, c)) {
                            if lhs != rhs {
                                return Some(vec![bot.to_string(), a.to_string(), b.to_string(), c.to_string()]);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Verify Dedekind's theorem: the lattice is modular iff it doesn't contain N₅.
pub fn verify_dedekind_theorem(lattice: &FiniteLattice) -> DedekindResult {
    let modular_check = crate::modular::check_modular_law(lattice);
    let has_pentagon = contains_pentagon_n5(lattice);

    let is_modular = modular_check.is_ok();
    let contains_pent = has_pentagon.is_some();

    DedekindResult {
        is_modular,
        contains_pentagon: contains_pent,
        pentagon_witness: has_pentagon,
        modular_violations: modular_check.err().unwrap_or_default(),
        theorem_holds: is_modular != contains_pent,
    }
}

/// Result of verifying Dedekind's theorem.
#[derive(Debug, Clone)]
pub struct DedekindResult {
    pub is_modular: bool,
    pub contains_pentagon: bool,
    pub pentagon_witness: Option<Vec<String>>,
    pub modular_violations: Vec<String>,
    pub theorem_holds: bool,
}

/// Verify the modular identity for all elements.
/// The modular identity is: x ∧ (y ∨ (x ∧ z)) = (x ∧ y) ∨ (x ∧ z)
/// This is equivalent to the modular law.
pub fn verify_modular_identity(lattice: &FiniteLattice) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    let labels: Vec<&str> = lattice.elements().iter().map(|e| e.label.as_str()).collect();

    for &x in &labels {
        for &y in &labels {
            for &z in &labels {
                // LHS: x ∧ (y ∨ (x ∧ z))
                if let (Some(_x_and_z), Some(y_or_xz)) = (lattice.meet(x, z), lattice.meet(x, z).and_then(|xz| lattice.join(y, xz))) {
                    if let Some(lhs) = lattice.meet(x, y_or_xz) {
                        // RHS: (x ∧ y) ∨ (x ∧ z)
                        if let (Some(x_and_y), Some(x_and_z2)) = (lattice.meet(x, y), lattice.meet(x, z)) {
                            if let Some(rhs) = lattice.join(x_and_y, x_and_z2) {
                                if lhs != rhs {
                                    violations.push(format!(
                                        "Modular identity fail: x={}, y={}, z={} | {} ≠ {}",
                                        x, y, z, lhs, rhs
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if violations.is_empty() { Ok(()) } else { Err(violations) }
}

/// Shearing identity (another form of the modular law):
/// If a ≤ b then (a ∨ x) ∧ b = a ∨ (x ∧ b).
pub fn verify_shearing_identity(lattice: &FiniteLattice) -> Result<(), Vec<String>> {
    crate::modular::check_modular_law(lattice)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lattice::{chain_lattice, diamond_m3_lattice, pentagon_n5_lattice, powerset_lattice};

    #[test]
    fn test_dedekind_modular_law_chain() {
        let chain = chain_lattice(4);
        assert!(dedekind_modular_law(&chain, "c0", "c2", "c3").unwrap());
    }

    #[test]
    fn test_dedekind_modular_law_diamond() {
        let m3 = diamond_m3_lattice();
        assert!(dedekind_modular_law(&m3, "bot", "a", "top").unwrap());
    }

    #[test]
    fn test_dedekind_pentagon_not_in_chain() {
        let chain = chain_lattice(5);
        assert!(contains_pentagon_n5(&chain).is_none());
    }

    #[test]
    fn test_dedekind_pentagon_not_in_diamond() {
        let m3 = diamond_m3_lattice();
        assert!(contains_pentagon_n5(&m3).is_none());
    }

    #[test]
    fn test_dedekind_pentagon_in_n5() {
        let n5 = pentagon_n5_lattice();
        assert!(contains_pentagon_n5(&n5).is_some());
    }

    #[test]
    fn test_dedekind_theorem_chain() {
        let chain = chain_lattice(4);
        let result = verify_dedekind_theorem(&chain);
        assert!(result.is_modular);
        assert!(!result.contains_pentagon);
        assert!(result.theorem_holds);
    }

    #[test]
    fn test_dedekind_theorem_n5() {
        let n5 = pentagon_n5_lattice();
        let result = verify_dedekind_theorem(&n5);
        assert!(!result.is_modular);
        assert!(result.contains_pentagon);
        assert!(result.theorem_holds);
    }

    #[test]
    fn test_dedekind_theorem_diamond() {
        let m3 = diamond_m3_lattice();
        let result = verify_dedekind_theorem(&m3);
        assert!(result.is_modular);
        assert!(!result.contains_pentagon);
        assert!(result.theorem_holds);
    }

    #[test]
    fn test_modular_identity_chain() {
        let chain = chain_lattice(4);
        assert!(verify_modular_identity(&chain).is_ok());
    }

    #[test]
    fn test_modular_identity_n5() {
        let n5 = pentagon_n5_lattice();
        assert!(verify_modular_identity(&n5).is_err());
    }

    #[test]
    fn test_shearing_identity() {
        let p = powerset_lattice(2);
        assert!(verify_shearing_identity(&p).is_ok());
    }
}
