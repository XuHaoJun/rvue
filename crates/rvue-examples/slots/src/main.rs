//! Table example demonstrating slot mechanism with props
//!
//! This example shows how to use slots for building composable table components:
//! - TableHeadSlot - slot for table header content
//! - TableRowSlot - slot for table rows with props (index, is_even)
//! - TableCellSlot - slot for table cells with props (column, is_header)
//!
//! Pattern: Create slot instances with props, pass them as component props using `slot={instance}`

use rvue::prelude::*;
use rvue::text::TextContext;
use rvue::widget::BuildContext;
use rvue::TaffyTree;
use rvue_macro::{component, slot, view};

#[slot]
struct TableHeadSlot {
    children: ChildrenFn,
}

#[slot]
struct TableRowSlot {
    index: i32,
    is_even: bool,
    children: ChildrenFn,
}

#[slot]
struct TableCellSlot {
    column: String,
    is_header: bool,
    children: ChildrenFn,
}

#[component]
fn Table(head: TableHeadSlot) -> impl View {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter: u64 = 0;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);
    head.children.run(&mut ctx)
}

#[component]
fn TableRow(slot: TableRowSlot) -> impl View {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter: u64 = 0;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);
    slot.children.run(&mut ctx)
}

#[component]
fn TableCell(slot: TableCellSlot) -> impl View {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter: u64 = 0;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);
    slot.children.run(&mut ctx)
}

#[component]
fn App() -> impl View {
    let headers = TableHeadSlot::new(
        (|ctx: &mut BuildContext| {
            let widget = rvue::widgets::Text::new("Table Header".to_string());
            let state = rvue::widget::Widget::build(widget, ctx);
            let inner_comp = state.component();
            let new_comp = rvue::component::Component::with_global_id(
                inner_comp.component_type.clone(),
                inner_comp.props.borrow().clone(),
            );
            ViewStruct::new(new_comp)
        })
        .to_children(),
    );

    let cell1_content: ChildrenFn = (|ctx: &mut BuildContext| {
        let widget = rvue::widgets::Text::new("Header 1".to_string());
        let state = rvue::widget::Widget::build(widget, ctx);
        let inner_comp = state.component();
        let new_comp = rvue::component::Component::with_global_id(
            inner_comp.component_type.clone(),
            inner_comp.props.borrow().clone(),
        );
        ViewStruct::new(new_comp)
    })
    .to_children();
    let cell2_content: ChildrenFn = (|ctx: &mut BuildContext| {
        let widget = rvue::widgets::Text::new("Header 2".to_string());
        let state = rvue::widget::Widget::build(widget, ctx);
        let inner_comp = state.component();
        let new_comp = rvue::component::Component::with_global_id(
            inner_comp.component_type.clone(),
            inner_comp.props.borrow().clone(),
        );
        ViewStruct::new(new_comp)
    })
    .to_children();
    let cell3_content: ChildrenFn = (|ctx: &mut BuildContext| {
        let widget = rvue::widgets::Text::new("Header 3".to_string());
        let state = rvue::widget::Widget::build(widget, ctx);
        let inner_comp = state.component();
        let new_comp = rvue::component::Component::with_global_id(
            inner_comp.component_type.clone(),
            inner_comp.props.borrow().clone(),
        );
        ViewStruct::new(new_comp)
    })
    .to_children();

    let cell1 = TableCellSlot::new(cell1_content).column("col1".to_string()).is_header(true);
    let cell2 = TableCellSlot::new(cell2_content).column("col2".to_string()).is_header(true);
    let cell3 = TableCellSlot::new(cell3_content).column("col3".to_string()).is_header(true);

    let row1_cells: ChildrenFn = (|ctx: &mut BuildContext| {
        let widget = rvue::widgets::Flex::new()
            .direction(rvue::style::FlexDirection::Row)
            .gap(0.0)
            .align_items(rvue::style::AlignItems::Center)
            .justify_content(rvue::style::JustifyContent::Start);
        let state = rvue::widget::Widget::build(widget, ctx);
        let inner_comp = state.component();
        let new_comp = rvue::component::Component::with_global_id(
            inner_comp.component_type.clone(),
            inner_comp.props.borrow().clone(),
        );
        ViewStruct::new(new_comp)
    })
    .to_children();

    let row1 = TableRowSlot::new(row1_cells).index(1).is_even(false);

    let data_cells1: ChildrenFn = (|ctx: &mut BuildContext| {
        let widget = rvue::widgets::Text::new("Data 1".to_string());
        let state = rvue::widget::Widget::build(widget, ctx);
        let inner_comp = state.component();
        let new_comp = rvue::component::Component::with_global_id(
            inner_comp.component_type.clone(),
            inner_comp.props.borrow().clone(),
        );
        ViewStruct::new(new_comp)
    })
    .to_children();
    let _data_cell1 = TableCellSlot::new(data_cells1).column("col1".to_string()).is_header(false);

    view! {
        <Flex direction="column" gap=20.0>
            <Text content="Table with Slots Example" />

            <Table head=headers />

            <Text content="Header Cells:" />
            <TableCell slot=cell1 />
            <TableCell slot=cell2 />
            <TableCell slot=cell3 />

            <Text content="Row:" />
            <TableRow slot=row1>
                <TableCell slot=data_cell1 />
            </TableRow>
        </Flex>
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_view = App(Default::default());
    let app_component = app_view.into_component();
    let app_view_struct = ViewStruct::new(app_component);
    rvue::run_app(|| app_view_struct)?;
    Ok(())
}
