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
                attributes: Vec::new(),
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

    // create_tagで作成された最後のトークン（latest_token）に対して1文字をそのトークンのタグの名前として追加する
    fn append_tag_name(&mut self, c: char) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag {
                    ref mut tag,
                    self_closing: _,
                    attributes: _,
                }
                | HtmlToken::EndTag { ref mut tag } => tag.push(c),
                _ => panic!("latest_token should be either StartTag or EndTag"),
            }
        }
    }

    // create_tagで作成された最後のトークン（latest_token）を返し、latest_tokenをNoneにする
    fn take_latest_token(&mut self) -> Option<HtmlToken> {
        assert!(self.latest_token.is_some());

        let t = self.latest_token.as_ref().cloned();
        self.latest_token = None;
        assert!(self.latest_token.is_none());

        t
    }

    // create_tagで作成された最後のトークン（latest_token）に対して新しい属性を追加する
    fn start_new_attribute(&mut self) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag {
                    tag: _,
                    self_closing: _,
                    ref mut attributes,
                } => attributes.push(Attribute::new()),
                _ => panic!("latest_token should be either StartTag"),
            }
        }
    }

    // create_tagで作成された最後のトークン（latest_token）に対して属性の文字を追加する
    fn append_attribute(&mut self, c: char, is_name: bool) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag {
                    tag: _,
                    self_closing: _,
                    ref mut attributes,
                } => {
                    let len = attributes.len();
                    assert!(len > 0);

                    attributes[len - 1].add_char(c, is_name);
                }
                _ => panic!("latest_token should be either StartTag"),
            }
        }
    }

    // create_tagで作成された最後のトークン（latest_token）が開始タグの場合にself_closingフラグをtrueにする
    fn set_self_closing_flag(&mut self) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag {
                    tag: _,
                    ref mut self_closing,
                    attributes: _,
                } => *self_closing = true,
                _ => panic!("latest_token should be StartTag"),
            }
        }
    }

    fn is_eof(&self) -> bool {
        self.pos > self.input.len()
    }
}

impl Iterator for HtmlTokenizer {
    type Item = HtmlToken;

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
                        return Some(HtmlToken::Eof);
                    }

                    // 上記以外の場合は文字トークンを返す
                    return Some(HtmlToken::Char(c));
                }

                // Dataのときに文字が < ならTagOpenに遷移する
                // <body>
                State::TagOpen => {
                    // 文字が / なら状態を次の状態のEndTagOpenに変更する
                    // </body>の/
                    if c == '/' {
                        self.state = State::EndTagOpen;
                        continue;
                    }

                    // 文字がアルファベットなら、現在の文字を再度取り扱う
                    // 状態をTagNameにして、現在の文字をもとにタグを作成する
                    // <body>のbとか
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

                // TagOpenのときに / ならEndTagOpenに遷移する
                // </body>の/
                State::EndTagOpen => {
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = State::TagName;
                        self.create_tag(false);
                        continue;
                    }
                }

                // TagOpenのときに文字がアルファベットならTagNameに遷移する
                // <div class="">のdivのあとの空文字
                State::TagName => {
                    // 文字がホワイトスペースのときBeforeAttributeNameに遷移する
                    if c == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }

                    // <br />など
                    if c == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }

                    // 文字が > のときはタグが閉じられたためDataに遷移してcreate_tagによって作成したlatest_tokenを返す
                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    // 文字がアルファベットの場合は現在のタグに文字をタグの名前として追加する
                    // <div class="">のdivのdとか
                    if c.is_ascii_uppercase() {
                        self.append_tag_name(c.to_ascii_lowercase());
                        continue;
                    }

                    // 入力文字が最後に到達した場合にはEofトークンを返す
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_tag_name(c);
                }

                // タグの属性の名前を処理する前の状態
                // <br class="" />のclassを処理する前の状態
                State::BeforeAttributeName => {
                    // <br class="" />のclassを処理し終わったあとの状態
                    if c == '/' || c == '>' || self.is_eof() {
                        self.reconsume = true;
                        self.state = State::AfterAttributeName;
                        continue;
                    }

                    self.reconsume = true;
                    self.state = State::AttributeName;
                    self.start_new_attribute();
                }

                // タグの属性の名前を処理する状態
                // <br class="" />のclassを処理する状態
                State::AttributeName => {
                    // <br class="" />のclassを処理し終わったあとの状態
                    if c == ' ' || c == '/' || c == '>' || self.is_eof() {
                        self.reconsume = true;
                        self.state = State::AfterAttributeName;
                        continue;
                    }

                    if c == '=' {
                        self.state = State::BeforeAttributeValue;
                        continue;
                    }

                    if c.is_ascii_uppercase() {
                        self.append_attribute(c.to_ascii_lowercase(), /*is_name*/ true);
                        continue;
                    }

                    self.append_attribute(c, /*is_name*/ true);
                }

                // タグの属性の名前を処理している状態
                // <br class="" />のclassを処理している状態
                State::AfterAttributeName => {
                    // 空欄は無視する
                    if c == ' ' {
                        continue;
                    }

                    // <br class="" />の / なのでSelfClosingStartTagに遷移する
                    if c == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }

                    // <br class="" />の = なのでBeforeAttributeValueに遷移する
                    if c == '=' {
                        self.state = State::BeforeAttributeValue;
                        continue;
                    }

                    // <br class="" />の > なのでDataに遷移してcreate_tagによって作成したlatest_tokenを返す
                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.reconsume = true;
                    self.state = State::AttributeName;
                    self.start_new_attribute();
                }

                // タグの属性の値を処理する前の状態
                // <br class="" />のclass="の"を処理する前の状態
                State::BeforeAttributeValue => {
                    // 空欄は無視する
                    if c == ' ' {
                        continue;
                    }

                    if c == '"' {
                        self.state = State::AttributeValueDoubleQuoted;
                        continue;
                    }

                    if c == '\'' {
                        self.state = State::AttributeValueSingleQuoted;
                        continue;
                    }

                    self.reconsume = true;
                    self.state = State::AttributeValueUnquoted;
                }

                // タグの属性の値を処理する状態（ダブルクォートで囲まれた値）
                // <br class="aaa" />の"aaa"を処理する状態
                State::AttributeValueDoubleQuoted => {
                    if c == '"' {
                        self.state = State::AfterAttributeValueQuoted;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_attribute(c, /*is_name*/ false);
                }

                // タグの属性の値を処理する状態（シングルクォートで囲まれた値）
                // <br class='aaa' />の'aaa'を処理する状態
                State::AttributeValueSingleQuoted => {
                    if c == '\'' {
                        self.state = State::AfterAttributeValueQuoted;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_attribute(c, /*is_name*/ false);
                }

                // タグの属性の値を処理する状態（クォートで囲まれていない値）
                // <br class=aaa />のaaaを処理する状態
                State::AttributeValueUnquoted => {
                    if c == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }

                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_attribute(c, /*is_name*/ false);
                }

                // タグの属性の値を処理し終わった状態
                // <br class="aaa" />の"aaa"を処理し終わった状態
                State::AfterAttributeValueQuoted => {
                    if c == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }

                    if c == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }

                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.reconsume = true;
                    self.state = State::BeforeAttributeName;
                }

                // タグが自己終了タグの場合の状態
                // <br />の/の状態
                State::SelfClosingStartTag => {
                    if c == '>' {
                        self.set_self_closing_flag();
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        // エラー処理
                        return Some(HtmlToken::Eof);
                    }
                }

                // <script>タグの中のJavaScriptを処理する状態
                State::ScriptData => {
                    if c == '<' {
                        self.state = State::ScriptDataLessThanSign;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    return Some(HtmlToken::Char(c));
                }

                // <script>タグの中のJavaScriptで < が出現した場合の状態
                // 次の文字がタグの終了を示すか、単なる文字であるかを判定する
                State::ScriptDataLessThanSign => {
                    if c == '/' {
                        self.buf = String::new();
                        self.state = State::ScriptDataEndTagOpen;
                        continue;
                    }

                    self.reconsume = true;
                    self.state = State::ScriptData;
                    return Some(HtmlToken::Char('<'));
                }

                // </script>タグの終了を処理する前の状態
                State::ScriptDataEndTagOpen => {
                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = State::ScriptDataEndTagName;
                        self.create_tag(false);
                        continue;
                    }

                    self.reconsume = true;
                    self.state = State::ScriptData;
                    return Some(HtmlToken::Char('<'));
                }

                // </script>タグのタグ名を処理する状態
                State::ScriptDataEndTagName => {
                    // 空白文字が出現した場合はタグ名の終了としてDataに遷移する
                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    // 次の文字がアルファベットのとき一時的なbufferに文字を追加して、文字をトークンに追加する
                    if c.is_ascii_alphabetic() {
                        self.buf.push(c);
                        self.append_tag_name(c.to_ascii_lowercase());
                        continue;
                    }

                    self.state = State::TemporaryBuffer;
                    self.buf = String::from("</") + &self.buf;
                    self.buf.push(c);
                    continue;
                }

                // 一時的なbufferに保存された文字列を処理する状態
                // HtmlTokenizerのbufにデータを蓄える
                State::TemporaryBuffer => {
                    self.reconsume = true;
                    if self.buf.chars().count() == 0 {
                        self.state = State::ScriptData;
                        return Some(HtmlToken::Char('<'));
                    }

                    // 最初の1文字を削除する
                    let c = self
                        .buf
                        .chars()
                        .nth(0)
                        .expect("self.buf should have at least 1 char");
                    self.buf.remove(0);
                    return Some(HtmlToken::Char(c));
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
    Data,                       // https://html.spec.whatwg.org/multipage/parsing.html#data-state
    TagOpen,             // https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
    EndTagOpen,          // https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
    TagName,             // https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
    BeforeAttributeName, // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
    AttributeName,       // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
    AfterAttributeName, // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
    BeforeAttributeValue, // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
    AttributeValueDoubleQuoted, // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
    AttributeValueSingleQuoted, // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
    AttributeValueUnquoted, // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
    AfterAttributeValueQuoted, // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
    SelfClosingStartTag, // https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
    ScriptData,          // https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
    ScriptDataLessThanSign, // https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-sign-state
    ScriptDataEndTagOpen, // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
    ScriptDataEndTagName, // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state
    TemporaryBuffer,      // https://html.spec.whatwg.org/multipage/parsing.html#temporary-buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_empty() {
        let html = "".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn test_start_and_end_tag() {
        let html = "<body></body>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [
            HtmlToken::StartTag {
                tag: "body".to_string(),
                self_closing: false,
                attributes: Vec::new(),
            },
            HtmlToken::EndTag {
                tag: "body".to_string(),
            },
        ];
        for e in expected {
            assert_eq!(tokenizer.next(), Some(e.clone()));
        }
    }

    #[test]
    fn test_attributes() {
        let html = "<p class=\"A\" id='B' foo=bar></p>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let mut attr1 = Attribute::new();
        attr1.add_char('c', true);
        attr1.add_char('l', true);
        attr1.add_char('a', true);
        attr1.add_char('s', true);
        attr1.add_char('s', true);
        attr1.add_char('A', false);

        let mut attr2 = Attribute::new();
        attr2.add_char('i', true);
        attr2.add_char('d', true);
        attr2.add_char('B', false);

        let mut attr3 = Attribute::new();
        attr3.add_char('f', true);
        attr3.add_char('o', true);
        attr3.add_char('o', true);
        attr3.add_char('b', false);
        attr3.add_char('a', false);
        attr3.add_char('r', false);

        let expected = [
            HtmlToken::StartTag {
                tag: "p".to_string(),
                self_closing: false,
                attributes: vec![attr1, attr2, attr3],
            },
            HtmlToken::EndTag {
                tag: "p".to_string(),
            },
        ];

        for e in expected {
            assert_eq!(tokenizer.next(), Some(e));
        }
    }

    #[test]
    fn test_self_closing_tag() {
        let html = "<img />".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = HtmlToken::StartTag {
            tag: "img".to_string(),
            self_closing: true,
            attributes: Vec::new(),
        };
        assert_eq!(tokenizer.next(), Some(expected));
    }

    #[test]
    fn test_script_tag() {
        let html = "<script>js code;</script>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [
            HtmlToken::StartTag {
                tag: "script".to_string(),
                self_closing: false,
                attributes: Vec::new(),
            },
            HtmlToken::Char('j'),
            HtmlToken::Char('s'),
            HtmlToken::Char(' '),
            HtmlToken::Char('c'),
            HtmlToken::Char('o'),
            HtmlToken::Char('d'),
            HtmlToken::Char('e'),
            HtmlToken::Char(';'),
            HtmlToken::EndTag {
                tag: "script".to_string(),
            },
        ];
        for e in expected {
            assert_eq!(tokenizer.next(), Some(e));
        }
    }
}
