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

    // create_tagで作成された最後のトークン（latest_token）に対して1文字をそのトークンのタグの名前として追加する
    fn append_tag_name(&mut self, c: char) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag { 
                    ref mut tag,
                    self_closing: _,
                    attributes: _,
                } | HtmlToken::EndTag { ref mut tag } => tag.push(c),
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
                        return Some(HtmlToken::Eof)
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
                        self.append_attribute(c.to_ascii_lowercase(), /*is_name*/true);
                        continue;
                    }

                    self.append_attribute(c, /*is_name*/true);
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

                    self.append_attribute(c, /*is_name*/false);
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

                    self.append_attribute(c, /*is_name*/false);
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

                    self.append_attribute(c, /*is_name*/false);
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
