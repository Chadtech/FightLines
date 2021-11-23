use crate::style::Style;
use crate::view::text::text;
use seed::dom_entity_names::Tag;
use seed::prelude::{El, MessageMapper, Node};
use std::borrow::Cow;

////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum Cell<Msg> {
    None,
    Model(Model<Msg>),
}

#[derive(Clone)]
pub struct Model<Msg> {
    styles: Vec<Style>,
    children: Vec<Node<Msg>>,
    tag_name: &'static str,
}

#[derive(Clone)]
pub struct Row<Msg>(Cell<Msg>);

////////////////////////////////////////////////////////////////
// Helpers //
////////////////////////////////////////////////////////////////

const DEFAULT_TAG_NAME: &str = "cell";

const ROW_TAG_NAME: &str = "row";

const STRING_TAG_NAME: &str = "string";

////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////

impl<Msg: 'static> Row<Msg> {
    pub fn none() -> Row<Msg> {
        Row(Cell::none())
    }

    fn cell(self) -> Cell<Msg> {
        self.0
    }

    pub fn from_cells(mut styles: Vec<Style>, cells: Vec<Cell<Msg>>) -> Row<Msg> {
        styles.push(Style::FlexRow);

        Row(Cell::group_in_tag_name(styles, cells, ROW_TAG_NAME))
    }

    pub fn from_str(s: &str) -> Row<Msg> {
        Row::from_cells(vec![], vec![Cell::from_str(vec![], s)])
    }

    pub fn map_msg<OtherMsg: 'static>(
        self,
        f: impl FnOnce(Msg) -> OtherMsg + 'static + Clone,
    ) -> Row<OtherMsg> {
        Row(self.0.map_msg(f))
    }
}

impl<Msg: 'static> Cell<Msg> {
    pub fn none() -> Cell<Msg> {
        Cell::None
    }

    pub fn html(self) -> Node<Msg> {
        match self {
            Cell::None => text(""),
            Cell::Model(model) => {
                let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed(model.tag_name)));

                for style in model.styles {
                    for class in style.css_classes() {
                        element.add_class(class);
                    }
                }

                for child in model.children {
                    element.children.push(child);
                }

                Node::Element(element)
            }
        }
    }

    pub fn with_tag_name(mut self, tag_name: &'static str) -> Cell<Msg> {
        match self {
            Cell::None => Cell::None,
            Cell::Model(mut model) => {
                model.tag_name = tag_name;

                Cell::Model(model)
            }
        }
    }

    pub fn with_styles(self, mut styles: Vec<Style>) -> Cell<Msg> {
        match self {
            Cell::None => Cell::None,
            Cell::Model(mut model) => {
                model.styles.append(&mut styles);
                Cell::Model(model)
            }
        }
    }

    pub fn from_html(styles: Vec<Style>, html: Vec<Node<Msg>>) -> Cell<Msg> {
        Cell::new(styles, html, DEFAULT_TAG_NAME)
    }

    pub fn from_rows(mut styles: Vec<Style>, rows: Vec<Row<Msg>>) -> Cell<Msg> {
        let row_cells = rows.into_iter().map(|row| row.cell()).collect();

        styles.push(Style::FlexCol);

        Cell::group(styles, row_cells)
    }

    pub fn group(styles: Vec<Style>, children: Vec<Cell<Msg>>) -> Cell<Msg> {
        let html_children = children.into_iter().map(|cell| cell.html()).collect();

        Cell::new(styles, html_children, DEFAULT_TAG_NAME)
    }

    fn group_in_tag_name(
        styles: Vec<Style>,
        children: Vec<Cell<Msg>>,
        tag_name: &'static str,
    ) -> Cell<Msg> {
        let html_children = children.into_iter().map(|cell| cell.html()).collect();

        Cell::Model(Model {
            styles,
            children: html_children,
            tag_name,
        })
    }

    pub fn from_str(styles: Vec<Style>, text_content: &str) -> Cell<Msg> {
        Cell::new(styles, vec![text(text_content)], STRING_TAG_NAME)
    }

    pub fn map_msg<OtherMsg: 'static>(
        self,
        f: impl FnOnce(Msg) -> OtherMsg + 'static + Clone,
    ) -> Cell<OtherMsg> {
        match self {
            Cell::None => Cell::None,
            Cell::Model(model) => {
                let new_children = model
                    .children
                    .into_iter()
                    .map(|html| html.map_msg(f.clone()))
                    .collect();
                Cell::new(model.styles, new_children, model.tag_name)
            }
        }
    }

    fn new(styles: Vec<Style>, children: Vec<Node<Msg>>, tag_name: &'static str) -> Cell<Msg> {
        Cell::Model(Model {
            styles,
            children,
            tag_name,
        })
    }
}
