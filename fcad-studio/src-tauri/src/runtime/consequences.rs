use crate::runtime::authority_dispatcher::{MutationOutcome, RenderHint};

/// Transitional adapter seam for PR #1.
///
/// The current renderer still polls ECS trackers. This function intentionally
/// keeps geometry consequences explicit without forcing the full
/// DomainEvent/RenderInvalidation queue migration into this PR.
pub fn apply_runtime_consequences(outcome: &MutationOutcome) -> bool {
    match outcome.render_hint {
        Some(RenderHint::GeometryChanged) | Some(RenderHint::LayerChanged) => true,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::authority_dispatcher::{MutationOutcome, RenderHint};

    #[test]
    fn noop_outcome_has_no_runtime_consequence() {
        assert!(!apply_runtime_consequences(&MutationOutcome::noop()));
    }

    #[test]
    fn render_hint_has_runtime_consequence() {
        let outcome = MutationOutcome::resource_changed(RenderHint::LayerChanged);
        assert!(apply_runtime_consequences(&outcome));
    }
}
