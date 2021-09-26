use seed::prelude::*;
use seed::virtual_dom::Text;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

pub enum Style {}

pub enum Attr<Msg> {
    OnClick(Msg),
}

pub struct El<Msg> {
    styles: Vec<Style>,
    attrs: Vec<Attr<Msg>>,
    children: Vec<El<Msg>>,
}

enum Body<Msg> {
    Row(Vec<El<Msg>>),
    Col(Vec<El<Msg>>),
    Text(String),
}

impl<Msg> Into<Node<Msg>> for El<Msg> {
    fn into(self) -> Node<Msg> {
        let node_text: Text = Text::new("POW");
        Node::Text(node_text)
    }
}


impl<Msg: 'static, OtherMs: 'static> MessageMapper<Ms, OtherMs> for El<Msg> {
    type SelfWithOtherMs = Node<OtherMs>;
    /// See note on impl for El
    fn map_msg(self, f: impl FnOnce(Msg) -> OtherMs + 'static + Clone) -> El<OtherMs> {
        El {
            styles: self.styles,
            attrs:
        }
        // match self {
        //     Node::Element(el) => Node::Element(el.map_msg(f)),
        //     Node::Text(text) => Node::Text(text),
        //     Node::Empty => Node::Empty,
        //     Node::NoChange => Node::NoChange,
        // }
    }
}
