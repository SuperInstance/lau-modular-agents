//! Core lattice structures: poset with join and meet operations.

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// A labeled element in a lattice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LatticeElement {
    /// Unique label identifying this element.
    pub label: String,
    /// Optional numeric rank/height in the lattice.
    pub rank: Option<usize>,
}

impl LatticeElement {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), rank: None }
    }

    pub fn with_rank(label: impl Into<String>, rank: usize) -> Self {
        Self { label: label.into(), rank: Some(rank) }
    }
}

impl Eq for LatticeElement {}

impl Hash for LatticeElement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}

/// A finite lattice with explicit join (∨) and meet (∧) operations.
///
/// Defined by a set of elements, a partial order (as adjacency), and join/meet tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiniteLattice {
    /// All elements in the lattice.
    elements: Vec<LatticeElement>,
    /// Partial order: covers[a] = set of elements that cover a (a < b, no c with a < c < b).
    covers: HashMap<String, HashSet<String>>,
    /// Join table: join of (a, b) stored by "a|b" key (always stores both orders).
    join_table: HashMap<String, String>,
    /// Meet table: meet of (a, b) stored by "a|b" key.
    meet_table: HashMap<String, String>,
    /// Top element label.
    top: Option<String>,
    /// Bottom element label.
    bottom: Option<String>,
}

impl FiniteLattice {
    /// Create a new empty lattice.
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            covers: HashMap::new(),
            join_table: HashMap::new(),
            meet_table: HashMap::new(),
            top: None,
            bottom: None,
        }
    }

    /// Create a lattice from elements and explicit join/meet tables.
    pub fn from_tables(
        elements: Vec<LatticeElement>,
        join_table: HashMap<(String, String), String>,
        meet_table: HashMap<(String, String), String>,
        covers: HashMap<String, HashSet<String>>,
    ) -> Self {
        let mut jt = HashMap::new();
        let mut mt = HashMap::new();
        for ((a, b), v) in &join_table {
            jt.insert(format!("{}|{}", a, b), v.clone());
            jt.insert(format!("{}|{}", b, a), v.clone());
        }
        for ((a, b), v) in &meet_table {
            mt.insert(format!("{}|{}", a, b), v.clone());
            mt.insert(format!("{}|{}", b, a), v.clone());
        }
        Self {
            elements,
            covers,
            join_table: jt,
            meet_table: mt,
            top: None,
            bottom: None,
        }
    }

    /// Add an element to the lattice.
    pub fn add_element(&mut self, elem: LatticeElement) {
        if !self.elements.iter().any(|e| e.label == elem.label) {
            self.covers.insert(elem.label.clone(), HashSet::new());
            self.elements.push(elem);
        }
    }

    /// Set the top element.
    pub fn set_top(&mut self, label: impl Into<String>) {
        self.top = Some(label.into());
    }

    /// Set the bottom element.
    pub fn set_bottom(&mut self, label: impl Into<String>) {
        self.bottom = Some(label.into());
    }

    /// Add a cover relation: a is covered by b (a < b, no element strictly between).
    pub fn add_cover(&mut self, a: impl Into<String>, b: impl Into<String>) {
        self.covers.entry(a.into()).or_default().insert(b.into());
    }

    /// Set the join of two elements.
    pub fn set_join(&mut self, a: &str, b: &str, result: impl Into<String>) {
        let r = result.into();
        self.join_table.insert(format!("{}|{}", a, b), r.clone());
        self.join_table.insert(format!("{}|{}", b, a), r);
    }

    /// Set the meet of two elements.
    pub fn set_meet(&mut self, a: &str, b: &str, result: impl Into<String>) {
        let r = result.into();
        self.meet_table.insert(format!("{}|{}", a, b), r.clone());
        self.meet_table.insert(format!("{}|{}", b, a), r);
    }

    /// Get all elements.
    pub fn elements(&self) -> &[LatticeElement] {
        &self.elements
    }

    /// Get element labels.
    pub fn labels(&self) -> Vec<String> {
        self.elements.iter().map(|e| e.label.clone()).collect()
    }

    /// Number of elements.
    pub fn size(&self) -> usize {
        self.elements.len()
    }

    /// Get the top element.
    pub fn top(&self) -> Option<&str> {
        self.top.as_deref()
    }

    /// Get the bottom element.
    pub fn bottom(&self) -> Option<&str> {
        self.bottom.as_deref()
    }

    /// Compute the join (∨) of two elements.
    pub fn join(&self, a: &str, b: &str) -> Option<&str> {
        self.join_table.get(&format!("{}|{}", a, b)).map(|s| s.as_str())
    }

    /// Compute the meet (∧) of two elements.
    pub fn meet(&self, a: &str, b: &str) -> Option<&str> {
        self.meet_table.get(&format!("{}|{}", a, b)).map(|s| s.as_str())
    }

    /// Check if a ≤ b in the partial order (transitive closure of covers).
    pub fn leq(&self, a: &str, b: &str) -> bool {
        if a == b { return true; }
        // BFS/DFS upward from a through covers
        let mut visited = HashSet::new();
        let mut stack = vec![a];
        while let Some(current) = stack.pop() {
            if current == b { return true; }
            if visited.insert(current.to_string()) {
                if let Some(cov) = self.covers.get(current) {
                    for next in cov {
                        stack.push(next.as_str());
                    }
                }
            }
        }
        false
    }

    /// Check if a < b (strict).
    pub fn lt(&self, a: &str, b: &str) -> bool {
        a != b && self.leq(a, b)
    }

    /// Get covers of an element.
    pub fn covers_of(&self, a: &str) -> HashSet<&str> {
        self.covers.get(a)
            .map(|s| s.iter().map(|x| x.as_str()).collect())
            .unwrap_or_default()
    }

    /// Verify lattice axioms: idempotent, commutative, associative, absorption.
    pub fn verify_axioms(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let labels: Vec<&str> = self.elements.iter().map(|e| e.label.as_str()).collect();

        for &a in &labels {
            // Idempotent: a ∨ a = a, a ∧ a = a
            if let Some(j) = self.join(a, a) {
                if j != a { errors.push(format!("Idempotent join fail: {} ∨ {} = {}", a, a, j)); }
            } else {
                errors.push(format!("Missing join: {} ∨ {}", a, a));
            }
            if let Some(m) = self.meet(a, a) {
                if m != a { errors.push(format!("Idempotent meet fail: {} ∧ {} = {}", a, a, m)); }
            } else {
                errors.push(format!("Missing meet: {} ∧ {}", a, a));
            }
        }

        for &a in &labels {
            for &b in &labels {
                // Commutative: a ∨ b = b ∨ a, a ∧ b = b ∧ a (ensured by storage, but verify)
                if let (Some(j1), Some(j2)) = (self.join(a, b), self.join(b, a)) {
                    if j1 != j2 { errors.push(format!("Commutative join fail: {} ∨ {} ≠ {} ∨ {}", a, b, b, a)); }
                }
                if let (Some(m1), Some(m2)) = (self.meet(a, b), self.meet(b, a)) {
                    if m1 != m2 { errors.push(format!("Commutative meet fail: {} ∧ {} ≠ {} ∧ {}", a, b, b, a)); }
                }
            }
        }

        for &a in &labels {
            for &b in &labels {
                for &c in &labels {
                    // Associative: (a ∨ b) ∨ c = a ∨ (b ∨ c)
                    if let (Some(ab), Some(bc)) = (self.join(a, b), self.join(b, c)) {
                        if let (Some(ab_c), Some(a_bc)) = (self.join(ab, c), self.join(a, bc)) {
                            if ab_c != a_bc {
                                errors.push(format!("Associative join fail: ({} ∨ {}) ∨ {} ≠ {} ∨ ({} ∨ {})", a, b, c, a, b, c));
                            }
                        }
                    }
                    // Associative meet
                    if let (Some(ab), Some(bc)) = (self.meet(a, b), self.meet(b, c)) {
                        if let (Some(ab_c), Some(a_bc)) = (self.meet(ab, c), self.meet(a, bc)) {
                            if ab_c != a_bc {
                                errors.push(format!("Associative meet fail: ({} ∧ {}) ∧ {} ≠ {} ∧ ({} ∧ {})", a, b, c, a, b, c));
                            }
                        }
                    }
                    // Absorption: a ∨ (a ∧ b) = a, a ∧ (a ∨ b) = a
                    if let Some(a_and_b) = self.meet(a, b) {
                        if let Some(j) = self.join(a, a_and_b) {
                            if j != a { errors.push(format!("Absorption join fail: {} ∨ ({} ∧ {}) = {} ≠ {}", a, a, b, j, a)); }
                        }
                    }
                    if let Some(a_or_b) = self.join(a, b) {
                        if let Some(m) = self.meet(a, a_or_b) {
                            if m != a { errors.push(format!("Absorption meet fail: {} ∧ ({} ∨ {}) = {} ≠ {}", a, a, b, m, a)); }
                        }
                    }
                }
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// Compute the transitive closure of the partial order.
    pub fn partial_order(&self) -> HashSet<(String, String)> {
        let mut order = HashSet::new();
        let labels: Vec<&str> = self.elements.iter().map(|e| e.label.as_str()).collect();
        for &a in &labels {
            for &b in &labels {
                if self.leq(a, b) {
                    order.insert((a.to_string(), b.to_string()));
                }
            }
        }
        order
    }
}

impl Default for FiniteLattice {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for lattice operations on generic types.
pub trait Lattice: Clone + PartialEq {
    /// Join (∨ / supremum / least upper bound).
    fn join(&self, other: &Self) -> Self;
    /// Meet (∧ / infimum / greatest lower bound).
    fn meet(&self, other: &Self) -> Self;
    /// Partial order comparison.
    fn leq(&self, other: &Self) -> bool;

    fn lt(&self, other: &Self) -> bool {
        self != other && self.leq(other)
    }
}

/// Hasse diagram representation of a lattice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HasseDiagram {
    pub nodes: Vec<LatticeElement>,
    pub edges: Vec<(String, String)>, // (lower, upper)
}

impl HasseDiagram {
    pub fn from_lattice(lattice: &FiniteLattice) -> Self {
        let nodes = lattice.elements().to_vec();
        let mut edges = Vec::new();
        for (a, covers) in &lattice.covers {
            for b in covers {
                edges.push((a.clone(), b.clone()));
            }
        }
        HasseDiagram { nodes, edges }
    }

    /// Get the height (length of longest chain from bottom to top).
    pub fn height(&self) -> usize {
        // Build adjacency and do longest path
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for (a, b) in &self.edges {
            adj.entry(a.as_str()).or_default().push(b.as_str());
        }
        let mut max_len = 0;
        for node in &self.nodes {
            let l = Self::longest_path(&adj, node.label.as_str(), &mut HashMap::new());
            max_len = max_len.max(l);
        }
        max_len
    }

    fn longest_path<'a>(
        adj: &HashMap<&'a str, Vec<&'a str>>,
        start: &'a str,
        cache: &mut HashMap<&'a str, usize>,
    ) -> usize {
        if let Some(&l) = cache.get(start) {
            return l;
        }
        let result = adj.get(start)
            .map(|neighbors| {
                neighbors.iter()
                    .map(|&n| Self::longest_path(adj, n, cache) + 1)
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        cache.insert(start, result);
        result
    }
}

/// Build a simple chain lattice of n elements.
pub fn chain_lattice(n: usize) -> FiniteLattice {
    let mut lattice = FiniteLattice::new();
    for i in 0..n {
        let label = format!("c{}", i);
        lattice.add_element(LatticeElement::with_rank(label.clone(), i));
    }
    for i in 0..n {
        for j in 0..n {
            let join_idx = i.max(j);
            let meet_idx = i.min(j);
            let a = format!("c{}", i);
            let b = format!("c{}", j);
            lattice.set_join(&a, &b, format!("c{}", join_idx));
            lattice.set_meet(&a, &b, format!("c{}", meet_idx));
        }
    }
    for i in 0..n.saturating_sub(1) {
        lattice.add_cover(format!("c{}", i), format!("c{}", i + 1));
    }
    if n > 0 {
        lattice.set_bottom("c0");
        lattice.set_top(format!("c{}", n - 1));
    }
    lattice
}

/// Build a diamond lattice M₃ (5 elements: bottom, top, and three incomparable middle elements).
pub fn diamond_m3_lattice() -> FiniteLattice {
    let mut lattice = FiniteLattice::new();
    lattice.add_element(LatticeElement::with_rank("bot", 0));
    lattice.add_element(LatticeElement::with_rank("a", 1));
    lattice.add_element(LatticeElement::with_rank("b", 1));
    lattice.add_element(LatticeElement::with_rank("c", 1));
    lattice.add_element(LatticeElement::with_rank("top", 2));

    lattice.set_bottom("bot");
    lattice.set_top("top");

    // Covers
    lattice.add_cover("bot", "a");
    lattice.add_cover("bot", "b");
    lattice.add_cover("bot", "c");
    lattice.add_cover("a", "top");
    lattice.add_cover("b", "top");
    lattice.add_cover("c", "top");

    // Join table
    for x in &["a", "b", "c"] {
        lattice.set_join("bot", x, *x);
        lattice.set_join("top", x, "top");
        lattice.set_join(x, x, *x);
    }
    lattice.set_join("bot", "bot", "bot");
    lattice.set_join("top", "top", "top");

    // a ∨ b = top, a ∨ c = top, b ∨ c = top (any two distinct middle elements join to top)
    for (x, y) in &[("a", "b"), ("a", "c"), ("b", "c")] {
        lattice.set_join(x, y, "top");
    }

    // Meet table
    for x in &["a", "b", "c"] {
        lattice.set_meet("top", x, *x);
        lattice.set_meet("bot", x, "bot");
        lattice.set_meet(x, x, *x);
    }
    lattice.set_meet("bot", "bot", "bot");
    lattice.set_meet("top", "top", "top");

    for (x, y) in &[("a", "b"), ("a", "c"), ("b", "c")] {
        lattice.set_meet(x, y, "bot");
    }

    lattice
}

/// Build the pentagon lattice N₅ (5 elements, non-modular).
pub fn pentagon_n5_lattice() -> FiniteLattice {
    let mut lattice = FiniteLattice::new();
    lattice.add_element(LatticeElement::with_rank("bot", 0));
    lattice.add_element(LatticeElement::with_rank("a", 1));
    lattice.add_element(LatticeElement::with_rank("b", 1));
    lattice.add_element(LatticeElement::with_rank("c", 2));
    lattice.add_element(LatticeElement::with_rank("top", 3));

    lattice.set_bottom("bot");
    lattice.set_top("top");

    // Covers: bot < a < c < top, and bot < b < top (b is incomparable with a and c)
    lattice.add_cover("bot", "a");
    lattice.add_cover("bot", "b");
    lattice.add_cover("a", "c");
    lattice.add_cover("c", "top");
    lattice.add_cover("b", "top");

    // Join table
    lattice.set_join("bot", "bot", "bot");
    lattice.set_join("top", "top", "top");
    lattice.set_join("bot", "a", "a");
    lattice.set_join("bot", "b", "b");
    lattice.set_join("bot", "c", "c");
    lattice.set_join("top", "a", "top");
    lattice.set_join("top", "b", "top");
    lattice.set_join("top", "c", "top");
    lattice.set_join("a", "a", "a");
    lattice.set_join("b", "b", "b");
    lattice.set_join("c", "c", "c");
    lattice.set_join("a", "b", "top"); // a ∨ b = top
    lattice.set_join("a", "c", "c");   // a ≤ c, so a ∨ c = c
    lattice.set_join("b", "c", "top"); // b ∨ c = top

    // Meet table
    lattice.set_meet("bot", "bot", "bot");
    lattice.set_meet("top", "top", "top");
    lattice.set_meet("bot", "a", "bot");
    lattice.set_meet("bot", "b", "bot");
    lattice.set_meet("bot", "c", "bot");
    lattice.set_meet("top", "a", "a");
    lattice.set_meet("top", "b", "b");
    lattice.set_meet("top", "c", "c");
    lattice.set_meet("a", "a", "a");
    lattice.set_meet("b", "b", "b");
    lattice.set_meet("c", "c", "c");
    lattice.set_meet("a", "b", "bot"); // a ∧ b = bot
    lattice.set_meet("a", "c", "a");   // a ≤ c, so a ∧ c = a
    lattice.set_meet("b", "c", "bot"); // b ∧ c = bot (key: makes it non-modular)

    lattice
}

/// Build a powerset lattice for a set of size n.
pub fn powerset_lattice(n: usize) -> FiniteLattice {
    let size = 1usize << n;
    let mut lattice = FiniteLattice::new();

    for i in 0..size {
        let bits: String = (0..n)
            .filter(|&j| i & (1 << j) != 0)
            .map(|j| char::from(b'a' + j as u8))
            .collect::<String>();
        let label = if bits.is_empty() { "\u{2205}".to_string() } else { bits.chars().collect::<Vec<_>>().iter().collect::<String>() };
        lattice.add_element(LatticeElement::new(label.clone()));
    }

    // Now we need to map bit patterns to labels
    let mut bit_to_label: HashMap<usize, String> = HashMap::new();
    for i in 0..size {
        let bits: Vec<char> = (0..n)
            .filter(|&j| i & (1 << j) != 0)
            .map(|j| char::from(b'a' + j as u8))
            .collect();
        let label = if bits.is_empty() { "\u{2205}".to_string() } else { bits.into_iter().collect() };
        bit_to_label.insert(i, label);
    }

    for i in 0..size {
        for j in 0..size {
            let join = i | j;
            let meet = i & j;
            let a = &bit_to_label[&i];
            let b = &bit_to_label[&j];
            lattice.set_join(a, b, &bit_to_label[&join]);
            lattice.set_meet(a, b, &bit_to_label[&meet]);
        }
    }

    // Covers: a covers b iff a ⊃ b and |a| = |b| + 1
    for i in 0..size {
        for j in 0..size {
            if i != j && (i & j) == j && (i.count_ones() as usize) == (j.count_ones() as usize) + 1 {
                lattice.add_cover(&bit_to_label[&j], &bit_to_label[&i]);
            }
        }
    }

    lattice.set_bottom("\u{2205}");
    let top_bits: String = (0..n).map(|j| char::from(b'a' + j as u8)).collect();
    lattice.set_top(&top_bits);

    lattice
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_lattice_creation() {
        let chain = chain_lattice(3);
        assert_eq!(chain.size(), 3);
        assert_eq!(chain.bottom(), Some("c0"));
        assert_eq!(chain.top(), Some("c2"));
    }

    #[test]
    fn test_chain_lattice_join() {
        let chain = chain_lattice(4);
        assert_eq!(chain.join("c0", "c2"), Some("c2"));
        assert_eq!(chain.join("c3", "c1"), Some("c3"));
    }

    #[test]
    fn test_chain_lattice_meet() {
        let chain = chain_lattice(4);
        assert_eq!(chain.meet("c0", "c2"), Some("c0"));
        assert_eq!(chain.meet("c3", "c1"), Some("c1"));
    }

    #[test]
    fn test_chain_lattice_order() {
        let chain = chain_lattice(4);
        assert!(chain.leq("c0", "c2"));
        assert!(chain.leq("c0", "c0"));
        assert!(!chain.leq("c2", "c0"));
        assert!(!chain.leq("c1", "c3") == false); // c1 ≤ c3 is true
        assert!(chain.leq("c1", "c3"));
    }

    #[test]
    fn test_chain_axioms() {
        let chain = chain_lattice(4);
        assert!(chain.verify_axioms().is_ok());
    }

    #[test]
    fn test_diamond_m3_creation() {
        let m3 = diamond_m3_lattice();
        assert_eq!(m3.size(), 5);
        assert_eq!(m3.bottom(), Some("bot"));
        assert_eq!(m3.top(), Some("top"));
    }

    #[test]
    fn test_diamond_m3_join() {
        let m3 = diamond_m3_lattice();
        assert_eq!(m3.join("a", "b"), Some("top"));
        assert_eq!(m3.join("a", "bot"), Some("a"));
        assert_eq!(m3.join("a", "top"), Some("top"));
    }

    #[test]
    fn test_diamond_m3_meet() {
        let m3 = diamond_m3_lattice();
        assert_eq!(m3.meet("a", "b"), Some("bot"));
        assert_eq!(m3.meet("a", "top"), Some("a"));
        assert_eq!(m3.meet("a", "bot"), Some("bot"));
    }

    #[test]
    fn test_diamond_m3_axioms() {
        let m3 = diamond_m3_lattice();
        assert!(m3.verify_axioms().is_ok());
    }

    #[test]
    fn test_pentagon_n5_creation() {
        let n5 = pentagon_n5_lattice();
        assert_eq!(n5.size(), 5);
    }

    #[test]
    fn test_pentagon_n5_axioms() {
        let n5 = pentagon_n5_lattice();
        assert!(n5.verify_axioms().is_ok());
    }

    #[test]
    fn test_powerset_lattice_2() {
        let p = powerset_lattice(2);
        assert_eq!(p.size(), 4);
        assert!(p.verify_axioms().is_ok());
    }

    #[test]
    fn test_powerset_lattice_3() {
        let p = powerset_lattice(3);
        assert_eq!(p.size(), 8);
        assert!(p.verify_axioms().is_ok());
    }

    #[test]
    fn test_hasse_diagram() {
        let chain = chain_lattice(3);
        let hasse = HasseDiagram::from_lattice(&chain);
        assert_eq!(hasse.nodes.len(), 3);
        assert_eq!(hasse.edges.len(), 2);
    }

    #[test]
    fn test_hasse_height() {
        let chain = chain_lattice(4);
        let hasse = HasseDiagram::from_lattice(&chain);
        assert_eq!(hasse.height(), 3); // 3 edges from c0 to c3
    }

    #[test]
    fn test_lattice_element_rank() {
        let e = LatticeElement::with_rank("x", 5);
        assert_eq!(e.rank, Some(5));
    }

    #[test]
    fn test_lattice_leq_reflexive() {
        let chain = chain_lattice(3);
        assert!(chain.leq("c0", "c0"));
        assert!(chain.leq("c1", "c1"));
    }

    #[test]
    fn test_lattice_lt_strict() {
        let chain = chain_lattice(3);
        assert!(chain.lt("c0", "c1"));
        assert!(!chain.lt("c1", "c1"));
    }
}
