use crate::node::Node;

pub(crate) struct NodeSerial {
    pub(crate) id: String,
    pub(crate) children: Vec<Node>,
}
