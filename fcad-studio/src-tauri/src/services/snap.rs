use fcad_core::application::snap::SnapState;

pub struct SnapService;

impl SnapService {
    pub fn toggle_ortho(state: &mut SnapState) -> bool {
        state.ortho_enabled = !state.ortho_enabled;
        state.ortho_enabled
    }

    pub fn toggle_osnaps(state: &mut SnapState) -> bool {
        state.osnaps_enabled = !state.osnaps_enabled;
        state.osnaps_enabled
    }

    pub fn toggle_grid_snap(state: &mut SnapState) -> bool {
        state.grid_snap_enabled = !state.grid_snap_enabled;
        state.grid_snap_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fcad_core::application::snap::SnapState;

    #[test]
    fn test_toggle_ortho() {
        let mut state = SnapState::default();
        let initial = state.ortho_enabled;
        let new_state = SnapService::toggle_ortho(&mut state);
        assert_eq!(new_state, !initial);
        assert_eq!(state.ortho_enabled, new_state);
    }

    #[test]
    fn test_toggle_osnaps() {
        let mut state = SnapState::default();
        let initial = state.osnaps_enabled;
        let new_state = SnapService::toggle_osnaps(&mut state);
        assert_eq!(new_state, !initial);
        assert_eq!(state.osnaps_enabled, new_state);
    }
}
