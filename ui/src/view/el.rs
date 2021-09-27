// use seed::prelude::*;
// use seed::virtual_dom::Text;
//
///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

pub struct El;

//
// #[derive(Clone, Copy)]
// pub enum Style {}
//
// // pub enum Attr<Msg> {
// //     OnClick(Msg),
// // }
// pub struct El<Msg>
// where
//     Msg: Clone,
// {
//     styles: Vec<Style>,
//     attrs: Vec<Attr<Msg>>,
//     body: Body<Msg>,
// }
//
// enum Attr<Msg>
// where
//     Msg: Clone,
// {
//     OnClick(Msg),
// }
//
// enum Body<Msg>
// where
//     Msg: Clone,
// {
//     Els(Vec<El<Msg>>),
//     Text(String),
// }
//
// impl<Msg> El<Msg>
// where
//     Msg: Clone,
// {
//     pub fn into_html(self) -> Node<Msg> {
//         let node_text: Text = Text::new("POW");
//         Node::Text(node_text)
//     }
// }
//
// impl<Msg> Clone for Attr<Msg>
// where
//     Msg: Clone,
// {
//     fn clone(&self) -> Self {
//         match self {
//             Attr::OnClick(msg) => Attr::OnClick(msg),
//         }
//     }
// }
//
// impl<Msg> Clone for El<Msg>
// where
//     Msg: Clone,
// {
//     fn clone(&self) -> Self {
//         Self {
//             styles: self.styles.clone(),
//             attrs: self.attrs.clone(),
//             body: self.body.clone(),
//         }
//     }
// }
//
// impl<Msg> Clone for Body<Msg>
// where
//     Msg: Clone,
// {
//     fn clone(&self) -> Self {
//         match self {
//             Body::Els(els) => Body::Els(els.clone()),
//             Body::Text(text) => Body::Text(text.clone()),
//         }
//     }
// }
//
// impl<Msg: 'static, OtherMs: 'static> MessageMapper<Msg, OtherMs> for El<Msg> {
//     type SelfWithOtherMs = El<OtherMs>;
//     /// See note on impl for El
//     fn map_msg(self, f: impl FnOnce(Msg) -> OtherMs + 'static + Clone) -> El<OtherMs> {
//         El {
//             styles: self.styles,
//             attrs: self
//                 .attrs
//                 .iter()
//                 .map(|attr| attr.map_msg(f.clone()))
//                 .collect(),
//             body: self.body.map_msg(f),
//         }
//         // match self {
//         //     Node::Element(el) => Node::Element(el.map_msg(f)),
//         //     Node::Text(text) => Node::Text(text),
//         //     Node::Empty => Node::Empty,
//         //     Node::NoChange => Node::NoChange,
//         // }
//     }
// }
//
// impl<Msg: 'static, OtherMs: 'static> MessageMapper<Msg, OtherMs> for Body<Msg> {
//     type SelfWithOtherMs = Body<OtherMs>;
//     /// See note on impl for El
//     fn map_msg(self, f: impl FnOnce(Msg) -> OtherMs + 'static + Clone) -> Body<OtherMs> {
//         match self {
//             Body::Text(text) => Body::Text(text),
//             Body::Els(els) => Body::Els(els.iter().map(|el| el.map_msg(f)).collect()),
//         }
//     }
// }
//
// impl<Msg: 'static, OtherMs: 'static> MessageMapper<Msg, OtherMs> for Attr<Msg> {
//     type SelfWithOtherMs = Attr<OtherMs>;
//     /// See note on impl for El
//     fn map_msg(self, f: impl FnOnce(Msg) -> OtherMs + 'static + Clone) -> Attr<OtherMs> {
//         match self {
//             Attr::OnClick(msg) => Attr::OnClick(f(msg)),
//         }
//     }
// }
