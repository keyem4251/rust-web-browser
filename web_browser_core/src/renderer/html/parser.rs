use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::Window;
use crate::renderer::html::token::HtmlTokenizer;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

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
                    self.mode = INsertionMode::BeforeHead;
                    continue;
                }
            }
        }

        self.window.clone()
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
