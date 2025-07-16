use alloc::format;
use alloc::rc::Rc;
use alloc::rc::Weak;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::str::FromStr;

use crate::renderer::html::attribute::Attribute;

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

    pub fn first_child(&self) -> Option<Rc<RefCell<Node>>> {
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

    pub fn next_sibling(&self) -> Option<Rc<RefCell<Node>>> {
        self.next_sibling.as_ref().cloned()
    }

    pub fn set_window(&mut self, window: Weak<RefCell<Window>>) {
        self.window = window;
    }

    pub fn kind(&self) -> NodeKind {
        self.kind.clone()
    }

    pub fn get_element(&self) -> Option<Element> {
        match &self.kind {
            NodeKind::Document | NodeKind::Text(_) => None,
            NodeKind::Element(ref e) => Some(e.clone()),
        }
    }

    pub fn element_kind(&self) -> Option<ElementKind> {
        match &self.kind {
            NodeKind::Document | NodeKind::Text(_) => None,
            NodeKind::Element(ref e) => Some(e.kind()),
        }
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

// https://html.spec.whatwg.org/multipage/nav-history-apis.html#window
#[derive(Debug, Clone)]
pub struct Window {
    document: Rc<RefCell<Node>>,
}

impl Window {
    pub fn new() -> Self {
        let window = Self {
            document: Rc::new(RefCell::new(Node::new(NodeKind::Document))),
        };

        window
            .document
            .borrow_mut()
            .set_window(Rc::downgrade(&Rc::new(RefCell::new(window.clone()))));
        window
    }

    pub fn document(&self) -> Rc<RefCell<Node>> {
        self.document.clone()
    }
}

// https://dom.spec.whatwg.org/#interface-element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    kind: ElementKind,
    attributes: Vec<Attribute>,
}

impl Element {
    pub fn new(element_name: &str, attributes: Vec<Attribute>) -> Self {
        Self {
            kind: ElementKind::from_str(element_name)
                .expect("failed to convert string to ElementKind"),
            attributes,
        }
    }

    pub fn kind(&self) -> ElementKind {
        self.kind
    }
}

// https://dom.spec.whatwg.org/#interface-element
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ElementKind {
    Html,   // https://html.spec.whatwg.org/multipage/semantics.html#the-html-element
    Head,   // https://html.spec.whatwg.org/multipage/semantics.html#the-head-element
    Style,  // https://html.spec.whatwg.org/multipage/semantics.html#the-style-element
    Script, // https://html.spec.whatwg.org/multipage/scripting.html#the-script-element
    Body,   // https://html.spec.whatwg.org/multipage/sections.html#the-body-element
    P,      // https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element
    H1,     // https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements
    H2,     // https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements
}

impl FromStr for ElementKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "html" => Ok(Self::Html),
            "head" => Ok(Self::Head),
            "style" => Ok(Self::Style),
            "script" => Ok(Self::Script),
            "body" => Ok(Self::Body),
            "p" => Ok(ElementKind::P),
            "h1" => Ok(ElementKind::H1),
            "h2" => Ok(ElementKind::H2),
            _ => Err(format!("unimplemented element name {:?}", s)),
        }
    }
}
