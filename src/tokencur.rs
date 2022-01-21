use crate::token::*;

#[derive(Debug)]
pub struct TokenCur {
    pub tokens: Vec<Token>,
    pub index: usize,
    length: usize,
}

#[allow(dead_code)]
impl TokenCur {
    pub fn new(tokens: Vec<Token>) -> Self {
        let length = tokens.len();
        Self {
            tokens,
            index: 0,
            length,
        }
    }

    pub fn can_read(&self) -> bool {
        return self.index < self.length
    }

    pub fn seek(&mut self, value: i32) {
        let mut index2 = self.index as isize + value as isize;
        if index2 < 0 { index2 = 0; }
        if index2 >= self.length as isize { index2 = self.length as isize; }
        self.index = index2 as usize;
    }

    pub fn eq_kind(&self, kind: TokenKind) -> bool {
        self.peek_kind() == kind
    }

    pub fn peek_kind(&self) -> TokenKind {
        if !self.can_read() { return TokenKind::None; }
        let t = &self.tokens[self.index];
        t.kind
    }

    pub fn eq_operator(&self) -> bool {
        let k = self.peek_kind();
        match k {
            TokenKind::Plus | TokenKind::Minus | TokenKind::Mul | TokenKind::Div | TokenKind::Mod => true,
            _ => false,
        }
    }

    pub fn eq_operator_str(&self) -> bool {
        let k = self.peek_kind();
        match k {
            TokenKind::And | TokenKind::Mul => true,
            _ => false,
        }
    }

    pub fn peek(&self) -> Token {
        if !self.can_read() { return Token::new_str(TokenKind::None, "", 0); }
        let t = &self.tokens[self.index];
        t.clone()
    }

    pub fn next(&mut self) -> Token {
        if !self.can_read() { return Token::new_str(TokenKind::None, "", 0); }
        let t = &self.tokens[self.index];
        self.index += 1;
        t.clone()
    }

    pub fn next_kind(&mut self) -> TokenKind {
        if !self.can_read() { return TokenKind::None; }
        let t = &self.tokens[self.index];
        self.index += 1;
        t.kind
    }

    pub fn eq_kinds(&self, kinds: &[TokenKind]) -> bool {
        for (i, k) in kinds.iter().enumerate() {
            let idx = self.index + i;
            if idx >= self.length { return false; }
            let k2 = self.tokens[idx].kind;
            if *k != k2 { return false; }
        }
        true
    }
}

#[cfg(test)]
mod test_tokencur {
    use super::*;
    use crate::tokenizer;
    #[test]
    fn test_tokencur1() {
        let t = tokenizer::tokenize("123 'abc'");
        let cur = TokenCur::new(t);
        assert_eq!(cur.peek_kind(), TokenKind::Int);
        assert_eq!(cur.eq_kinds(&[TokenKind::Int, TokenKind::String]), true);
        //
        let t = tokenizer::tokenize("123回");
        let mut cur = TokenCur::new(t);
        assert_eq!(cur.next_kind(), TokenKind::Int);
        assert_eq!(cur.next_kind(), TokenKind::Repeat);
    }
}