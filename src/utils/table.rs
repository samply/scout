use std::collections::HashSet;

use dioxus::prelude::*;
use itertools::Itertools;

#[derive(Props, Clone, PartialEq)]
pub struct TableProps {
    pub columns: Vec<Column>,
    pub data: Vec<Vec<String>>,
    pub ondetail: EventHandler<usize>,
}

#[derive(Clone, PartialEq, Debug)]
enum DragState {
    None,
    Mousedown(usize),
    Dragging(usize),
    Dragover(usize, usize),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Column {
    name: String,
    categorical: bool,
    hidden: bool,
}

impl Column {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            categorical: false,
            hidden: false,
        }
    }

    pub fn categorical(mut self) -> Self {
        self.categorical = true;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }
}

#[component]
pub fn Table(props: TableProps) -> Element {
    let columns = use_signal(|| props.columns.clone());
    let mut search_text = use_signal(|| "".to_string());
    let mut custom_columns = use_signal(|| {
        props
            .columns
            .iter()
            .filter(|c| !c.hidden)
            .map(|c| c.name.clone())
            .collect::<Vec<_>>()
    });
    let mut sort_by = use_signal(|| props.columns[0].name.clone());
    let mut sort_ascending = use_signal(|| true);
    let mut column_search_text = use_signal(|| vec![String::new(); props.columns.len()]);
    let mut column_category_filter =
        use_signal(|| vec![HashSet::<String>::new(); props.columns.len()]);
    let mut drag_state = use_signal(|| DragState::None);
    use_effect(move || {
        // Run this effect when drag_state changes
        drag_state();
        // Rerun the anchor positioning polyfill
        document::eval("if (window.CSSAnchorPositioning) window.CSSAnchorPositioning()");
    });
    let cloned_data = props.data.clone();
    let filtered_data = use_memo(move || {
        let mut data = cloned_data
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                // Filter rows based on search text
                let search_text = search_text.read().to_lowercase();
                row.iter()
                    .any(|cell| cell.to_lowercase().contains(&search_text))
            })
            .filter(|(_, row)| {
                // Filter rows based on column-specific search text
                row.iter().enumerate().all(|(i, cell)| {
                    let filter_text = column_search_text
                        .read()
                        .get(i)
                        .unwrap_or(&String::new())
                        .to_lowercase();
                    let category_filter = column_category_filter.get(i).unwrap();
                    (filter_text.is_empty() && category_filter.is_empty())
                        || (!filter_text.is_empty() && cell.to_lowercase().contains(&filter_text))
                        || (!category_filter.is_empty() && category_filter.contains(cell))
                })
            })
            .sorted_by_key(|(_, row)| {
                // Sort by the column specified in sort_by
                let idx = columns
                    .read()
                    .iter()
                    .position(|h| &h.name == &sort_by())
                    .unwrap_or(0);
                row.get(idx).cloned().unwrap_or_default()
            })
            .map(|(id, row)| {
                // Collect only the custom columns
                (
                    id,
                    custom_columns
                        .read()
                        .iter()
                        .filter_map(|header| {
                            columns
                                .read()
                                .iter()
                                .position(|h| &h.name == header)
                                .and_then(|idx| row.get(idx).cloned())
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        if !sort_ascending() {
            data.reverse();
        }
        data
    });
    rsx! {
        div { class: "m-4 flex items-center gap-2",
            input {
                class: "border border-gray-300 rounded p-1",
                placeholder: "Search table...",
                value: "{search_text}",
                oninput: move |event| search_text.set(event.value()),
            }
            button {
                class: "border border-gray-300 rounded px-2 py-1 bg-gray-100 hover:bg-gray-200 [anchor-name:--customize-button]",
                popovertarget: "customize-popover",
                "Customize Columns"
            }
            div {
                // The anchor positioning polyfill requires inset-auto for whatever reason
                class: "border border-gray-300 rounded shadow-md p-2 absolute [position-anchor:--customize-button] [position-area:bottom_center] inset-auto",
                id: "customize-popover",
                popover: "auto",
                for header in props.columns.iter().cloned() {
                    label {
                        key: "{header.name}",
                        class: "flex items-center gap-2",
                        input {
                            r#type: "checkbox",
                            checked: custom_columns().contains(&header.name),
                            onchange: {
                                let header = header.clone();
                                move |_| {
                                    custom_columns
                                        .with_mut(|vec| {
                                            if vec.contains(&header.name) {
                                                vec.retain(|h| h != &header.name);
                                            } else {
                                                let pos = columns
                                                    .read()
                                                    .iter()
                                                    .position(|h| h.name == header.name)
                                                    .unwrap();
                                                let mut insert_at = vec.len();
                                                for (i, h) in vec.iter().enumerate() {
                                                    if let Some(col_pos) = columns
                                                        .read()
                                                        .iter()
                                                        .position(|c| &c.name == h)
                                                    {
                                                        if col_pos > pos {
                                                            insert_at = i;
                                                            break;
                                                        }
                                                    }
                                                }
                                                vec.insert(insert_at, header.name.clone());
                                            }
                                        });
                                }
                            },
                        }
                        span { "{header.name}" }
                    }
                }
                // Reset columns button
                button {
                    class: "border border-gray-300 rounded px-2 py-1 mt-2 bg-gray-100 hover:bg-gray-200 text-sm",
                    onclick: move |_| {
                        custom_columns
                            .set(
                                props
                                    .columns
                                    .iter()
                                    .filter(|c| !c.hidden)
                                    .map(|c| c.name.clone())
                                    .collect(),
                            );
                    },
                    "Reset Columns"
                }
            }
        }
        div {
            class: "grid gap-px p-px m-4",
            style: "grid-template-columns: repeat({custom_columns().len()}, auto)",
            div { class: "grid grid-cols-subgrid col-span-full",
                for (i , idx , header) in custom_columns()
                    .iter()
                    .enumerate()
                    .map(|(i, header)| (
                        i,
                        columns().iter().position(|c| &c.name == header).unwrap(),
                        header,
                    ))
                {
                    div {
                        key: "{header}",
                        class: "outline outline-gray-300 bg-gray-100 flex",
                        style: "anchor-name: --header-{i+1}",
                        // To make only the handle draggable, we need to set the draggable attribute conditionally
                        // https://stackoverflow.com/questions/26283661/drag-drop-with-handle
                        draggable: drag_state() == DragState::Mousedown(i),
                        ondragstart: move |_| {
                            if drag_state() == DragState::Mousedown(i) {
                                drag_state.set(DragState::Dragging(i));
                            }
                        },
                        ondragend: move |_| {
                            drag_state.set(DragState::None);
                        },
                        span { class: "font-bold px-2 py-1", "{header}" }
                        // Sort button
                        button {
                            class: "ml-auto flex items-center px-1",
                            onclick: {
                                let header = header.clone();
                                move |_| {
                                    if sort_by() == header.clone() {
                                        sort_ascending.set(!sort_ascending());
                                    } else {
                                        sort_by.set(header.clone());
                                        sort_ascending.set(true);
                                    }
                                }
                            },
                            svg {
                                class: if sort_by() == header.clone() { if sort_ascending() { "text-blue-500" } else { "text-blue-500 rotate-180" } } else { "" },
                                fill: "currentColor",
                                width: "24",
                                height: "24",
                                xmlns: "http://www.w3.org/2000/svg",
                                "viewBox": "0 -960 960 960",
                                path { d: "M480-528 296-344l-56-56 240-240 240 240-56 56z" }
                            }
                        }
                        // Filter button
                        button {
                            class: "flex items-center px-1 [anchor-name:filter-popover-{i}]",
                            class: if !column_search_text()[idx].is_empty() || !column_category_filter()[idx].is_empty() { "text-blue-500" },
                            popovertarget: "filter-popover-{i}",
                            svg {
                                fill: "currentColor",
                                "viewBox": "0 -960 960 960",
                                width: "24",
                                xmlns: "http://www.w3.org/2000/svg",
                                height: "24",
                                path { d: "M440-160q-17 0-28.5-11.5T400-200v-240L168-736q-15-20-4.5-42t36.5-22h560q26 0 36.5 22t-4.5 42L560-440v240q0 17-11.5 28.5T520-160zm40-308 198-252H282zm0 0" }
                            }
                        }
                        div {
                            // The anchor positioning polyfill requires inset-auto for whatever reason
                            class: "border border-gray-300 rounded shadow-md p-2 absolute min-w-50 [position-anchor:filter-popover-{i}] [position-area:bottom_center] inset-auto",
                            id: "filter-popover-{i}",
                            popover: "auto",
                            input {
                                class: "border border-gray-300 rounded p-1 w-full",
                                placeholder: "Filter by {header}",
                                value: column_search_text()[idx].clone(),
                                oninput: move |event: Event<FormData>| {
                                    column_search_text
                                        .with_mut(|vec| {
                                            vec[idx] = event.value();
                                        });
                                },
                            }
                            // Checkboxes for categorical filters
                            if columns().iter().find(|c| &c.name == header).unwrap().categorical {
                                div { class: "mt-2",
                                    for value in props
                                        .data
                                        .iter()
                                        .filter_map(|row| {
                                            row.get(columns().iter().position(|c| &c.name == header).unwrap())
                                        })
                                        .unique()
                                        .sorted()
                                    {
                                        label {
                                            key: "{value}",
                                            class: "flex items-center gap-2",
                                            input {
                                                r#type: "checkbox",
                                                checked: column_category_filter.get(idx).unwrap().contains(value),
                                                onchange: {
                                                    let value = value.clone();
                                                    move |_| {
                                                        column_category_filter
                                                            .with_mut(|filters| {
                                                                let filter = filters.get_mut(idx).unwrap();
                                                                if filter.contains(&value) {
                                                                    filter.remove(&value);
                                                                } else {
                                                                    filter.insert(value.clone());
                                                                }
                                                            });
                                                    }
                                                },
                                            }
                                            span { "{value}" }
                                        }
                                    }
                                }
                            }
                            // Reset filter button
                            button {
                                class: "border border-gray-300 rounded px-2 py-1 mt-2 bg-gray-100 hover:bg-gray-200 text-sm",
                                onclick: move |_| {
                                    column_search_text
                                        .with_mut(|vec| {
                                            vec[idx] = String::new();
                                        });
                                    column_category_filter
                                        .with_mut(|vec| {
                                            vec[idx] = HashSet::new();
                                        });
                                },
                                "Reset Filter"
                            }
                        }
                        // Drag handle
                        div {
                            class: "flex items-center px-1 cursor-grab",
                            onmousedown: move |_| {
                                drag_state.set(DragState::Mousedown(i));
                            },
                            onmouseup: move |_| {
                                drag_state.set(DragState::None);
                            },
                            svg {
                                "viewBox": "0 -960 960 960",
                                width: "24",
                                xmlns: "http://www.w3.org/2000/svg",
                                height: "24",
                                path { d: "M360-160q-33 0-56.5-23.5T280-240t23.5-56.5T360-320t56.5 23.5T440-240t-23.5 56.5T360-160m240 0q-33 0-56.5-23.5T520-240t23.5-56.5T600-320t56.5 23.5T680-240t-23.5 56.5T600-160M360-400q-33 0-56.5-23.5T280-480t23.5-56.5T360-560t56.5 23.5T440-480t-23.5 56.5T360-400m240 0q-33 0-56.5-23.5T520-480t23.5-56.5T600-560t56.5 23.5T680-480t-23.5 56.5T600-400M360-640q-33 0-56.5-23.5T280-720t23.5-56.5T360-800t56.5 23.5T440-720t-23.5 56.5T360-640m240 0q-33 0-56.5-23.5T520-720t23.5-56.5T600-800t56.5 23.5T680-720t-23.5 56.5T600-640" }
                            }
                        }
                    }
                }
                // Dragover indicator
                if let DragState::Dragover(_, i) = drag_state() {
                    div {
                        class: "absolute bg-blue-500 w-[3px]",
                        style: "top: anchor(--header-1 top); bottom: anchor(--header-1 bottom);",
                        style: if i == 0 { "left: anchor(--header-1 left); translate: -2px;" } else { "right: anchor(--header-{i} right); translate: 2px;" },
                    }
                }
                // Create invisible drop zones between headers
                match drag_state() {
                    DragState::Dragging(dragged_index) | DragState::Dragover(dragged_index, _) => {
                        rsx! {
                            for i in 0..custom_columns().len() + 1 {
                                if i != dragged_index && i != dragged_index + 1 {
                                    div {
                                        class: "absolute",
                                        style: "left: anchor(--header-{i} center, 0); right: anchor(--header-{i+1} center, 0); top: anchor(--header-1 -50%); bottom: anchor(--header-1 150%);",
                                        ondragover: move |event| {
                                            event.prevent_default();
                                            if drag_state() != DragState::Dragover(dragged_index, i) {
                                                drag_state.set(DragState::Dragover(dragged_index, i));
                                            }
                                        },
                                        ondragleave: move |_| {
                                            drag_state.set(DragState::Dragging(dragged_index));
                                        },
                                        ondrop: move |_| {
                                            custom_columns
                                                .with_mut(|cols| {
                                                    let col = cols.remove(dragged_index);
                                                    let insert_at = if i > dragged_index { i - 1 } else { i };
                                                    cols.insert(insert_at, col);
                                                });
                                        },
                                    }
                                }
                            }
                        }
                    }
                    _ => rsx! {},
                }
            }
            for (id , row) in filtered_data().into_iter() {
                div { class: "grid grid-cols-subgrid col-span-full hover:bg-gray-50",
                    for cell in row.iter() {
                        div {
                            class: "outline outline-gray-300 px-2 py-1",
                            onclick: move |_| {
                                (props.ondetail)(id);
                            },
                            "{cell}"
                        }
                    }
                }
            }
        }
    }
}
