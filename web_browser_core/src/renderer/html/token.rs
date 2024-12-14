use crate::renderer::html::attribute::Attribute;
use alloc::{string::String, vec::Vec};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTokenizer {
    state: State,
    pos: usize,
    reconsume: bool,
    latest_token: Option<HtmlToken>,
    input: Vec<char>,
    buf: String,
}

impl HtmlTokenizer {
    pub fn new(html: String) -> Self {
        Self {
            state: State::Data,
            pos: 0,
            reconsume: false,
            latest_token: None,
            input: html.chars().collect(),
            buf: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlToken {
    // 開始タグ
    StartTag {
        tag: String,
        self_closing: bool,
        attributes: Vec<Attribute>,
    },
    // 終了タグ
    EndTag {
        tag: String,
    },
    // 文字
    Char(char),
    // ファイルの終了（End Of File）
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Data, // https://html.spec.whatwg.org/multipage/parsing.html#data-state
    TagOpen,     // https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
    EndTagOpen,     // https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
    TagName,     // https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
    BeforeAttributeName,// https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
    AttributeName, // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
    AfterAttributeName, // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
    BeforeAttributeValue, // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
    AttributeValueDoubleQuoted, // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
    AttributeValueSingleQuoted, // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
    AttributeValueUnquoted, // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
    AfterAttributeValueQuoted, // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
    SelfClosingStartTag, // https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
    ScriptData, // https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
    ScriptDataLessThanSign, // https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-sign-state
    ScriptDataEndTagOpen, // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
    ScriptDataEndTagName, // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state
    TemporaryBuffer, // https://html.spec.whatwg.org/multipage/parsing.html#temporary-buffer
}
