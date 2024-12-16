use crate::renderer::html::attribute::Attribute;
use alloc::{string::String, vec::Vec};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTokenizer {
    state: State,
    pos: usize,
    reconsume: bool, // 状態だけ更新して、使用した文字をもう一度再使用する
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

    // inputの文字列から現在の位置（pos）の文字を返し、posを1つ進める
    fn consume_next_input(&mut self) -> char {
        let c = self.input[self.pos];
        self.pos += 1;
        c
    }

    // StartTagまたはEndTagトークンを作成しlatest_tokenフィールドにセットする
    fn create_tag(&mut self, start_tag_token: bool) {
        if start_tag_token {
            self.latest_token = Some(HtmlToken::StartTag {
                tag: String::new(), 
                self_closing: false, 
                attributes: Vec::new() 
            });
        } else {
            self.latest_token = Some(HtmlToken::EndTag { tag: String::new() });
        }
    }

    // 使用した文字を再利用する場合には現在の位置（進めたpos）から1つ戻った位置の文字を返す
    fn reconsume_input(&mut self) -> char {
        self.reconsume = false;
        self.input[self.pos - 1]
    }
}

impl Iterator for HtmlTokenizer {
    type Item = HtmlTokenizer;

    fn next(&mut self) -> Option<Self::Item> {
        // 現在の位置が入力文字よりの長さより長い場合はNoneを返す
        if self.pos >= self.input.len() {
            return None;
        }

        loop {
            let c = match self.reconsume {
                true => self.reconsume_input(),
                false => self.consume_next_input(),                
            };

            match self.state {
                State::Data => {
                    // 文字が < なら状態を次の状態のTagOpenに変更する
                    if c == '<' {
                        self.state = State::TagOpen;
                        continue;
                    }

                    // 入力文字が最後に到達した場合にはEofトークンを返す
                    if self.is_eof() {
                        return Some(HtmlToken::Eof)
                    }

                    // 上記以外の場合は文字トークンを返す
                    return Some(HtmlToken::Char(c));
                }
                State::TagOpen => {
                    // 文字が / なら状態を次の状態のEndTagOpenに変更する
                    if c == '/' {
                        self.state = State::EndTagOpen;
                        continue;
                    }

                    // 文字がアルファベットなら、現在の文字を再度取り扱う
                    // 状態をTagNameにして、現在の文字をもとにタグを作成する
                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = State::TagName;
                        self.create_tag(true);
                        continue;
                    }

                    // 入力文字が最後に到達した場合にはEofトークンを返す
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    // 上記以外の場合は現在の文字をサイト取り扱う
                    self.reconsume = true;
                    self.state = State::Data;
                }
            }
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
