use crate::renderer::dom::node::Element;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use crate::renderer::dom::node::Window;
use crate::renderer::html::attribute::Attribute;
use crate::renderer::html::token::{HtmlTokenizer, HtmlToken};
use alloc::collections::vec_deque::VecDeque;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::Ref;
use core::cell::RefCell;
use core::str::FromStr;

#[derive(Debug, Clone)]
pub struct HtmlParser {
    window: Rc<RefCell<Window>>, // Domツリーのルートノードを持つWindowオブジェクト
    mode: InsertionMode, // 状態遷移で使用される現在の状態
    // https://html.spec.whatwg.org/multipage/parsing.html#original-insertion-mode
    original_insertion_mode: InsertionMode, // とある状態に遷移したときに以前の挿入モードを保存するためのフィールド
    // https://html.spec.whatwg.org/multipage/parsing.html#the-stack-of-open-elements
    stack_of_open_elements: Vec<Rc<RefCell<Node>>>, // HTMLの構文解析中にブラウザが使用するスタック。常に最も深い階層の開いている要素が位置する
    t: HtmlTokenizer,
}

impl HtmlParser {
    pub fn new(t: HtmlTokenizer) -> Self {
        Self {
            window: Rc::new(RefCell::new(Window::new())),
            mode: InsertionMode::Initial,
            original_insertion_mode: InsertionMode::Initial,
            stack_of_open_elements: Vec::new(),
            t,
        }
    }

    pub fn construct_tree(&mut self) -> Rc<RefCell<Window>> {
        let mut token = self.t.next();
        while token.is_some() {
            match self.mode {
                // DOCTYPEトークンをサポートしないため<!doctype html>のようなトークンは文字トークンとして表される。
                // 文字トークンは無視する
                InsertionMode::Initial => {
                    if let Some(HtmlToken::Char(_)) = token {
                        token = self.t.next();
                        continue;
                    }

                    self.mode = InsertionMode::BeforeHtml;
                    continue;
                }
                InsertionMode::BeforeHtml => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            if c == ' ' || c == '\n' { // 空文字、改行文字は無視する
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            if tag == "html" { // HtmlTokenの開始タグがhtmlの場合
                                // DOMツリーに新しいノードを追加する
                                self.insert_element(tag, attributes.to_vec());
                                self.mode = InsertionMode::BeforeHead;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => { // EOFトークンが来たら
                            // 今まで構築していたDOMツリーを返す
                            return self.window.clone();
                        }
                        _ => {}
                    }
                    // それ以外のトークンの場合は自動的にHTML要素をDOMツリーに追加する
                    self.insert_element("html", Vec::new());
                    self.mode = InsertionMode::BeforeHead;
                    continue;
                }
                InsertionMode::BeforeHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            if c == ' ' || c == '\n' {
                                // 次のトークンが空文字、開業のときは次のトークンに移動する
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            if tag == "head" {
                                self.insert_element(tag, attributes.to_vec());
                                self.mode = InsertionMode::InHead;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                    // それ以外の場合、HEAD要素をDOMツリーに追加する
                    self.insert_element("head", Vec::new());
                    self.mode = InsertionMode::InHead;
                    continue;
                }
                InsertionMode::InHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            // 次のトークンが空文字、改行のときは次のトークンに移動
                            if c == ' ' || c == '\n' {
                                self.insert_char(c);
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            if tag == "style" || tag == "script" {
                                // タグの名前がstyle、scriptだったとき新しいノードを追加して、Text状態に遷移する
                                self.insert_element(tag, attributes.to_vec());
                                self.original_insertion_mode = self.mode;
                                self.mode = InsertionMode::Text;
                                token = self.t.next();
                                continue;
                            }
                            
                            if tag == "body" {
                                // このブラウザがすべての仕様を実装していないので、headが省略されているHTMLを扱うのに必要
                                // これがないとheadが省略されているHTMLで無限ループが発生
                                self.pop_until(ElementKind::Head);
                                self.mode = InsertionMode::AfterHead;
                                continue;
                            }

                            if let Ok(_element_kind) = ElementKind::from_str(tag) {
                                // このブラウザがすべての仕様を実装していないので、headが省略されているHTMLを扱うのに必要
                                // これがないとheadが省略されているHTMLで無限ループが発生
                                self.pop_until(ElementKind::Head);
                                self.mode = InsertionMode::AfterHead;
                                continue;
                            }
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            // 次のトークンがEndTagでheadの場合、すたっくに保存されているノードを取り出し
                            // 次の状態のAfterHeadに遷移する
                            if tag == "head" {
                                self.mode = InsertionMode::AfterHead;
                                token = self.t.next();
                                self.pop_until(ElementKind::Head);
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                    // meta、titleなどのサポートしていないタグは無視する
                    token = self.t.next();
                    continue;

                }
                InsertionMode::AfterHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            if c == ' ' || c == '\n' {
                                // 次のトークンが空文字、改行のときは無視する
                                self.insert_char(c);
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            if tag == "body" {
                                // 次のトークンがStartTagでbodyのときにDOMツリーに新しいノードを追加
                                // InBodyに状態を遷移する
                                self.insert_element(tag, attributes.to_vec());
                                token = self.t.next();
                                self.mode = InsertionMode::InBody;
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                    // それ以外の場合、body要素をDOMツリーに追加する
                    self.insert_element("body", Vec::new());
                    self.mode = InsertionMode::InBody;
                    continue;
                }
                InsertionMode::InBody => {
                    match token {
                        Some(HtmlToken::StartTag { 
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => match tag.as_str() {
                            "p" => {
                                // タグの名前がpのときにElementノードを作成しDOMツリーに追加する
                                // その後トークンを次に進める
                                self.insert_element(tag, attributes.to_vec());
                                token = self.t.next();
                                continue;
                            }
                            "h1" | "h2" => {
                                // h1、h2の開始タグが現れたら、DOMツリーに追加する
                                self.insert_element(tag, attributes.to_vec());
                                token = self.t.next();
                                continue;
                            }
                            "a" => {
                                // aの開始タグが現れたら、DOMツリーに追加する
                                self.insert_element(tag, attributes.to_vec());
                                token = self.t.next();
                                continue;
                            }
                            _ => {
                                token = self.t.next();
                            }
                        },
                        Some(HtmlToken::EndTag {
                            ref tag
                         }) => {
                            // body終了タグ、html終了タグの場合に次の状態に遷移する
                            match tag.as_str() {
                                "body" => {
                                    self.mode = InsertionMode::AfterBody;
                                    token = self.t.next();
                                    if !self.contain_in_stack(ElementKind::Body) {
                                        // パースの失敗、トークンを無視する
                                        continue;
                                    }
                                    self.pop_until(ElementKind::Body);
                                    continue;
                                }
                                "html" => {
                                    if self.pop_current_node(ElementKind::Body) {
                                        self.mode = InsertionMode::AfterBody;
                                        assert!(self.pop_current_node(ElementKind::Html));
                                    } else {
                                        token = self.t.next();
                                    }
                                    continue;
                                }
                                "p" => {
                                    // 次のトークンが終了タグのとき、スタックからpタグまでを取り出しトークンを次に進める
                                    let element_kind = ElementKind::from_str(tag).expect("failed to convert string to ElementKind");
                                    token = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                "h1" | "h2" => {
                                    // h1、h2終了タグのときスタックからh1、h2タグまでを取り出しトークンを次に進める
                                    let element_kind = ElementKind::from_str(tag).expect("failed to convert string to ElementKind");
                                    token = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                "a" => {
                                    // a終了タグのときスタックからaタグまでを取り出しトークンを次に進める
                                    let element_kind = ElementKind::from_str(tag).expect("failed to convert string to ElementKind");
                                    token = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                _ => {
                                    token = self.t.next();
                                }
                            }
                        }
                        Some(HtmlToken::Char(c)) => {
                            // Body状態のときに文字が出てきたらinsert_charを呼び
                            // テキストノードをDOMツリーに追加する
                            self.insert_char(c);
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                    }
                }
                InsertionMode::Text => {
                    match token {
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        Some(HtmlToken::EndTag { 
                            ref tag
                        }) => {
                            // style終了タグ、script終了タグが出てきたら下の状態に戻る
                            if tag == "style" {
                                self.pop_until(ElementKind::Style);
                                self.mode = self.original_insertion_mode;
                                token = self.t.next();
                                continue;
                            }
                            if tag == "script" {
                                self.pop_until(ElementKind::Script);
                                self.mode = self.original_insertion_mode;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Char(c)) => {
                            // 終了タグが出てくるまで文字をテキストノードとしてDOMツリーに追加します
                            self.insert_char(c);
                            token = self.t.next();
                            continue;
                        }
                        _ => {}
                    }
                    self.mode = self.original_insertion_mode;
                }
                InsertionMode::AfterBody => {
                    // 主にhtml終了タグを扱う
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            // 文字トークンのときは無視して次のトークンも移動する
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::EndTag {
                            ref tag
                         }) => {
                            // EndTagでタグがhtmlのときにAfterAfterBody状態に遷移する
                            if tag == "html" {
                                self.mode = InsertionMode::AfterAfterBody;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                    // それ以外の場合はInBodyに遷移する
                    self.mode = InsertionMode::InBody;
                }
                InsertionMode::AfterAfterBody => {
                    match token {
                        Some(HtmlToken::Char(_c)) => {
                            // 次のトークンが文字のときは無視して次のトークンに移動する
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::Eof) | None => {
                            // 次のトークンがEofまたは存在しないとき
                            // トークン列をすべて消費したため構築したDOMツリーを返す
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    // パースの失敗
                    // 再度トークンを解釈しようと試みる
                    self.mode = InsertionMode::InBody;
                }
            }
        }

        self.window.clone()
    }

    fn create_element(&self, tag: &str, attributes: Vec<Attribute>) -> Node {
        Node::new(NodeKind::Element(Element::new(tag, attributes)))
    }

    // HTMLの構造を解析して要素ノードを正しい位置に挿入する
    // 指定されたタグと属性も持つノードを作成し、挿入先の位置を決定する
    fn insert_element(&mut self, tag: &str, attributes: Vec<Attribute>) {
        let window = self.window.borrow();
        let current = match self.stack_of_open_elements.last() {
            // 現在の開いている要素スタックの最後のノードを取得 (current) する
            Some(n) => n.clone(),
            // スタックがからの場合はルート要素が現在参照しているノードのためwindow.documentを返す
            None => window.document(),
        };

        // 受け取ったタグ、属性をもとにノードを作成
        let node = Rc::new(RefCell::new(self.create_element(tag, attributes)));
        if current.borrow().first_child().is_some() {
            // 現在参照しているノードにすでに子要素がある場合
            let mut last_sibling = current.borrow().first_child();
            loop {
                // 最後の兄弟ノードを探索する
                last_sibling = match last_sibling {
                    Some(ref node) => {
                        if node.borrow().next_sibling().is_some() {
                            node.borrow().next_sibling()
                        } else {
                            break;
                        }
                    }
                    None => unimplemented!("last_sibling should be Some"),
                };
            }

            // 受け取ったタグ、属性をもとにして作成したノードを最後の兄弟ノードの直後に挿入する
            last_sibling.unwrap().borrow_mut().set_next_sibling(Some(node.clone()));
            node.borrow_mut().set_previous_sibling(Rc::downgrade(
                &current.borrow().first_child().expect("failed to get a first child"),
            ))
        } else {
            // 兄弟ノードが存在しない場合
            // 新しいノードを現在参照しているノードの最初の子要素として設定する
            current.borrow_mut().set_first_child(Some(node.clone()));
        }

        // 挿入が完了したら、現在参照しているノードと新しいノードを相互にリンクする
        // 現在参照しているノードの最後の子ノードを新しいノードに設定する
        current.borrow_mut().set_last_child(Rc::downgrade(&node));
        // 新しいノードの親を現在参照しているノードに設定する
        node.borrow_mut().set_parent(Rc::downgrade(&current)); 
        // 新しいノードを開いている要素スタックに追加する
        self.stack_of_open_elements.push(node);
    }

    // stack_of_open_elementsから1つのノードを取り出して
    // 特定の種類と一致する場合にtrueを返す
    fn pop_current_node(&mut self, element_kind: ElementKind) -> bool {
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n,
            None => return false,
        };

        if current.borrow().element_kind() == Some(element_kind) {
            self.stack_of_open_elements.pop();
            return true;
        }

        false
    }

    // stack_of_open_elementsから特定の種類の要素が現れるまでノードを取り出し続ける
    fn pop_until(&mut self, element_kind: ElementKind) {
        assert!(
            self.contain_in_stack(element_kind),
            "stack doesn't have an element {:?}",
            element_kind,
        );

        loop {
            let current = match self.stack_of_open_elements.pop() {
                Some(n) => n,
                None => return,
            };

            if current.borrow().element_kind() == Some(element_kind) {
                return;
            }
        }
    }

    fn contain_in_stack(&mut self, element_kind: ElementKind) -> bool {
        for i in 0..self.stack_of_open_elements.len() {
            if self.stack_of_open_elements[i].borrow().element_kind() == Some(element_kind) {
                return true;
            }
        }

        false
    }

    fn create_char(&self, c: char) -> Node {
        let mut s = String::new();
        s.push(c);
        Node::new(NodeKind::Text(s))
    }

    // 新しい文字ノードを作成してDOMツリーに追加するか、現在のテキストノードに新しい文字を挿入する
    fn insert_char(&mut self, c: char) {
        // 現在開いている要素スタックの最後のノードを取得する
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n.clone(),
            None => return, 
            // スタックが空の場合はルートノードの配下にテキストノードを追加することになる
            // これは適切ではないので何もせずにメソッドを終了する
        };

        // 現在参照しているノードがテキストノードの場合、そのノードに文字を追加する
        if let NodeKind::Text(ref mut s) = current.borrow_mut().kind {
            s.push(c);
            return;
        }

        // 改行文字や空白文字のときはテキストノードを追加しない
        if c == '\n' || c == ' ' {
            return;
        }

        // この前の時点で現在参照しているノードがテキストの場合、改行、空白文字の場合はreturn済
        // 現在参照しているノードが文字ノードでない場合新しいテキストノードを作成する
        let node = Rc::new(RefCell::new(self.create_char(c)));

        // 現在参照しているノードに子要素がある場合
        if current.borrow().first_child().is_some() {
            // 新しいテキストノードを直後に追加
            current
            .borrow()
            .first_child()
            .unwrap()
            .borrow_mut()
            .set_next_sibling(Some(node.clone()));

            node.borrow_mut().set_previous_sibling(Rc::downgrade(
                &current.borrow().first_child().expect("failed to get a first child"),
            ));
        } else {
            // 兄弟ノードが存在しない場合

            // 新しいテキストノードを現在参照しているノードの最初の子要素として設定
            current.borrow_mut().set_first_child(Some(node.clone()));
        }

        // 現在参照しているノードへの新規ノードの追加が完了したら、親子、兄弟関係のリンク
        // 現在参照しているノードの最後の子ノードを新しいノードに設定
        current.borrow_mut().set_last_child(Rc::downgrade(&node));
        // 新しいノードの親を現在参照しているノードに設定
        node.borrow_mut().set_parent(Rc::downgrade(&current));;

        // 新しいノードを開いている要素スタックに追加
        self.stack_of_open_elements.push(node);
    }
}

/// https://html.spec.whatwg.org/multipage/parsing.html#the-insertion-mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertionMode {
    Initial, 
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    Text,
    AfterBody,
    AfterAfterBody
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;

    #[test]
    fn test_empty() {
        let html = "".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let expected = Rc::new(RefCell::new(Node::new(NodeKind::Document)));
        assert_eq!(expected, window.borrow().document());
    }
}
