//! Protocol-independent observation and guarded script-edit semantics, with a separate
//! authenticated local editor boundary.
//!
//! The observation core never acquires source or depends on transport. Target resolution only
//! validates filesystem identities and mutually authenticates source-free local candidates.
//! The edit core reduces independently acquired evidence; it cannot mutate editor state.

pub mod observation;
pub mod script_edit;

pub mod bridge;
pub mod project_fs;
mod project_fs_validation;
pub mod runner;
pub mod target;
