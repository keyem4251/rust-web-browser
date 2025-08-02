use alloc::{string::String, vec::Vec};

use crate::renderer::css::token::{CssToken, CssTokenizer};
use core::iter::Peekable;

#[derive(Debug, Clone)]
pub struct CssParser {
    t: Peekable<CssTokenizer>,
}

impl CssParser {
    pub fn new(t: CssTokenizer) -> Self {
        Self { t: t.peekable() }
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
