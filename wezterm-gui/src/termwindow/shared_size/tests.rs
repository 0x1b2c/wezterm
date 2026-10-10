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
