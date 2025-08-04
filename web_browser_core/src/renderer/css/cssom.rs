use alloc::{string::{String, ToString}, vec::Vec};

use crate::renderer::css::token::{self, CssToken, CssTokenizer};
use core::iter::Peekable;

#[derive(Debug, Clone)]
pub struct CssParser {
    t: Peekable<CssTokenizer>,
}

impl CssParser {
    pub fn new(t: CssTokenizer) -> Self {
        Self { t: t.peekable() }
    }

    pub fn parse_stylesheet(&mut self) -> StyleSheet {
        let mut sheet = StyleSheet::new();
        sheet.set_rules(self.consume_list_of_rules());
        sheet
    }

    fn consume_list_of_rules(&mut self) -> Vec<QualifiedRule> {
        let mut rules = Vec::new();
        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => rules,
            };

            match token {
                // AtKeywordトークンが出てきた場合、他のCSSをインポートする
                // @import、メディアクエリを表す@mediaなどのルールが始まることを表す
                CssToken::AtKeyword(_keyword) => {
                    let _rule = self.consume_qualified_rule();
                    // 今回は@から始まるルールはサポートしないので無視
                }
                _ => {
                    // @キーワードトークン以外の場合、1つのルールを解釈しベクタに追加する
                    let rule = self.consume_qualified_rule();
                    match rule {
                        Some(r) => rules.push(r),
                        None => return rules,
                    }
                }
            }
        }
    }

    fn consume_qualified_rule(&mut self) -> Option<QualifiedRule> {
        let mut rule = QualifiedRule::new();
        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return None,
            };

            match token {
                CssToken::OpenCurly => {
                    // 次のトークンが { のとき、宣言ブロックの開始を表す
                    // 宣言ブロックの解釈を行い、ルールのdeclarationsフィールドに設定する
                    assert_eq!(self.t.next(), Some(CssToken::OpenCurly));
                    rule.set_declarations(self.consume_list_of_declarations());
                    return Some(rule);
                }
                _ => {
                    // ルールのセレクタとして扱う
                    // セレクタを解釈し、ルールのselectorフィールドに設定する
                    rule.set_selector(self.consume_selector());
                }
            }
        }
    }

    fn consume_selector(&mut self) -> Selector {
        let token = match self.t.next() {
            Some(t) => t,
            None => panic!("should have a token but got None"),
        };

        match token {
            // ハッシュトークンのときIDセレクタを作成して返す
            CssToken::HashToken(value) => Selector::IdSelector(value[1..].to_string()),
            CssToken::Delim(delim) => {
                // ピリオドのときクラスセレクタを作成して返す
                if delim == '.' {
                    return Selector::ClassSelector(self.consume_ident());
                }
                panic!("Parse error: {:?} is an unexpected token.", token);
            }
            CssToken::Ident(ident) => {
                // 識別子のときタイプセレクタを作成して返す
                // a:hoverのようなセレクタはタイプセレクタとして扱う
                if self.t.peek() == Some(&CssToken::Colon) {
                    // コロンが出てきた場合は宣言ブロックの開始直前までトークンを進める
                    while self.t.peek() != Some(&CssToken::OpenCurly) {
                        self.t.next();
                    }
                }
                Selector::TypeSelector(ident.to_string())
            }
            CssToken::AtKeyword(_keyword) => {
                // @から始まるルールを無視するための宣言ブロックの開始直前までトークンを進める
                while self.t.peek() != Some(&CssToken::OpenCurly) {
                    self.t.next();
                }
                Selector::UnknownSelector
            }
            _ => {
                self.t.next();
                Selector::UnknownSelector
            }
        }
    }

    fn consume_list_of_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();
        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return declarations,
            };

            match token {
                CssToken::CloseCurly => {
                    assert_eq!(self.t.next(), Some(CssToken::CloseCurly));
                    // 閉じ並み括弧が現れたら、今まで作成した宣言を返す
                    return declarations;
                }
                CssToken::SemiColon => {
                    // 次のトークンがセミコロンの場合、1つの宣言が終了したことを表す
                    // 単にセミコロンのトークンを消費し何もしない
                    assert_eq!(self.t.next(), Some(CssToken::SemiColon));
                }
                CssToken::Ident(ref _ident) => {
                    // 次のトークンが識別子のとき、1つの宣言を解釈し追加する
                    if let Some(declaration) = self.consume_declaration() {
                        declarations.push(declaration);
                    }
                }
                _ => {
                    self.t.next();
                }
            }
        }
    }

    fn consume_declaration(&mut self) -> Option<Declaration> {
        if self.t.peek().is_none() {
            return None;
        }

        let mut declaration = Declaration::new();
        // プロパティを処理する
        declaration.set_property(self.consume_ident());
        match self.t.next() {
            Some(token) => match token {
                CssToken::Colon => {}
                // トークンが転んでない場合はパースエラーなのでNoneを返す
                _ => return None, 
            },
            None => return None,
        }

        declaration.set_value(self.consume_component_value());
        Some(declaration)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheet {
    // https://drafts.csswg.org/cssom/#dom-cssstylesheet-cssrules
    pub rules: Vec<QualifiedRule>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn set_rules(&mut self, rules: Vec<QualifiedRule>) {
        self.rules = rules;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedRule {
    // https://www.w3.org/TR/selectors-4/#typedef-selector-list
    pub selector: Selector,
    // https://www.w3.org/TR/css-syntax-3/#parse-a-list-of-declarations
    pub declarations: Vec<Declaration>,
}

impl QualifiedRule {
    pub fn new() -> Self {
        Self {
            selector: Selector: TypeSelection("".to_string()),
            declarations: Vec::new(),
        }
    }

    pub fn set_selector(&mut self, selector: Selector) {
        self.selector = selector;
    }

    pub fn set_declarations(&mut self, declarations: Vec<Declarasion>) {
        self.declarations = declarations;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    TypeSelector(String), // https://www.w3.org/TR/selectors-4/#type-selectors
    ClassSelector(String), // https://www.w3.org/TR/selectors-4/#class-html
    IdSelector(String), // https://www.w3.org/TR/selectors-4/#id-selectors
    UnknownSelector, // パース中にエラーが起こったときに使用されるセレクタ
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: ComponentValue,
}

impl Declaration {
    pub fn new() -> Self {
        Self {
            property: String::new(),
            value: ComponentValue::Ident(String::new()),
        }
    }

    pub fn set_property(&mut self, property: String) {
        self.property = property;
    }

    pub fn set_value(&mut self, value: ComponentValue) {
        self.value = value;
    }
}

pub type ComponentValue = CssToken;
