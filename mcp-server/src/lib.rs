//! Protocol-independent observation, guarded script editing and known-path opening,
//! with a separate authenticated local editor boundary.
//!
//! The observation core never acquires source or depends on transport. Target resolution only
//! validates filesystem identities and mutually authenticates source-free local candidates.
//! The edit core reduces independently acquired evidence; it cannot mutate editor state.
//! The opening core interprets lifecycle evidence; transport and native effects remain separate.

pub mod observation;
pub mod script_close;
pub(crate) mod script_closed_edit;
pub mod script_discovery;
pub mod script_edit;
pub mod script_open;
pub mod script_read;

pub mod bridge;
pub mod project_fs;
mod project_fs_validation;
pub mod runner;
pub mod target;
