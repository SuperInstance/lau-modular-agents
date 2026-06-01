//! Jordan-Hölder theorem: composition series length is invariant.
//!
//! A composition series of a modular lattice from bottom to top is a chain
//! bottom = a₀ < a₁ < ... < aₙ = top where each interval [aᵢ, aᵢ₊₁] is simple
//! (has no proper refinement). The Jordan-Hölder theorem states that any two
//! composition series have the same length, and the simple intervals are
//! isomorphic up to permutation.

use serde::{Serialize, Deserialize};
use crate::lattice::FiniteLattice;

/// A composition series: a maximal chain from bottom to top.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionSeries {
    /// The chain of element labels from bottom to top.
    pub chain: Vec<String>,
    /// The length (number of steps = chain.len() - 1).
    pub length: usize,
    /// The factors (intervals) as pairs (lower, upper).
    pub factors: Vec<(String, String)>,
}

impl CompositionSeries {
    /// Build a composition series from a chain.
    pub fn new(chain: Vec<String>) -> Self {
        let length = chain.len().saturating_sub(1);
        let factors = chain.windows(2)
            .map(|w| (w[0].clone(), w[1].clone()))
            .collect();
        Self { chain, length, factors }
    }

    /// Check if this is a valid composition series in the given lattice.
    pub fn is_valid(&self, lattice: &FiniteLattice) -> bool {
        if self.chain.is_empty() { return false; }

        // Must start at bottom and end at top
        if let Some(bottom) = lattice.bottom() {
            if self.chain[0] != bottom { return false; }
        }
        if let Some(top) = lattice.top() {
            if self.chain[self.chain.len() - 1] != top { return false; }
        }

        // Each consecutive pair must be a cover relation
        for i in 0..self.chain.len().saturating_sub(1) {
            let lower = &self.chain[i];
            let upper = &self.chain[i + 1];
            if !lattice.covers_of(lower).contains(upper.as_str()) {
                return false;
            }
        }

        true
    }
}

/// Find all maximal chains in a lattice.
pub fn find_maximal_chains(lattice: &FiniteLattice) -> Vec<Vec<String>> {
    let bottom = match lattice.bottom() {
        Some(b) => b.to_string(),
        None => return Vec::new(),
    };
    let top = match lattice.top() {
        Some(t) => t.to_string(),
        None => return Vec::new(),
    };

    let mut chains = Vec::new();
    let mut current = vec![bottom.clone()];
    find_chains_recursive(lattice, &bottom, &top, &mut current, &mut chains);
    chains
}

fn find_chains_recursive(
    lattice: &FiniteLattice,
    current: &str,
    top: &str,
    chain: &mut Vec<String>,
    result: &mut Vec<Vec<String>>,
) {
    if current == top {
        result.push(chain.clone());
        return;
    }

    let covers = lattice.covers_of(current);
    for next in covers {
        chain.push(next.to_string());
        find_chains_recursive(lattice, next, top, chain, result);
        chain.pop();
    }
}

/// Compute the Jordan-Hölder length (composition length) of a lattice.
/// Returns the length of any composition series (they're all the same for modular lattices).
pub fn jordan_holder_length(lattice: &FiniteLattice) -> Option<usize> {
    let chains = find_maximal_chains(lattice);
    chains.first().map(|c| c.len().saturating_sub(1))
}

/// Verify the Jordan-Hölder theorem: all maximal chains have the same length.
pub fn verify_jordan_holder(lattice: &FiniteLattice) -> Result<usize, Vec<usize>> {
    let chains = find_maximal_chains(lattice);
    let lengths: Vec<usize> = chains.iter().map(|c| c.len() - 1).collect();

    if lengths.is_empty() {
        return Ok(0);
    }

    let first = lengths[0];
    if lengths.iter().all(|&l| l == first) {
        Ok(first)
    } else {
        Err(lengths)
    }
}

/// Compute all composition series for a lattice.
pub fn all_composition_series(lattice: &FiniteLattice) -> Vec<CompositionSeries> {
    find_maximal_chains(lattice)
        .into_iter()
        .map(CompositionSeries::new)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lattice::{chain_lattice, diamond_m3_lattice, powerset_lattice, pentagon_n5_lattice};

    #[test]
    fn test_composition_series_chain() {
        let chain = chain_lattice(4);
        let series = all_composition_series(&chain);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].length, 3);
    }

    #[test]
    fn test_composition_series_valid() {
        let chain = chain_lattice(4);
        let series = all_composition_series(&chain);
        assert!(series[0].is_valid(&chain));
    }

    #[test]
    fn test_jordan_holder_length_chain() {
        let chain = chain_lattice(5);
        assert_eq!(jordan_holder_length(&chain), Some(4));
    }

    #[test]
    fn test_jordan_holder_diamond() {
        let m3 = diamond_m3_lattice();
        // M₃ has multiple maximal chains, all of length 2
        let result = verify_jordan_holder(&m3);
        assert_eq!(result, Ok(2));
    }

    #[test]
    fn test_jordan_holder_powerset() {
        let p = powerset_lattice(3);
        let result = verify_jordan_holder(&p);
        assert_eq!(result, Ok(3));
    }

    #[test]
    fn test_maximal_chains_diamond() {
        let m3 = diamond_m3_lattice();
        let chains = find_maximal_chains(&m3);
        assert_eq!(chains.len(), 3); // bot-a-top, bot-b-top, bot-c-top
    }

    #[test]
    fn test_maximal_chains_powerset2() {
        let p = powerset_lattice(2);
        let chains = find_maximal_chains(&p);
        assert_eq!(chains.len(), 2); // ∅-a-ab, ∅-b-ab
    }

    #[test]
    fn test_composition_factors() {
        let chain = chain_lattice(3);
        let series = CompositionSeries::new(vec!["c0".into(), "c1".into(), "c2".into()]);
        assert_eq!(series.factors.len(), 2);
        assert_eq!(series.factors[0], ("c0".into(), "c1".into()));
        assert_eq!(series.factors[1], ("c1".into(), "c2".into()));
    }

    #[test]
    fn test_pentagon_jordan_holder() {
        // N₅ is not modular, but Jordan-Hölder may still hold for some chains
        let n5 = pentagon_n5_lattice();
        let result = verify_jordan_holder(&n5);
        // N₅ has chains of different lengths: bot-b-top (length 2) and bot-a-c-top (length 3)
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_lattice_length() {
        let empty = FiniteLattice::new();
        assert_eq!(jordan_holder_length(&empty), None);
    }
}
