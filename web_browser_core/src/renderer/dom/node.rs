use alloc::rc::Rc;
use alloc::rc::Weak;
use alloc::string::String;
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

    pub fn set_parent(&mut self, parent: Weak<RefCell<Node>>) {
        self.parent = parent;
    }

    pub fn parent(&self) -> Weak<RefCell<Node>> {
        self.parent.clone()
    }

    pub fn set_first_child(&mut self, first_child: Option<Rc<RefCell<Node>>>) {
        self.first_child = first_child;
    }

    pub fn first_child(&self) -> Option<Rc<RefCell<Node>>>{
        self.first_child.as_ref().cloned()
    }

    pub fn set_last_child(&mut self, last_child: Weak<RefCell<Node>>) {
        self.last_child = last_child;
    }

    pub fn last_child(&self) -> Weak<RefCell<Node>> {
        self.last_child.clone()
    }

    pub fn set_previous_sibling(&mut self, previous_sibling: Weak<RefCell<Node>>) {
        self.previous_sibling = previous_sibling;
    }

    pub fn previous_sibling(&self) -> Weak<RefCell<Node>> {
        self.previous_sibling.clone()
    }

    pub fn set_next_sibling(&mut self, next_sibling: Option<Rc<RefCell<Node>>>) {
        self.next_sibling = next_sibling;
    }

    pub fn next_sibling(&self) -> Option<Rc<RefCell<Node>>>{
        self.next_sibling.as_ref().cloned()
    }
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    // HTML文書のDOMツリーのルート要素。getElementByIdやappendChildなどでDOMツリーの操作を行う
    Document, // https://dom.spec.whatwg.org/#interface-document
    
    // <p>タグなど。tagName、getAttributeなどでタグの情報を取得できる
    Element(Element), // https://dom.spec.whatwg.org/#interface-element DOMツリー内の要素ノード

    // 要素内のテキストコンテンツを表す。
    Text(String), // https://dom.spec.whatwg.org/#interface-text
}


