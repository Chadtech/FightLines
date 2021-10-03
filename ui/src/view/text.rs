use seed::prelude::Node;
use seed::virtual_dom::Text;

pub fn text<Msg>(str: &str) -> Node<Msg> {
    let node_text: Text = Text::new(str.to_string());
    Node::Text(node_text)
}
