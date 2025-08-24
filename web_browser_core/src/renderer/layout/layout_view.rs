use core::cell::RefCell;

use alloc::rc::Rc;

use crate::renderer::{css::{self, cssom::StyleSheet}, dom::node::{ElementKind, Node}, layout::layout_object::{LayoutObject, LayoutObjectKind, LayoutPoint, LayoutSize}};

#[derive(Debug, Clone)]
pub struct LayoutView {
    root: Option<Rc<RefCell<LayoutObject>>>
}

impl LayoutView {
    pub fn new(
        root: Rc<RefCell<Node>>,
        cssom: &StyleSheet,
    ) -> Self {
        // レイアウトツリーは描画される要素だけを持つツリーなので
        // <body>タグを取得し、その子要素以下をレイアウトツリーのノードに変換する
        let body_root = get_target_element_node(Some(root), ElementKind::Body);
        let mut tree = Self {
            root: build_layout_tree(&body_root, &None, cssom)
        };
        tree.update_layout();
        tree
    }

    pub fn root(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.root.clone()
    }

    fn update_layout(&mut self) {
        Self::calculate_node_size(&self.root, LayoutSize::new(CONTENT_AREA_WIDTH, 0));
        Self::calculate_node_position(
            &self.root,
            LayoutPoint::new(0, 0),
            LayoutObjectKind::Block,
            None,
            None,
        );
    }

    fn calculate_node_size(node: &Option<Rc<RefCell<LayoutObject>>>, parent_size: LayoutSize) {
        if let Some(n) = node {
            // ノードがブロック要素の場合、子ノードのレイアウトを計算する前に横幅を決める
            if n.borrow().kind() == LayoutObjectKind::Block {
                // 親のノードのサイズによって横幅を決定する
                // ブロック要素は親の横幅いっぱいまで広がるので親ノードの横幅と同等になる
                n.borrow_mut().compute_size(parent_size);
            }

            let first_child = n.borrow().first_child();
            Self::calculate_node_size(&first_child, n.borrow().size());
            let next_sibling = n.borrow().next_sibling();
            Self::calculate_node_size(&next_sibling, parent_size);

            // 子ノードのサイズが決まったあとにサイズを計算する
            // ブロック要素のとき、高さは子ノードの高さに依存する
            // インライン要素のとき、高さも横幅も子ノードに依存する
            // 子要素のサイズが決定したあとなので子要素のサイズを下に高さを決定する
            n.borrow_mut().compute_size(parent_size);
        }
    }
}

fn build_layout_tree(
    node: &Option<Rc<RefCell<Node>>>,
    parent_obj: &Option<Rc<RefCell<LayoutObject>>>,
    cssom: &StyleSheet,
) -> Option<Rc<RefCell<LayoutObject>>> {
    // create_layout_object関数によってノードとなるLayoutObjectの作成を行う
    // CSSによってdisplay:noneが指定されていたらノードは作成されない
    let mut target_node = node.clone();
    // 与えられたDOMノードに対して対応するレイアウトオブジェクトを作る
    let mut layout_object = create_layout_object(node, parent_obj, cssom);

    // もしノードが作成されなかった場合DOMノードの兄弟ノードを使用して
    // LayoutObjectの作成を行う。LayoutObjectが作成されるまで兄弟ノードをたどる。
    // 最初に作成したレイアウトオブジェクトが存在しない場合
    while layout_object.is_none() {
        if let Some(n) = target_node {
            target_node = n.borrow().next_sibling().clone();
            // 兄弟ノードに繰り返しレイアウトオブジェクトの作成を試みる
            layout_object = create_layout_object(&target_node, parent_obj, cssom)
        } else {
            // もし兄弟ノードがない場合、処理するDOMツリーは終了したので
            // 今まで作成したレイアウトツリーを返す
            return layout_object;
        }
    }

    if let Some(n) = target_node {
        let original_first_child = n.borrow().first_child();
        let original_next_sibling = n.borrow().next_sibling();
        let mut first_child = build_layout_tree(&original_first_child, &layout_object, cssom);
        let mut next_sibling = build_layout_tree(&original_next_sibling, &None, cssom);

        // もし子ノードにdisplay:nodeが指定されていた場合
        // LayoutObjectは作成されないため、子ノードの兄弟ノードを使用してLayoutObjectを作成する
        // LayoutObjectが作成されるか、たどるべき兄弟ノードがなくなるまで処理を繰り返す
        if first_child.is_none() && original_first_child.is_some() {
            let mut original_dom_node = original_first_child.expect("first child should exist").borrow().next_sibling();

            loop {
                first_child = build_layout_tree(&original_dom_node, &layout_object, cssom);
                if first_child.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node.expect("next sibling should exist").borrow().next_sibling();
                    continue;
                }
                break;
            }
        }

        // もし兄弟ノードにdisplay:nodeが指定されていた場合LayoutObjectは作成されないため
        // 兄弟ノードの兄弟ノードを使用してLayoutObjectの作成を行う
        // LayoutObjectが作成されるか、たどるべき兄弟ノードがなくなるまで処理を繰り返す
        if next_sibling.is_none() && n.borrow().next_sibling().is_some() {
            let mut original_dom_node = original_next_sibling.expect("first child should exist").borrow().next_sibling();

            loop {
                next_sibling = build_layout_tree(&original_dom_node, &None, cssom);
                if next_sibling.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node.expect("next sibling should exist").borrow().next_sibling();
                    continue;
                }

                break;
            }
        }

        // 作成した子と兄弟のレイアウトオブジェクトを追加する
        let obj: &Rc<RefCell<LayoutObject>> = match &layout_object {
            Some(ref obj) => obj,
            None => panic!("render object should exist here"),
        };
        obj.borrow_mut().set_first_child(first_child);
        obj.borrow_mut().set_next_sibling(next_sibling);
    }

    layout_object
}
