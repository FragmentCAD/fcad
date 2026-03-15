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
