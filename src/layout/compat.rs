//! Opt-in layout compatibility switches.
//!
//! Ironpress 1.4 laid out table cells with a simpler model than the
//! Chromium-parity engine: collapsed borders took no layout space, and block
//! children inside a cell lost their vertical margins except the cell's first
//! top and last bottom margin. Documents whose pagination was tuned against
//! those metrics can opt back into them with
//! [`crate::HtmlConverter::legacy_table_cell_metrics`]. The switch is scoped to
//! one conversion on the current thread.

use std::cell::Cell;

thread_local! {
    static LEGACY_TABLE_CELL_METRICS: Cell<bool> = const { Cell::new(false) };
}

/// Whether the conversion running on this thread uses Ironpress 1.4 table-cell metrics.
pub(crate) fn legacy_table_cell_metrics() -> bool {
    LEGACY_TABLE_CELL_METRICS.with(Cell::get)
}

/// Restores the previous switch value when the conversion scope ends.
pub(crate) struct LegacyTableCellMetricsScope {
    previous: bool,
}

impl LegacyTableCellMetricsScope {
    pub(crate) fn enter(enabled: bool) -> Self {
        let previous = LEGACY_TABLE_CELL_METRICS.with(|flag| flag.replace(enabled));
        Self { previous }
    }
}

impl Drop for LegacyTableCellMetricsScope {
    fn drop(&mut self) {
        LEGACY_TABLE_CELL_METRICS.with(|flag| flag.set(self.previous));
    }
}
