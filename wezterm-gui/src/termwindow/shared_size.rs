//! Several GUI clients, on different machines, can show the same mux
//! session. Each pane has a single size on the mux server: the size the
//! client that resized it last asked for. A client only reports its size
//! when its own window changes size, so a client that comes back into use
//! after another machine was used keeps rendering panes laid out for the
//! other machine's window.
//!
//! The size follows the client that was used last (like tmux's
//! `window-size latest`): when a window gains focus, receives keyboard or
//! mouse input, or switches tabs, it reports its own size again for its
//! active tab, but only if the panes do not already have that size.
use mux::tab::TabId;

/// Sizes in cells, as (cols, rows).
pub type CellSize = (usize, usize);

/// One pane of the active tab: the size this window lays it out at, and the
/// size its terminal reports, which for a remote pane is the size the mux
/// server holds.
#[derive(Debug, Clone, Copy)]
pub struct PaneSizes {
    pub laid_out: CellSize,
    pub reported: CellSize,
}

/// The state right after this window last reported its size for a tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeClaim {
    pub tab_id: TabId,
    pub wanted: CellSize,
    pub tab_size_after: CellSize,
}

/// Whether the window should report `wanted` for the tab again.
///
/// `tab_size` is the size the local tab currently has; a resync with the
/// mux server replaces it with the server's size. `last_claim` keeps a tab
/// that its minimum pane sizes hold larger than the window from being
/// resized again on every keystroke.
pub fn claim_needed(
    tab_id: TabId,
    wanted: CellSize,
    tab_size: CellSize,
    panes: &[PaneSizes],
    last_claim: Option<SizeClaim>,
) -> bool {
    if wanted.0 == 0 || wanted.1 == 0 {
        return false;
    }
    if panes.iter().any(|p| p.laid_out != p.reported) {
        return true;
    }
    if tab_size == wanted {
        return false;
    }
    let already_claimed = SizeClaim {
        tab_id,
        wanted,
        tab_size_after: tab_size,
    };
    last_claim != Some(already_claimed)
}

impl super::TermWindow {
    /// Report this window's size for its active tab again when another
    /// client of the same mux session has resized it; see the module
    /// documentation.
    pub fn claim_shared_tab_size(&mut self) {
        if self.resizes_pending > 0 {
            // The window is in the middle of changing size itself; its own
            // resize will report the new size.
            return;
        }
        let mux = mux::Mux::get();
        let tab = match mux.get_active_tab_for_window(self.mux_window_id) {
            Some(tab) => tab,
            None => return,
        };
        let size = self.terminal_size;
        let wanted = (size.cols, size.rows);
        let tab_size = tab.get_size();
        let panes: Vec<PaneSizes> = tab
            .iter_panes()
            .iter()
            .map(|p| {
                let dims = p.pane.get_dimensions();
                PaneSizes {
                    laid_out: (p.width, p.height),
                    reported: (dims.cols, dims.viewport_rows),
                }
            })
            .collect();
        if !claim_needed(
            tab.tab_id(),
            wanted,
            (tab_size.cols, tab_size.rows),
            &panes,
            self.last_size_claim,
        ) {
            return;
        }
        log::debug!(
            "tab {} does not have this window's size {:?}; reporting it again",
            tab.tab_id(),
            wanted
        );
        tab.resize(size);
        let after = tab.get_size();
        self.last_size_claim = Some(SizeClaim {
            tab_id: tab.tab_id(),
            wanted,
            tab_size_after: (after.cols, after.rows),
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn pane(laid_out: CellSize, reported: CellSize) -> PaneSizes {
        PaneSizes { laid_out, reported }
    }

    #[test]
    fn a_window_reclaims_the_size_after_another_client_resized_its_panes() {
        // The server holds the other machine's smaller size for the pane.
        let panes = [pane((200, 50), (160, 44))];
        assert!(claim_needed(1, (200, 50), (200, 50), &panes, None));
    }

    #[test]
    fn a_window_reclaims_the_size_after_a_resync_gave_its_tab_another_clients_size() {
        // The local tab and its panes were replaced by the server's layout.
        let panes = [pane((160, 44), (160, 44))];
        assert!(claim_needed(1, (200, 50), (160, 44), &panes, None));
    }

    #[test]
    fn a_window_whose_panes_already_have_its_size_reports_nothing() {
        let panes = [pane((100, 50), (100, 50)), pane((99, 50), (99, 50))];
        assert!(!claim_needed(1, (200, 50), (200, 50), &panes, None));
    }

    #[test]
    fn a_tab_held_larger_than_the_window_is_not_resized_again_on_every_input() {
        let claim = SizeClaim {
            tab_id: 1,
            wanted: (20, 5),
            tab_size_after: (24, 6),
        };
        let panes = [pane((12, 6), (12, 6)), pane((11, 6), (11, 6))];
        assert!(!claim_needed(1, (20, 5), (24, 6), &panes, Some(claim)));
        // A different window size is a new claim.
        assert!(claim_needed(1, (30, 8), (24, 6), &panes, Some(claim)));
    }

    #[test]
    fn a_window_without_a_size_yet_reports_nothing() {
        let panes = [pane((0, 0), (80, 24))];
        assert!(!claim_needed(1, (0, 0), (80, 24), &panes, None));
    }
}
