use core::cell::RefCell;

use alloc::{rc::Rc, vec::Vec};

use crate::renderer::dom::node::{Element, ElementKind, Node, NodeKind};

pub fn get_target_element_node(
    node: Option<Rc<RefCell<Node>>>,
    element_kind: ElementKind,
) -> Option<Rc<RefCell<Node>>> {
    match node {
        Some(n) => {
            if n.borrow().kind() == NodeKind::Element(Element::new(&element_kind.to_string(), Vec::new())) {
                // 1
                return Some(n.clone());
            }
            // 2
            let result1 = get_target_element_node(n.borrow().first_child(), element_kind);
            // 3
            let result2 = get_target_element_node(n.borrow().next_sibling(), element_kind);
            if result1.is_none() && result2.is_none() {
                return None;
            }
            if result1.is_none() {
                return result2;
            }
            result1
        }
        None => None,
    }
}