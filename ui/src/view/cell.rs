use crate::style::Style;
use crate::view::text::text;
use seed::dom_entity_names::Tag;
use seed::prelude::{mouse_ev, El, Ev, MessageMapper, Node};
use shared::point::Point;
use std::borrow::Cow;
use std::rc::Rc;
use web_sys::MouseEvent;

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
    mouse_down_handler: Option<Rc<dyn Fn(MouseEvent) -> Msg>>,
    mouse_up_handler: Option<Rc<dyn Fn(MouseEvent) -> Msg>>,
    mouse_move_handler: Option<Rc<dyn Fn(MouseEvent) -> Msg>>,
    click_handler: Option<Rc<dyn Fn(MouseEvent) -> Msg>>,
    as_img: Option<String>,
    screen_pos: Option<Point<i16>>,
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

    pub fn empty(styles: Vec<Style>) -> Cell<Msg> {
        Cell::group(styles, vec![])
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

                if let Some(img_source) = model.as_img {
                    element.add_attr(Cow::Borrowed("src"), img_source);
                }

                for child in model.children {
                    element.children.push(child);
                }

                if let Some(msg) = model.mouse_down_handler {
                    element.add_event_handler(mouse_ev(Ev::MouseDown, move |event| msg(event)));
                }

                if let Some(msg) = model.mouse_up_handler {
                    element.add_event_handler(mouse_ev(Ev::MouseUp, move |event| msg(event)));
                }

                if let Some(msg) = model.mouse_move_handler {
                    element.add_event_handler(mouse_ev(Ev::MouseMove, move |event| msg(event)));
                }

                if let Some(msg) = model.click_handler {
                    element.add_event_handler(mouse_ev(Ev::Click, move |event| msg(event)));
                }

                if let Some(screen_pos) = model.screen_pos {
                    let mut style_str = "position:absolute; ".to_string();
                    style_str.push_str("top:");
                    style_str.push_str(screen_pos.y.to_string().as_str());
                    style_str.push_str("px; ");
                    style_str.push_str("left:");
                    style_str.push_str(screen_pos.x.to_string().as_str());
                    style_str.push_str("px; ");

                    element.add_attr(Cow::Borrowed("style"), style_str);
                }

                Node::Element(element)
            }
        }
    }

    pub fn with_tag_name(self, tag_name: &'static str) -> Cell<Msg> {
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

    pub fn on_mouse_down(self, msg: impl FnOnce(MouseEvent) -> Msg + Clone + 'static) -> Cell<Msg> {
        self.on_mouse_down_helper(Some(msg))
    }

    fn on_mouse_down_helper(
        self,
        maybe_msg: Option<impl FnOnce(MouseEvent) -> Msg + Clone + 'static>,
    ) -> Cell<Msg> {
        match self {
            Cell::None => Cell::None,
            Cell::Model(mut model) => match maybe_msg {
                Some(msg) => {
                    model.mouse_down_handler = Some(Rc::new(move |event| msg.clone()(event)));
                    Cell::Model(model)
                }
                None => Cell::Model(model),
            },
        }
    }

    pub fn on_mouse_up(self, msg: impl FnOnce(MouseEvent) -> Msg + Clone + 'static) -> Cell<Msg> {
        self.on_mouse_up_helper(Some(msg))
    }

    fn on_mouse_up_helper(
        self,
        maybe_msg: Option<impl FnOnce(MouseEvent) -> Msg + Clone + 'static>,
    ) -> Cell<Msg> {
        match self {
            Cell::None => Cell::None,
            Cell::Model(mut model) => match maybe_msg {
                Some(msg) => {
                    model.mouse_up_handler = Some(Rc::new(move |event| msg.clone()(event)));
                    Cell::Model(model)
                }
                None => Cell::Model(model),
            },
        }
    }

    pub fn on_mouse_move(self, msg: impl FnOnce(MouseEvent) -> Msg + Clone + 'static) -> Cell<Msg> {
        self.on_mouse_move_helper(Some(msg))
    }

    fn on_mouse_move_helper(
        self,
        maybe_msg: Option<impl FnOnce(MouseEvent) -> Msg + Clone + 'static>,
    ) -> Cell<Msg> {
        match self {
            Cell::None => Cell::None,
            Cell::Model(mut model) => match maybe_msg {
                Some(msg) => {
                    model.mouse_move_handler = Some(Rc::new(move |event| msg.clone()(event)));
                    Cell::Model(model)
                }
                None => Cell::Model(model),
            },
        }
    }

    pub fn on_click(self, msg: impl FnOnce(MouseEvent) -> Msg + Clone + 'static) -> Cell<Msg> {
        self.on_click_helper(Some(msg))
    }

    fn on_click_helper(
        self,
        maybe_msg: Option<impl FnOnce(MouseEvent) -> Msg + Clone + 'static>,
    ) -> Cell<Msg> {
        match self {
            Cell::None => Cell::None,
            Cell::Model(mut model) => match maybe_msg {
                Some(msg) => {
                    model.click_handler = Some(Rc::new(move |event| msg.clone()(event)));
                    Cell::Model(model)
                }
                None => Cell::Model(model),
            },
        }
    }

    pub fn at_screen_pos(self, x: i16, y: i16) -> Cell<Msg> {
        match self {
            Cell::None => Cell::None,
            Cell::Model(mut model) => {
                model.screen_pos = Some(Point { x, y });

                Cell::Model(model)
            }
        }
    }

    pub fn with_img_src(self, source: String) -> Cell<Msg> {
        match self {
            Cell::None => Cell::None,
            Cell::Model(mut model) => {
                model.as_img = Some(source);
                model.tag_name = "img";
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
            mouse_down_handler: None,
            mouse_up_handler: None,
            mouse_move_handler: None,
            click_handler: None,
            as_img: None,
            screen_pos: None,
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

                let new_on_mouse_down = model.mouse_down_handler.map(|msg| {
                    let msg_mapper = f.clone();
                    move |event| msg_mapper(msg(event))
                });

                let new_on_mouse_up = model.mouse_up_handler.map(|msg| {
                    let msg_mapper = f.clone();
                    move |event| msg_mapper(msg(event))
                });

                let new_on_mouse_move = model.mouse_move_handler.map(|msg| {
                    let msg_mapper = f.clone();
                    move |event| msg_mapper(msg(event))
                });

                let new_on_click = model.click_handler.map(|msg| {
                    let msg_mapper = f.clone();
                    move |event| msg_mapper(msg(event))
                });

                Cell::new(model.styles, new_children, model.tag_name)
                    .on_mouse_down_helper(new_on_mouse_down)
                    .on_mouse_up_helper(new_on_mouse_up)
                    .on_mouse_move_helper(new_on_mouse_move)
                    .on_click_helper(new_on_click)
            }
        }
    }

    fn new(styles: Vec<Style>, children: Vec<Node<Msg>>, tag_name: &'static str) -> Cell<Msg> {
        Cell::Model(Model {
            styles,
            children,
            tag_name,
            mouse_down_handler: None,
            mouse_up_handler: None,
            mouse_move_handler: None,
            click_handler: None,
            as_img: None,
            screen_pos: None,
        })
    }
}
