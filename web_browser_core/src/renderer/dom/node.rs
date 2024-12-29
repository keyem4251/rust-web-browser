use alloc::rc::Rc;
use alloc::rc::Weak;
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,                      // ノードの種類
    window: Weak<RefCell<Window>>, // 1つのページに対して1つのウィンドウを持つ。DOMツリーを持つウィンドウ。
    parent: Weak<RefCell<Node>>,   // ノードの親ノード
    first_child: Option<Rc<RefCell<Node>>>, // ノードの最初の子ノード
    last_child: Weak<RefCell<Node>>, // ノードの最後の子ノード
    previous_sibling: Weak<RefCell<Node>>, // ノードの前の兄弟ノード
    next_sibling: Option<Rc<RefCell<Node>>>, // ノードの次の兄弟ノード
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            window: Weak::new(),
            parent: Weak::new(),
            first_child: None,
            last_child: Weak::new(),
            previous_sibling: Weak::new(),
            next_sibling: None,
        }
    }
}
