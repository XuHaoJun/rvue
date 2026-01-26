//! Table example demonstrating slot mechanism with props
//!
//! This example shows how to use slots for building composable table components:
//! - TableHeadSlot - slot for table header content
//! - TableRowSlot - slot for table rows with props (index, is_even)
//! - TableCellSlot - slot for table cells with props (column, is_header)
//!
//! Pattern: Create slot instances with props, pass them as component props using `slot={instance}`

use rvue::prelude::*;
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
    let head_view = head.children.run();

    let root = Component::new(
        0,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 0.0,
            align_items: "stretch".to_string(),
            justify_content: "start".to_string(),
        },
    );

    root.add_child(head_view.root_component);

    root
}

#[component]
fn TableRow(slot: TableRowSlot) -> impl View {
    let cells_view = slot.children.run();

    let root = Component::new(
        0,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 0.0,
            align_items: "center".to_string(),
            justify_content: "start".to_string(),
        },
    );

    root.add_child(cells_view.root_component);

    root
}

#[component]
fn TableCell(slot: TableCellSlot) -> impl View {
    let cell_view = slot.children.run();

    let root = Component::new(
        0,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 0.0,
            align_items: "center".to_string(),
            justify_content: "start".to_string(),
        },
    );

    root.add_child(cell_view.root_component);

    root
}

#[component]
fn App() -> impl View {
    let headers = TableHeadSlot::new(
        (|| {
            ViewStruct::from_component(Component::new(
                0,
                ComponentType::Text,
                ComponentProps::Text {
                    content: "Table Header".to_string(),
                    font_size: None,
                    color: None,
                },
            ))
        })
        .to_children(),
    );

    let cell1_content: ChildrenFn = (|| {
        ViewStruct::from_component(Component::new(
            0,
            ComponentType::Text,
            ComponentProps::Text { content: "Header 1".to_string(), font_size: None, color: None },
        ))
    })
    .to_children();

    let cell2_content: ChildrenFn = (|| {
        ViewStruct::from_component(Component::new(
            0,
            ComponentType::Text,
            ComponentProps::Text { content: "Header 2".to_string(), font_size: None, color: None },
        ))
    })
    .to_children();

    let cell3_content: ChildrenFn = (|| {
        ViewStruct::from_component(Component::new(
            0,
            ComponentType::Text,
            ComponentProps::Text { content: "Header 3".to_string(), font_size: None, color: None },
        ))
    })
    .to_children();

    let cell1 = TableCellSlot::new(cell1_content).column("col1".to_string()).is_header(true);

    let cell2 = TableCellSlot::new(cell2_content).column("col2".to_string()).is_header(true);

    let cell3 = TableCellSlot::new(cell3_content).column("col3".to_string()).is_header(true);

    let row1_cells: ChildrenFn = (|| {
        ViewStruct::from_component(Component::new(
            0,
            ComponentType::Flex,
            ComponentProps::Flex {
                direction: "row".to_string(),
                gap: 0.0,
                align_items: "center".to_string(),
                justify_content: "start".to_string(),
            },
        ))
    })
    .to_children();

    let row1 = TableRowSlot::new(row1_cells).index(1).is_even(false);

    let data_cells1: ChildrenFn = (|| {
        ViewStruct::from_component(Component::new(
            0,
            ComponentType::Text,
            ComponentProps::Text { content: "Data 1".to_string(), font_size: None, color: None },
        ))
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
