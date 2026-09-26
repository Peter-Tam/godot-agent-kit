//! Reusable, protocol-independent evidence classification for read-only GDScript observation.
//!
//! Integrations acquire and authenticate live evidence separately; this library never reads
//! project files, speaks to Godot, serializes a wire format, or authorizes an edit.

pub mod observation;
