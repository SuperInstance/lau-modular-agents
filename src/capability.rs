//! Capability hierarchy as modular lattice.
//!
//! Application of modular lattice theory to agent capability hierarchies.
//! Capabilities form a modular lattice where join = combined capability,
//! meet = shared capability. This gives clean composition guarantees.

use serde::{Serialize, Deserialize};
use crate::lattice::{FiniteLattice, LatticeElement};
use crate::modular::check_modular_law;
use std::collections::HashMap;

/// A capability in an agent hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Capability {
    pub name: String,
    pub level: usize,
    pub description: String,
}

impl Capability {
    pub fn new(name: impl Into<String>, level: usize) -> Self {
        Self {
            name: name.into(),
            level,
            description: String::new(),
        }
    }

    pub fn with_description(name: impl Into<String>, level: usize, desc: impl Into<String>) -> Self {
        Self { name: name.into(), level, description: desc.into() }
    }
}

/// An agent's capability hierarchy represented as a modular lattice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilityHierarchy {
    /// Agent identifier.
    pub agent_id: String,
    /// Capabilities indexed by name.
    pub capabilities: HashMap<String, Capability>,
    /// The underlying modular lattice.
    lattice: FiniteLattice,
}

impl AgentCapabilityHierarchy {
    /// Create a new empty capability hierarchy.
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            capabilities: HashMap::new(),
            lattice: FiniteLattice::new(),
        }
    }

    /// Add a capability to the hierarchy.
    pub fn add_capability(&mut self, cap: Capability) {
        self.lattice.add_element(LatticeElement::with_rank(&cap.name, cap.level));
        self.capabilities.insert(cap.name.clone(), cap);
    }

    /// Set that cap_a is a sub-capability of cap_b (a ≤ b).
    pub fn set_subcapability(&mut self, a: &str, b: &str) {
        self.lattice.add_cover(a, b);
    }

    /// Set the join of two capabilities.
    pub fn set_join(&mut self, a: &str, b: &str, result: &str) {
        self.lattice.set_join(a, b, result);
    }

    /// Set the meet of two capabilities.
    pub fn set_meet(&mut self, a: &str, b: &str, result: &str) {
        self.lattice.set_meet(a, b, result);
    }

    /// Set the bottom capability (base capability).
    pub fn set_base(&mut self, name: &str) {
        self.lattice.set_bottom(name);
    }

    /// Set the top capability (full capability).
    pub fn set_full(&mut self, name: &str) {
        self.lattice.set_top(name);
    }

    /// Get the underlying lattice.
    pub fn lattice(&self) -> &FiniteLattice {
        &self.lattice
    }

    /// Verify this forms a valid modular lattice (composition guarantees).
    pub fn verify_modular(&self) -> Result<(), Vec<String>> {
        check_modular_law(&self.lattice)
    }

    /// Get the composition guarantee: if cap_a ≤ cap_b, then combining
    /// cap_a with any cap_x and intersecting with cap_b is the same as
    /// intersecting cap_x with cap_b first.
    pub fn composition_guarantee(&self, a: &str, x: &str, b: &str) -> bool {
        if !self.lattice.leq(a, b) { return true; }
        dedekind_modular_law(&self.lattice, a, x, b).unwrap_or(false)
    }

    /// Check if a capability subsumes another.
    pub fn subsumes(&self, a: &str, b: &str) -> bool {
        self.lattice.leq(b, a)
    }

    /// Get all capabilities.
    pub fn all_capabilities(&self) -> Vec<&Capability> {
        self.capabilities.values().collect()
    }
}

fn dedekind_modular_law(lattice: &FiniteLattice, a: &str, x: &str, b: &str) -> Option<bool> {
    let x_meet_b = lattice.meet(x, b)?;
    let a_join_x = lattice.join(a, x)?;
    let lhs = lattice.join(a, x_meet_b)?;
    let rhs = lattice.meet(a_join_x, b)?;
    Some(lhs == rhs)
}

/// Build a capability lattice from a list of capabilities with explicit join/meet.
#[derive(Debug, Clone)]
pub struct CapabilityLattice {
    hierarchy: AgentCapabilityHierarchy,
}

impl CapabilityLattice {
    /// Create from a builder pattern.
    pub fn build(agent_id: impl Into<String>) -> CapabilityLatticeBuilder {
        CapabilityLatticeBuilder {
            hierarchy: AgentCapabilityHierarchy::new(agent_id),
        }
    }

    pub fn hierarchy(&self) -> &AgentCapabilityHierarchy {
        &self.hierarchy
    }

    pub fn into_hierarchy(self) -> AgentCapabilityHierarchy {
        self.hierarchy
    }
}

pub struct CapabilityLatticeBuilder {
    hierarchy: AgentCapabilityHierarchy,
}

impl CapabilityLatticeBuilder {
    pub fn capability(mut self, name: &str, level: usize) -> Self {
        self.hierarchy.add_capability(Capability::new(name, level));
        self
    }

    pub fn subcapability(mut self, a: &str, b: &str) -> Self {
        self.hierarchy.set_subcapability(a, b);
        self
    }

    pub fn join(mut self, a: &str, b: &str, result: &str) -> Self {
        self.hierarchy.set_join(a, b, result);
        self
    }

    pub fn meet(mut self, a: &str, b: &str, result: &str) -> Self {
        self.hierarchy.set_meet(a, b, result);
        self
    }

    pub fn base(mut self, name: &str) -> Self {
        self.hierarchy.set_base(name);
        self
    }

    pub fn full(mut self, name: &str) -> Self {
        self.hierarchy.set_full(name);
        self
    }

    pub fn finish(self) -> CapabilityLattice {
        CapabilityLattice { hierarchy: self.hierarchy }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_robot_hierarchy() -> CapabilityLattice {
        CapabilityLattice::build("robot-1")
            .capability("base", 0)
            .capability("move", 1)
            .capability("sense", 1)
            .capability("navigate", 2)
            .capability("full", 3)
            .base("base")
            .full("full")
            .subcapability("base", "move")
            .subcapability("base", "sense")
            .subcapability("move", "navigate")
            .subcapability("sense", "navigate")
            .subcapability("navigate", "full")
            // Join table
            .join("base", "base", "base")
            .join("base", "move", "move")
            .join("base", "sense", "sense")
            .join("base", "navigate", "navigate")
            .join("base", "full", "full")
            .join("move", "move", "move")
            .join("move", "sense", "navigate")
            .join("move", "navigate", "navigate")
            .join("move", "full", "full")
            .join("sense", "sense", "sense")
            .join("sense", "navigate", "navigate")
            .join("sense", "full", "full")
            .join("navigate", "navigate", "navigate")
            .join("navigate", "full", "full")
            .join("full", "full", "full")
            // Meet table
            .meet("base", "base", "base")
            .meet("base", "move", "base")
            .meet("base", "sense", "base")
            .meet("base", "navigate", "base")
            .meet("base", "full", "base")
            .meet("move", "move", "move")
            .meet("move", "sense", "base")
            .meet("move", "navigate", "move")
            .meet("move", "full", "move")
            .meet("sense", "sense", "sense")
            .meet("sense", "navigate", "sense")
            .meet("sense", "full", "sense")
            .meet("navigate", "navigate", "navigate")
            .meet("navigate", "full", "navigate")
            .meet("full", "full", "full")
            .finish()
    }

    #[test]
    fn test_capability_lattice_build() {
        let cl = build_robot_hierarchy();
        assert_eq!(cl.hierarchy().lattice().size(), 5);
    }

    #[test]
    fn test_capability_lattice_modular() {
        let cl = build_robot_hierarchy();
        assert!(cl.hierarchy().verify_modular().is_ok());
    }

    #[test]
    fn test_capability_subsumes() {
        let cl = build_robot_hierarchy();
        assert!(cl.hierarchy().subsumes("navigate", "move"));
        assert!(cl.hierarchy().subsumes("full", "base"));
        assert!(!cl.hierarchy().subsumes("move", "sense"));
    }

    #[test]
    fn test_capability_join() {
        let cl = build_robot_hierarchy();
        let l = cl.hierarchy().lattice();
        assert_eq!(l.join("move", "sense"), Some("navigate"));
    }

    #[test]
    fn test_capability_meet() {
        let cl = build_robot_hierarchy();
        let l = cl.hierarchy().lattice();
        assert_eq!(l.meet("move", "sense"), Some("base"));
    }

    #[test]
    fn test_capability_composition_guarantee() {
        let cl = build_robot_hierarchy();
        // move ≤ navigate, so modular law holds for any x
        assert!(cl.hierarchy().composition_guarantee("move", "sense", "navigate"));
    }

    #[test]
    fn test_capability_lattice_axioms() {
        let cl = build_robot_hierarchy();
        assert!(cl.hierarchy().lattice().verify_axioms().is_ok());
    }

    #[test]
    fn test_capability_hierarchy_creation() {
        let h = AgentCapabilityHierarchy::new("test-agent");
        assert_eq!(h.agent_id, "test-agent");
    }

    #[test]
    fn test_capability_list() {
        let cl = build_robot_hierarchy();
        let caps = cl.hierarchy().all_capabilities();
        assert_eq!(caps.len(), 5);
    }
}
