use super::*;
use crate::layout::cells::TableRowCells;
use crate::layout::elements::{GridRow, LayoutVisitor, TableRow};

mod grid;
mod table;

/// Top-left `y` and height of each table row, using the same cursor walk as
/// painting so a rowspan cell can span intervening `border-spacing`.
fn table_row_boxes(
    elements: &[&dyn LayoutElement],
    mut cursor_y: f32,
    mut previous_margin_bottom: f32,
    mut first_margin: FirstMarginState,
) -> Vec<Option<(f32, f32)>> {
    struct RowFlow {
        margins: crate::layout::flow_metrics::BlockMargins,
        internal: crate::layout::flow_metrics::BlockMargins,
        extra_end: f32,
        height: f32,
    }

    struct Probe(Option<RowFlow>);

    impl LayoutVisitor for Probe {
        fn visit_table_row(&mut self, element: &TableRow) {
            self.0 = Some(RowFlow {
                margins: element.flow.margins,
                internal: element.flow.internal,
                extra_end: element.flow.extra_end,
                height: element.content.cells.row_block_extent(),
            });
        }
    }

    elements
        .iter()
        .map(|element| {
            let mut probe = Probe(None);
            element.accept(&mut probe);
            let Some(row) = probe.0 else {
                cursor_y -= crate::layout::paginate::estimate_element_height(*element);
                return None;
            };
            if first_margin == FirstMarginState::Pending {
                cursor_y -= collapsed_margin_top_extra(row.margins.start, previous_margin_bottom);
            }
            first_margin = FirstMarginState::Pending;
            cursor_y -= row.internal.start;
            let row_y = cursor_y;
            cursor_y -= row.height + row.internal.end + row.extra_end + row.margins.end;
            previous_margin_bottom = row.margins.end;
            Some((row_y, row.height))
        })
        .collect()
}

fn rowspan_cell_height(
    row_boxes: &[Option<(f32, f32)>],
    index: usize,
    span: usize,
    row_height: f32,
) -> f32 {
    if span <= 1 {
        // A single-row cell is exactly its row. Re-deriving that height from
        // the cursor walk would add float noise that tips pixel-snapped
        // vertical alignment.
        return row_height;
    }
    let last = index + span - 1;
    match (
        row_boxes.get(index).copied().flatten(),
        row_boxes.get(last).copied().flatten(),
    ) {
        (Some((top, _)), Some((last_top, last_height))) => {
            (top - (last_top - last_height)).max(row_height)
        }
        _ => {
            row_height
                + row_boxes
                    .iter()
                    .skip(index + 1)
                    .take(span.saturating_sub(1))
                    .filter_map(|entry| entry.map(|(_, height)| height))
                    .sum::<f32>()
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum FirstMarginState {
    #[default]
    Pending,
    Resolved,
}

/// CSS table-cell paint phases coordinated across every row of one table grid.
///
/// A cell whose compositing must remain atomic is emitted in `Contents`; an
/// ordinary in-flow cell contributes independently to all three phases.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TableCellPaintPhase {
    #[default]
    All,
    Backgrounds,
    Borders,
    Contents,
}

impl TableCellPaintPhase {
    const fn paints_backgrounds(self) -> bool {
        matches!(self, Self::All | Self::Backgrounds)
    }

    const fn paints_borders(self) -> bool {
        matches!(self, Self::All | Self::Borders)
    }

    const fn paints_contents(self) -> bool {
        matches!(self, Self::All | Self::Contents)
    }
}

/// Entry state for a contiguous run of internal table/grid rows.
///
/// Reordered painting receives positions from the shared flow planner, where
/// the first table margin has already been collapsed; source-order painting
/// resolves it here. The distinction is explicit so a margin is never applied
/// twice merely because stacking order required a planning pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct NestedRowsFlow {
    position: FlowPosition,
    first_margin: FirstMarginState,
}

impl NestedRowsFlow {
    pub(super) const fn pending(position: FlowPosition) -> Self {
        Self {
            position,
            first_margin: FirstMarginState::Pending,
        }
    }

    pub(super) const fn resolved(position: FlowPosition) -> Self {
        Self {
            position,
            first_margin: FirstMarginState::Resolved,
        }
    }
}

struct NestedRowsRenderer<'call, 'fonts> {
    content: &'call mut String,
    paint: bool,
    origin_x: f32,
    cursor_y: f32,
    page_ext_gstates: &'call mut Vec<(String, f32)>,
    bg_alpha_counter: &'call mut usize,
    custom_fonts: &'fonts HashMap<String, TtfFont>,
    prepared_custom_fonts: &'fonts PreparedCustomFonts,
    page_shadings: &'call mut Vec<ShadingEntry>,
    shading_counter: &'call mut usize,
    pdf_writer: &'call mut PdfWriter,
    page_images: &'call mut Vec<ImageRef>,
    annotations: &'call mut Vec<LinkAnnotation>,
    stacking: &'call mut StackingTraversal,
    abs_origins: &'call mut HashMap<usize, PdfPoint>,
    page_paint_box: PdfRect,
    initial_fixed_origin: PdfPoint,
    page_height: f32,
    previous_margin_bottom: f32,
    first_margin: FirstMarginState,
    row_boxes: Vec<Option<(f32, f32)>>,
    element_index: usize,
    table_cell_phase: TableCellPaintPhase,
    handled: bool,
}

impl LayoutVisitor for NestedRowsRenderer<'_, '_> {
    fn visit_table_row(&mut self, element: &TableRow) {
        self.render_table_row(element);
    }

    fn visit_grid_row(&mut self, element: &GridRow) {
        self.render_grid_row(element);
    }
}

/// Render table and grid rows at a resolved flow origin.
///
/// Page roots, ordinary containers, table cells, and grid/flex descendants all
/// enter this same painter; only the flow origin differs.
pub(super) fn render_rows(
    content: &mut String,
    elements: &[&dyn LayoutElement],
    origin_x: f32,
    flow: NestedRowsFlow,
    paint: bool,
    abs_origins: &mut HashMap<usize, PdfPoint>,
    ctx: &mut PageRenderContext<'_>,
) -> FlowPosition {
    let mut position = flow.position;
    let mut first_margin = flow.first_margin;
    let mut start = 0;
    while start < elements.len() {
        let table_grid = elements[start]
            .table_grid_owner()
            .map(crate::layout::elements::TableGridOwner::table_grid_identity);
        let end = if let Some(table_grid) = table_grid {
            elements[start + 1..]
                .iter()
                .position(|element| {
                    element
                        .table_grid_owner()
                        .map(crate::layout::elements::TableGridOwner::table_grid_identity)
                        != Some(table_grid)
                })
                .map_or(elements.len(), |offset| start + 1 + offset)
        } else {
            start + 1
        };
        let segment_flow = NestedRowsFlow {
            position,
            first_margin,
        };
        if paint && table_grid.is_some() {
            for phase in [
                TableCellPaintPhase::Backgrounds,
                TableCellPaintPhase::Borders,
                TableCellPaintPhase::Contents,
            ] {
                position = render_row_pass(
                    content,
                    &elements[start..end],
                    origin_x,
                    segment_flow,
                    true,
                    phase,
                    abs_origins,
                    ctx,
                );
            }
        } else {
            position = render_row_pass(
                content,
                &elements[start..end],
                origin_x,
                segment_flow,
                paint,
                TableCellPaintPhase::All,
                abs_origins,
                ctx,
            );
        }
        first_margin = FirstMarginState::Pending;
        start = end;
    }
    position
}

#[allow(clippy::too_many_arguments)]
fn render_row_pass(
    content: &mut String,
    elements: &[&dyn LayoutElement],
    origin_x: f32,
    flow: NestedRowsFlow,
    paint: bool,
    table_cell_phase: TableCellPaintPhase,
    abs_origins: &mut HashMap<usize, PdfPoint>,
    ctx: &mut PageRenderContext<'_>,
) -> FlowPosition {
    let row_boxes = table_row_boxes(
        elements,
        flow.position.cursor_y,
        flow.position.previous_margin_bottom,
        flow.first_margin,
    );
    let mut renderer = NestedRowsRenderer {
        content,
        paint,
        origin_x,
        cursor_y: flow.position.cursor_y,
        page_ext_gstates: ctx.page_ext_gstates,
        bg_alpha_counter: ctx.bg_alpha_counter,
        custom_fonts: ctx.text.custom_fonts,
        prepared_custom_fonts: ctx.text.prepared_custom_fonts,
        page_shadings: ctx.shadings,
        shading_counter: ctx.shading_counter,
        pdf_writer: ctx.text.pdf_writer,
        page_images: ctx.text.page_images,
        annotations: ctx.text.annotations,
        stacking: &mut ctx.stacking,
        abs_origins,
        page_paint_box: ctx.paint_box,
        initial_fixed_origin: ctx.initial_fixed_origin,
        page_height: ctx.text.page_height,
        previous_margin_bottom: flow.position.previous_margin_bottom,
        first_margin: flow.first_margin,
        row_boxes,
        element_index: 0,
        table_cell_phase,
        handled: false,
    };
    for (element_index, &element) in elements.iter().enumerate() {
        renderer.element_index = element_index;
        renderer.handled = false;
        element.accept(&mut renderer);
        if !renderer.handled {
            renderer.cursor_y -= crate::layout::paginate::estimate_element_height(element);
            renderer.previous_margin_bottom = 0.0;
        }
    }
    FlowPosition::new(
        renderer.cursor_y,
        renderer.cursor_y,
        renderer.previous_margin_bottom,
    )
}

#[cfg(test)]
mod tests {
    use super::rowspan_cell_height;

    #[test]
    fn single_row_cell_height_is_exactly_the_row_height() {
        // Re-deriving it from the cursor walk yields 14.400002 for 14.400001,
        // which tips pixel-snapped vertical alignment by a whole device pixel.
        let boxes = [Some((87.75, 14.400001))];
        let height = rowspan_cell_height(&boxes, 0, 1, 14.400001);
        assert_eq!(height.to_bits(), 14.400001_f32.to_bits());
    }

    #[test]
    fn rowspan_cell_height_includes_border_spacing_between_rows() {
        let boxes = [Some((100.0, 20.0)), Some((77.0, 20.0))];
        assert_eq!(rowspan_cell_height(&boxes, 0, 2, 20.0), 43.0);
    }
}
