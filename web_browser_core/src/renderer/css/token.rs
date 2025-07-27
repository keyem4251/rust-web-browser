use alloc::{string::{self, String}, vec::Vec};

#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
    HashToken(String),      // https://www.w3.org/TR/css-syntax-3/#typedef-hash-token
    Delim(char),            // https://www.w3.org/TR/css-syntax-3/#typedef-delim-token
    Number(f64),            // https://www.w3.org/TR/css-syntax-3/#typedef-number-token
    Colon,                  // https://www.w3.org/TR/css-syntax-3/#typedef-colon-token
    SemiColon,              // https://www.w3.org/TR/css-syntax-3/#typedef-semicolon-token
    OpenParenthesis,        // https://www.w3.org/TR/css-syntax-3/#tokendef-open-paren
    CloseParenthesis,       // https://www.w3.org/TR/css-syntax-3/#tokendef-close-paren
    OpenCurly,              // https://www.w3.org/TR/css-syntax-3/#tokendef-open-curly
    CloseCurly,             // https://www.w3.org/TR/css-syntax-3/#tokendef-close-curly
    Ident(String),          // https://www.w3.org/TR/css-syntax-3/#typedef-ident-token
    StringToken(String),    // https://www.w3.org/TR/css-syntax-3/#typedef-string-token
    AtKeyword(String),      // https://www.w3.org/TR/css-syntax-3/#typedef-at-keyword-token
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssTokenizer {
    pos: usize,
    input: Vec<char>,
}

impl CssTokenizer {
    pub fn new(css: String) -> Self {
        Self { pos: 0, input: css.chars().collect() }
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-a-string-token
    // 再びダブルクォーテーション、またはシングルクォーテーションが現れるまで入力を文字として解釈する
    fn consume_string_token(&mut self) -> String {
        let mut s = String::new();
        loop {
            // 現在のトークンの位置が入力の長さを超えたら文字列を返す
            if self.pos >= self.input.len() {
                return s;
            }
            self.pos += 1;
            // 現在のトークンを取得
            let c = self.input[self.pos];
            match c {
                '"' | '\'' => break, // ダブルクォーテーション、シングルクォーテーションが出たので文字列を返す
                _ => s.push(c),
            }
        }
        s
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-number
    // 数字、またはピリオドが出続けている間、数字として解釈する。それ以外の入力がきたら数字を返す。
    fn consume_numeric_token(&mut self) -> f64 {
        let mut num = 0f64;
        let mut floating = false;
        let mut floating_digit = 1f64;

        loop {
            // 現在のトークンの位置が入力の長さを超えたら数値を返す
            if self.pos >= self.input.len() {
                return num;
            }
            // 現在のトークンを取得
            let c = self.input[self.pos];
            match c {
                '0'..='9' => {
                    if floating {
                        floating_digit *= 1f64 / 10f64;
                        num += (c.to_digit(10).unwrap() as f64) * floating_digit
                    } else {
                        num = num * 10.0 + (c.to_digit(10).unwrap() as f64);
                    }
                    self.pos += 1;
                }
                '.' => {
                    floating = true;
                    self.pos += 1;
                }
                _ => break,
            }
        }
        num
    }
}

impl Iterator for CssTokenizer {
    type Item = CssToken;

    // https://www.w3.org/TR/css-syntax-3/#consume-token
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos >= self.input.len() {
                return None;
            }

            let c = self.input[self.pos];
            let token = match c {
                // 次のトークンを決定する
                '(' => CssToken::OpenParenthesis,
                ')' => CssToken::CloseParenthesis,
                ',' => CssToken::Delim(','),
                '.' => CssToken::Delim('.'),
                ':' => CssToken::Colon,
                ';' => CssToken::SemiColon,
                '{' => CssToken::OpenCurly,
                '}' => CssToken::CloseCurly,
                ' ' | '\n' => {
                    self.pos += 1;
                    continue;
                }
                '"' | '\'' => {
                    let value = self.consume_string_token();
                    CssToken::StringToken(value)
                }
                '0'..='9' => {
                    let t = CssToken::Number(self.consume_numeric_token());
                    self.pos += 1;
                    t
                }
                _ => {
                    unimplemented!("char {} is not supported yet", c);
                }
            };

            self.pos += 1;
            return Some(token);
        }
    }
}
