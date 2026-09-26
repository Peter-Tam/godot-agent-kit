//! Protocol-independent observation semantics and a separate authenticated local editor boundary.
//!
//! The observation core never acquires source or depends on transport. Target resolution only
//! validates filesystem identities and mutually authenticates source-free local candidates.

pub mod observation;

pub mod bridge;
pub mod project_fs;
pub mod target;
