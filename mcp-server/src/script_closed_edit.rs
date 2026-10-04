//! Closed-source intent and source-free, sticky effect reduction.
use crate::observation::{ProjectRoot, RequestId, ResourcePath, SessionId};
use crate::script_edit::ReplacementSource;
use serde::{Deserialize, Serialize};
#[path = "script_closed_edit/model.rs"]
mod model;
pub(crate) use model::unsigned as model_unsigned;
pub(crate) use model::{CheckedClosedRequest, ClosedExpectedBasis};
pub(crate) use model::{ExpectedState, FileRevision, NativeReceipt, State};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Effects {
    pub authorized: bool,
    pub entry_refused: bool,
    pub resource_entered: bool,
    pub resource_changed: bool,
    pub disk_entered: bool,
    pub written_bytes: u64,
    pub truncated: bool,
    pub flushed: bool,
    pub readback: bool,
    pub mtime_restored: bool,
}
#[derive(Debug, Serialize)]
pub(crate) struct ClosedOutcome {
    pub request_id: Option<String>,
    pub interval: Option<serde_json::Value>,
    pub target: Option<serde_json::Value>,
    pub outcome: &'static str,
    pub application: &'static str,
    pub reason: String,
    pub stage: &'static str,
    pub history: &'static str,
    pub lifecycle: Lifecycle,
    pub effects: Effects,
    pub evidence: Option<serde_json::Value>,
    pub limitations: Vec<&'static str>,
    pub next_action: serde_json::Value,
}
#[derive(Debug, Serialize)]
pub(crate) struct Lifecycle {
    pub admitted: &'static str,
    pub final_state: &'static str,
}
impl Effects {
    pub(crate) fn merge(&mut self, r: &NativeReceipt) {
        self.resource_entered |= r.resource_entered;
        self.resource_changed |= r.resource_changed;
        self.disk_entered |= r.disk_entered;
        self.written_bytes = self.written_bytes.max(r.written_bytes.parse().unwrap_or(0));
        self.truncated |= r.truncate_done;
        self.flushed |= r.fsync_done;
        self.readback |= r.pread_done;
        self.mtime_restored |= r.mtime_restored;
        // A terminal discard only proves no entry when no earlier receipt showed entry.
        self.entry_refused |= r.terminal_discard && !r.resource_entered && !r.disk_entered;
    }
    pub(crate) fn outcome(
        &self,
        verified: bool,
        unchanged: bool,
        reason: &str,
        denied: bool,
        present: bool,
    ) -> ClosedOutcome {
        let entered = self.resource_entered || self.disk_entered;
        let known = self.resource_changed || self.written_bytes > 0 || self.truncated;
        let verified = verified
            && !denied
            && if unchanged {
                !self.authorized
                    && !entered
                    && !known
                    && !self.flushed
                    && !self.readback
                    && !self.mtime_restored
            } else {
                self.authorized
                    && self.disk_entered
                    && self.truncated
                    && self.flushed
                    && self.readback
                    && self.mtime_restored
                    && (!present || self.resource_entered)
            };
        let (outcome, application) = if verified {
            (
                if unchanged {
                    "verified_unchanged"
                } else {
                    "verified_changed"
                },
                if unchanged { "not_applied" } else { "applied" },
            )
        } else if known {
            (
                "applied_unverified",
                if self.flushed && self.readback && self.truncated {
                    "applied"
                } else {
                    "partly_applied"
                },
            )
        } else if entered || (self.authorized && !self.entry_refused) {
            ("application_unknown", "unknown")
        } else {
            ("refused", "not_applied")
        };
        ClosedOutcome {
            request_id: None,
            interval: None,
            target: None,
            outcome,
            application,
            reason: reason.to_owned(),
            stage: if self.authorized {
                if self.disk_entered {
                    "verification"
                } else {
                    "apply"
                }
            } else {
                "preflight"
            },
            history: "not_applicable_closed",
            lifecycle: Lifecycle {
                admitted: "closed",
                final_state: if verified { "closed" } else { "unknown" },
            },
            effects: self.clone(),
            evidence: None,
            limitations: if present {
                vec!["loaded_class_not_reloaded"]
            } else {
                vec![]
            },
            next_action: serde_json::json!({"kind":if verified {"none"} else {"fresh_read"}}),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authorization_loss_never_becomes_clean_refusal() {
        let e = Effects {
            authorized: true,
            ..Effects::default()
        };
        assert_eq!(
            e.outcome(false, false, "disconnected", false, false)
                .application,
            "unknown"
        );
    }
    #[test]
    fn entered_partial_effects_survive_later_denial() {
        let e = Effects {
            authorized: true,
            resource_entered: true,
            resource_changed: true,
            ..Effects::default()
        };
        let out = e.outcome(false, false, "denied_access", true, true);
        assert_eq!(out.outcome, "applied_unverified");
        assert!(out.evidence.is_none());
        assert!(out.effects.resource_changed);
    }
    #[test]
    fn unchanged_is_zero_effect_only() {
        let out = Effects::default().outcome(true, true, "complete", false, false);
        assert_eq!(out.outcome, "verified_unchanged");
        assert_eq!(out.application, "not_applied");
    }

    #[test]
    fn full_native_metadata_receipt_does_not_prove_independent_success() {
        let e = Effects {
            authorized: true,
            disk_entered: true,
            written_bytes: 12,
            truncated: true,
            flushed: true,
            readback: true,
            mtime_restored: true,
            ..Effects::default()
        };
        assert_eq!(
            e.outcome(false, false, "incomplete_verification", false, false)
                .outcome,
            "applied_unverified"
        );
        assert_eq!(
            e.outcome(true, false, "complete", false, false).outcome,
            "verified_changed"
        );
        assert_ne!(
            e.outcome(true, true, "complete", false, false).outcome,
            "verified_unchanged"
        );
    }
    #[test]
    fn terminal_discard_cannot_erase_earlier_entry() {
        let e = Effects {
            authorized: true,
            entry_refused: true,
            resource_entered: true,
            resource_changed: true,
            ..Effects::default()
        };
        assert_eq!(
            e.outcome(false, false, "cancelled", false, true)
                .application,
            "partly_applied"
        );
    }
}
