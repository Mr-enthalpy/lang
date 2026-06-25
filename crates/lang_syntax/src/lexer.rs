use crate::{
    normalize_source_text, Diagnostic, DiagnosticCode, OperatorSpelling, Span, Symbol, Token,
    TokenKind, TriviaKind,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(source: &str) -> LexOutput {
    let normalized = normalize_source_text(source);
    Lexer::new(&normalized).lex()
}

struct Lexer<'src> {
    source: &'src str,
    byte: usize,
    line: usize,
    column: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl<'src> Lexer<'src> {
    fn new(source: &'src str) -> Self {
        Self {
            source,
            byte: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn lex(mut self) -> LexOutput {
        while !self.is_eof() {
            self.lex_token();
        }

        let eof_span = Span::at(self.byte, self.line, self.column);
        self.tokens.push(Token::new(TokenKind::Eof, eof_span, ""));

        LexOutput {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn lex_token(&mut self) {
        if self.peek_is_ascii_ident_start() {
            self.lex_name();
        } else if self.peek_is_ascii_digit() {
            self.lex_numeric_literal();
        } else if self.has_leading_dot_float() {
            self.lex_numeric_literal();
        } else if self.has_ranked_string_start() {
            self.lex_string_literal();
        } else if self.peek_is_whitespace() {
            self.lex_whitespace();
        } else if self.starts_with("//") {
            self.lex_line_comment();
        } else if self.starts_with("/*") {
            self.lex_block_comment();
        } else if self.lex_operator_or_symbol() {
            // Operator or symbol consumed.
        } else {
            self.lex_invalid_token();
        }
    }

    fn lex_name(&mut self) {
        let start = self.mark();
        self.advance_char();

        while self.peek_is_ascii_ident_continue() {
            self.advance_char();
        }

        self.push_token(TokenKind::Name, start);
    }

    fn lex_numeric_literal(&mut self) {
        let start = self.mark();

        if self.starts_with(".") {
            self.advance_char();
            self.lex_radix_digits(|c| c.is_ascii_digit());
            if self.peek_char().map_or(false, |c| c == 'e' || c == 'E') {
                self.lex_decimal_exponent();
            }
            self.push_token(TokenKind::FloatLiteral, start);
            return;
        }

        if self.starts_with("0b") || self.starts_with("0B") {
            self.advance_char();
            self.advance_char();
            let has_digits = self.lex_radix_digits(|c| matches!(c, '0'..='1'));
            if !has_digits {
                self.emit_invalid_numeric("expected binary digits after `0b`");
            }
            self.push_token(TokenKind::IntLiteral, start);
            return;
        }

        if self.starts_with("0o") || self.starts_with("0O") {
            self.advance_char();
            self.advance_char();
            let has_digits = self.lex_radix_digits(|c| matches!(c, '0'..='7'));
            if !has_digits {
                self.emit_invalid_numeric("expected octal digits after `0o`");
            }
            self.push_token(TokenKind::IntLiteral, start);
            return;
        }

        if self.starts_with("0x") || self.starts_with("0X") {
            self.advance_char();
            self.advance_char();

            if self.starts_with(".")
                && self.source[self.byte + 1..]
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_ascii_hexdigit())
            {
                self.advance_char();
                self.lex_radix_digits(|c| c.is_ascii_hexdigit());
                if self.starts_with("p") || self.starts_with("P") {
                    self.advance_char();
                    if self.starts_with("+") || self.starts_with("-") {
                        self.advance_char();
                    }
                    let has_exp = self.lex_radix_digits(|c| c.is_ascii_digit());
                    if !has_exp {
                        self.emit_invalid_numeric("expected decimal digits in hex float exponent");
                    }
                    self.push_token(TokenKind::FloatLiteral, start);
                    return;
                }
                self.emit_invalid_numeric("expected hex float exponent after hex fraction");
                self.push_token(TokenKind::FloatLiteral, start);
                return;
            }

            let has_digits = self.lex_radix_digits(|c| c.is_ascii_hexdigit());
            if !has_digits {
                self.emit_invalid_numeric("expected hexadecimal digits after `0x`");
                self.push_token(TokenKind::IntLiteral, start);
                return;
            }

            if self.starts_with(".") {
                let after_dot = self.source[self.byte + 1..].chars().next();
                if after_dot.map_or(false, |c| c.is_ascii_hexdigit()) {
                    self.advance_char();
                    self.lex_radix_digits(|c| c.is_ascii_hexdigit());
                } else if after_dot.map_or(false, |c| c == 'p' || c == 'P') {
                    self.advance_char();
                }
            }

            if self.starts_with("p") || self.starts_with("P") {
                self.advance_char();
                if self.starts_with("+") || self.starts_with("-") {
                    self.advance_char();
                }
                let has_exp = self.lex_radix_digits(|c| c.is_ascii_digit());
                if !has_exp {
                    self.emit_invalid_numeric("expected decimal digits in hex float exponent");
                }
                self.push_token(TokenKind::FloatLiteral, start);
                return;
            }

            self.push_token(TokenKind::IntLiteral, start);
            return;
        }

        self.lex_radix_digits(|c| c.is_ascii_digit());

        if self.starts_with(".") {
            let after_dot = self.source[self.byte + 1..].chars().next();
            if after_dot.map_or(false, |c| c.is_ascii_digit()) {
                self.advance_char();
                self.lex_radix_digits(|c| c.is_ascii_digit());
                if self.peek_char().map_or(false, |c| c == 'e' || c == 'E') {
                    self.lex_decimal_exponent();
                }
                self.push_token(TokenKind::FloatLiteral, start);
                return;
            } else if !after_dot.map_or(false, |c| c == '.') {
                if self.is_exponent_after_dot() {
                    self.advance_char();
                    self.lex_decimal_exponent();
                    self.push_token(TokenKind::FloatLiteral, start);
                    return;
                }
                let is_ident =
                    after_dot.map_or(false, |c| matches!(c, 'a'..='z' | 'A'..='Z' | '_'));
                if after_dot == Some('\'')
                    && self.source[self.byte + 2..]
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_ascii_digit())
                {
                    self.advance_char();
                    self.emit_invalid_numeric("invalid digit separator position");
                    self.advance_char();
                    self.lex_radix_digits(|c| c.is_ascii_digit());
                    if self.peek_char().map_or(false, |c| c == 'e' || c == 'E') {
                        self.lex_decimal_exponent();
                    }
                    self.push_token(TokenKind::FloatLiteral, start);
                    return;
                }
                if !is_ident {
                    self.advance_char();
                    if self.peek_char().map_or(false, |c| c == 'e' || c == 'E') {
                        self.lex_decimal_exponent();
                    }
                    self.push_token(TokenKind::FloatLiteral, start);
                    return;
                }
            }
        }

        if self.peek_char().map_or(false, |c| c == 'e' || c == 'E') {
            self.lex_decimal_exponent();
            self.push_token(TokenKind::FloatLiteral, start);
            return;
        }

        self.push_token(TokenKind::IntLiteral, start);
    }

    fn lex_string_literal(&mut self) {
        let start = self.mark();

        let mut k: usize = 0;
        while self.peek_char() == Some('\\') {
            self.advance_char();
            k += 1;
        }
        self.advance_char();

        let mut consecutive_bs: usize = 0;

        loop {
            if self.is_eof() || matches!(self.peek_char(), Some('\n' | '\r')) {
                let span = self.span_from(start);
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::UnclosedString,
                    "unclosed string literal",
                    span,
                ));
                self.push_token_with_span(TokenKind::StringLiteral, span);
                return;
            }

            match self.peek_char() {
                Some('\\') => {
                    self.advance_char();
                    consecutive_bs += 1;
                }
                Some('"') => {
                    if consecutive_bs >= k {
                        self.advance_char();
                        self.push_token(TokenKind::StringLiteral, start);
                        return;
                    }
                    self.advance_char();
                    consecutive_bs = 0;
                }
                Some(_) => {
                    self.advance_char();
                    consecutive_bs = 0;
                }
                None => unreachable!(),
            }
        }
    }

    fn has_leading_dot_float(&self) -> bool {
        self.starts_with(".")
            && self.source[self.byte + 1..]
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_digit())
    }

    fn is_exponent_after_dot(&self) -> bool {
        let rest = &self.source[self.byte + 1..];
        let first = match rest.chars().next() {
            Some(c) if c == 'e' || c == 'E' => c,
            _ => return false,
        };
        let after_e = &rest[first.len_utf8()..];
        let mut chars = after_e.chars();
        let mut next = chars.next();
        if next.map_or(false, |c| c == '+' || c == '-') {
            next = chars.next();
        }
        next.map_or(false, |c| c.is_ascii_digit())
    }

    fn has_ranked_string_start(&self) -> bool {
        let rest = &self.source[self.byte..];
        let bs_count = rest.chars().take_while(|&c| c == '\\').count();
        rest.chars().nth(bs_count) == Some('"')
    }

    fn lex_radix_digits(&mut self, is_digit: fn(char) -> bool) -> bool {
        let mut any = false;
        let mut prev_was_digit = false;

        loop {
            let ch = self.peek_char();
            match ch {
                Some(c) if is_digit(c) => {
                    self.advance_char();
                    any = true;
                    prev_was_digit = true;
                }
                Some('\'') => {
                    let next = self.source[self.byte + 1..].chars().next();
                    let next_is_digit = next.map_or(false, is_digit);
                    if !prev_was_digit || !next_is_digit {
                        self.emit_invalid_numeric("invalid digit separator position");
                        self.advance_char();
                        prev_was_digit = false;
                        continue;
                    }
                    self.advance_char();
                    prev_was_digit = false;
                }
                _ => break,
            }
        }

        any
    }

    fn lex_decimal_exponent(&mut self) {
        self.advance_char();
        if self.peek_char().map_or(false, |c| c == '+' || c == '-') {
            self.advance_char();
        }
        let has_digits = self.lex_radix_digits(|c| c.is_ascii_digit());
        if !has_digits {
            self.emit_invalid_numeric("expected decimal digits in exponent");
        }
    }

    fn emit_invalid_numeric(&mut self, message: &str) {
        let span = Span::at(self.byte, self.line, self.column);
        self.diagnostics.push(Diagnostic::new(
            DiagnosticCode::InvalidNumericLiteral,
            message,
            span,
        ));
    }

    fn lex_whitespace(&mut self) {
        let start = self.mark();

        while self.peek_is_whitespace() {
            self.advance_char();
        }

        self.push_token(TokenKind::Trivia(TriviaKind::Whitespace), start);
    }

    fn lex_line_comment(&mut self) {
        let start = self.mark();
        self.advance_char();
        self.advance_char();

        while let Some(ch) = self.peek_char() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.advance_char();
        }

        self.push_token(TokenKind::Trivia(TriviaKind::LineComment), start);
    }

    fn lex_block_comment(&mut self) {
        let start = self.mark();
        self.advance_char();
        self.advance_char();

        let mut depth: usize = 1;

        while !self.is_eof() {
            if self.starts_with("/*") {
                self.advance_char();
                self.advance_char();
                depth += 1;
                continue;
            }

            if self.starts_with("*/") {
                self.advance_char();
                self.advance_char();
                depth -= 1;
                if depth == 0 {
                    break;
                }
                continue;
            }

            self.advance_char();
        }

        let span = self.span_from(start);
        if depth != 0 {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::UnclosedComment,
                "unclosed block comment",
                span,
            ));
        }

        self.push_token_with_span(TokenKind::Trivia(TriviaKind::BlockComment), span);
    }

    fn lex_operator_or_symbol(&mut self) -> bool {
        // Longest-match priority order across operators and structural symbols:
        //   1. 3-char operators:   <<=  >>=
        //   2. 2-char structural:  =>   ->   |>   ..   ::
        //   3. 2-char operators:   ++   --
        //   4. 2-char operators:   +=   -=   *=   /=   &=   |=   &&   ||
        //   5. 2-char operators:   <=   >=   ==   !=
        //   6. 2-char operators:   <<   >>
        //   7. 1-char operators:   +  -  *  /  !  &  |  @  ~  ^  $  ?
        //   8. 1-char structural:  <  >  =  .  :  ,  ;  (  )  [  ]  {  }

        // Step 0: 3-char structural symbol
        if self.starts_with("===") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Symbol(Symbol::TripleEqual), start);
            return true;
        }

        // Step 1: 3-char operators
        if self.starts_with("<<=") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::LessLessEqual), start);
            return true;
        }
        if self.starts_with(">>=") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.advance_char();
            self.push_token(
                TokenKind::Operator(OperatorSpelling::GreaterGreaterEqual),
                start,
            );
            return true;
        }

        // Step 2: 2-char structural symbols
        if self.starts_with("=>") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Symbol(Symbol::FatArrow), start);
            return true;
        }
        if self.starts_with("->") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Symbol(Symbol::ThinArrow), start);
            return true;
        }
        if self.starts_with("|>") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Symbol(Symbol::PipeGreater), start);
            return true;
        }
        if self.starts_with("..") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Symbol(Symbol::DotDot), start);
            return true;
        }
        if self.starts_with("::") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Symbol(Symbol::ColonColon), start);
            return true;
        }

        // Step 3: ++  --
        if self.starts_with("++") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::PlusPlus), start);
            return true;
        }
        if self.starts_with("--") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::MinusMinus), start);
            return true;
        }

        // Step 4: +=  -=  *=  /=  &=  |=  &&  ||
        if self.starts_with("+=") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::PlusEqual), start);
            return true;
        }
        if self.starts_with("-=") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::MinusEqual), start);
            return true;
        }
        if self.starts_with("*=") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::StarEqual), start);
            return true;
        }
        if self.starts_with("/=") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::SlashEqual), start);
            return true;
        }
        if self.starts_with("&=") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::AmpEqual), start);
            return true;
        }
        if self.starts_with("|=") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::PipeEqual), start);
            return true;
        }
        if self.starts_with("&&") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::AmpAmp), start);
            return true;
        }
        if self.starts_with("||") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::PipePipe), start);
            return true;
        }

        // Step 5: <=  >=  ==  !=
        if self.starts_with("<=") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::LessEqual), start);
            return true;
        }
        if self.starts_with(">=") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::GreaterEqual), start);
            return true;
        }
        if self.starts_with("==") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::EqualEqual), start);
            return true;
        }
        if self.starts_with("!=") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::BangEqual), start);
            return true;
        }

        // Step 6: <<  >>
        if self.starts_with("<<") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::LessLess), start);
            return true;
        }
        if self.starts_with(">>") {
            let start = self.mark();
            self.advance_char();
            self.advance_char();
            self.push_token(TokenKind::Operator(OperatorSpelling::GreaterGreater), start);
            return true;
        }

        // Step 7: 1-char operators
        let single_ops: &[(char, OperatorSpelling)] = &[
            ('+', OperatorSpelling::Plus),
            ('-', OperatorSpelling::Minus),
            ('*', OperatorSpelling::Star),
            ('/', OperatorSpelling::Slash),
            ('!', OperatorSpelling::Bang),
            ('&', OperatorSpelling::Amp),
            ('|', OperatorSpelling::Pipe),
            ('@', OperatorSpelling::At),
            ('~', OperatorSpelling::Tilde),
            ('^', OperatorSpelling::Caret),
            ('$', OperatorSpelling::Dollar),
            ('?', OperatorSpelling::Question),
        ];
        for &(ch, spelling) in single_ops {
            if self.peek_char() == Some(ch) {
                let start = self.mark();
                self.advance_char();
                self.push_token(TokenKind::Operator(spelling), start);
                return true;
            }
        }

        // Step 8: 1-char structural symbols
        let single_symbols: &[(char, Symbol)] = &[
            ('(', Symbol::LParen),
            (')', Symbol::RParen),
            ('[', Symbol::LBracket),
            (']', Symbol::RBracket),
            ('{', Symbol::LBrace),
            ('}', Symbol::RBrace),
            (',', Symbol::Comma),
            (':', Symbol::Colon),
            ('=', Symbol::Equal),
            ('.', Symbol::Dot),
            ('<', Symbol::Less),
            ('>', Symbol::Greater),
            (';', Symbol::Semicolon),
        ];
        for &(ch, symbol) in single_symbols {
            if self.peek_char() == Some(ch) {
                let start = self.mark();
                self.advance_char();
                self.push_token(TokenKind::Symbol(symbol), start);
                return true;
            }
        }

        false
    }

    fn lex_invalid_token(&mut self) {
        let start = self.mark();
        self.advance_char();
        let span = self.span_from(start);

        self.diagnostics.push(Diagnostic::new(
            DiagnosticCode::InvalidToken,
            "invalid token",
            span,
        ));
        self.push_token_with_span(TokenKind::Invalid, span);
    }

    fn push_token(&mut self, kind: TokenKind, start: Mark) {
        let span = self.span_from(start);
        self.push_token_with_span(kind, span);
    }

    fn push_token_with_span(&mut self, kind: TokenKind, span: Span) {
        let text = self.source[span.byte_start..span.byte_end].to_string();
        self.tokens.push(Token::new(kind, span, text));
    }

    fn span_from(&self, start: Mark) -> Span {
        Span::new(start.byte, self.byte, start.line, start.column)
    }

    fn mark(&self) -> Mark {
        Mark {
            byte: self.byte,
            line: self.line,
            column: self.column,
        }
    }

    fn is_eof(&self) -> bool {
        self.byte >= self.source.len()
    }

    fn starts_with(&self, text: &str) -> bool {
        self.source[self.byte..].starts_with(text)
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.byte..].chars().next()
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.byte += ch.len_utf8();

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += ch.len_utf8();
        }

        Some(ch)
    }

    fn peek_is_ascii_ident_start(&self) -> bool {
        matches!(self.peek_char(), Some('a'..='z' | 'A'..='Z' | '_'))
    }

    fn peek_is_ascii_ident_continue(&self) -> bool {
        matches!(
            self.peek_char(),
            Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_')
        )
    }

    fn peek_is_ascii_digit(&self) -> bool {
        matches!(self.peek_char(), Some('0'..='9'))
    }

    fn peek_is_whitespace(&self) -> bool {
        matches!(self.peek_char(), Some(' ' | '\t' | '\r' | '\n'))
    }
}

#[derive(Clone, Copy)]
struct Mark {
    byte: usize,
    line: usize,
    column: usize,
}
