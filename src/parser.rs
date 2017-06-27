use engine::{Expr,Query, BinOp, UnrOp};

#[derive(Debug, PartialEq)]
enum Token {
    Comma,
    Select,
    Id(String),
    LParen,
    RParen,
    By,
    From,
    Where,
    EOF,
    Underscore,
    Int(i64),
    Add,
    Sub,
    Sum,
    Sums,
    Max,
    Min,
    Product,
    Products,
}

#[derive(Debug)]
struct Scanner {
    cmd: Vec<char>,
    pos: usize,
    tok: Token,
}

impl Scanner {
    fn new(s: &str) -> Self {
        Scanner { 
            cmd: s.chars().collect() ,
            pos: 0,    
            tok: Token::Underscore,
        }
    }

    fn next_token(&mut self) -> Result<Token, &'static str> {
        let token = match self.cur_char() {
            Some(' ') => return self.scan_whitespace(),
            Some('a'..'z') => return self.scan_id_or_keyword(),
            Some('(') => Token::LParen,
            Some(')') => Token::RParen,
            Some(',') => Token::Comma,
            Some('0'..'9') => return self.scan_number(),
            Some('+') => Token::Add,
            Some('-') => Token::Sub,
            Some(c) => {
                println!("unexpected char={:?}", c);
                return Err("unexpected token");
            }
            None => Token::EOF,
        };
        self.pos += 1;                
        Ok(token)
    }

    fn scan_number(&mut self) -> Result<Token, &'static str> {
        let mut s = String::new();
        loop {
            match self.cur_char() {
                Some(c@'0'..'9') => s.push(c),
                Some('+') | Some('-') | Some('*') | Some('/') | Some(' ') | None => break,
                Some(_) => return Err("unexpected digit"),                
            }
            self.pos += 1;
        }
        match s.parse::<i64>() {
            Ok(val) => Ok(Token::Int(val)),
            Err(_) => Err("cannot parse int"),
        }
    }

    fn peek_next_token(&mut self) -> Result<Token, &'static str> {
        let pos = self.pos;
        let res = self.next_token();
        self.pos = pos;
        res
    }

    fn scan_whitespace(&mut self) -> Result<Token, &'static str> {
        loop {
            match self.cur_char() {
                Some(' ') => self.pos += 1,
                Some(_) => return self.next_token(),
                None => return Ok(Token::EOF),
            }
        }
    }

    fn scan_id_or_keyword(&mut self) -> Result<Token,  &'static str> {
        let mut id = String::new();
        loop {
            match self.cur_char() {
                Some('(') | Some(')') | Some(' ') | Some(',') | Some('+') | Some('-') | None => break,
                Some(c) => id.push(c) 
            }
            self.pos += 1;
        }
        let tok = match id.as_ref() {
            "select" => Token::Select,
            "from" => Token::From,
            "by" => Token::By,
            "where" => Token::Where,
            "sum" => Token::Sum,
            "sums" => Token::Sums,
            "product" => Token::Product,
            "products" => Token::Products,
            "max" => Token::Max,
            "min" => Token::Min,
            _ => Token::Id(id)
        };
        Ok(tok)
    }

    #[inline]
    fn cur_char(&self) -> Option<char> {
        self.cmd.get(self.pos).map(|c| *c)
    }
}

#[derive(Debug)]
pub struct Parser {
    scanner: Scanner,
}

impl Parser {
    pub fn new(s: &str) -> Self {
        Parser { scanner: Scanner::new(s) }
    }

    pub fn parse(&mut self) -> Result<Query, &'static str> {
        let select = self.parse_select()?;
        let by = self.parse_by()?;
        let from = self.parse_from()?;
        let filters = self.parse_where()?;
        Ok(Query::from(select, by, from, filters))
    }

    fn parse_select(&mut self) -> Result<Vec<Expr>, &'static str> {
        match self.next_token() {
            Ok(Token::Select) => (),
            Ok(_) => return Err("expected select token"),
            Err(err) => return Err(err),
        }
        let mut exprs = Vec::new();
        loop {
            match self.parse_expr() {
                Ok(expr) => exprs.push(expr),
                Err(err) => return Err(err),
            }
            match self.peek_next_token() {
                Ok(Token::Comma) => {
                    let _ = self.scanner.next_token().unwrap();
                }
                Err(err) => return Err(err),
                Ok(_) =>break,
            }
        }
        Ok(exprs)
    }

    fn parse_unr_fn(&mut self, op: UnrOp) -> Result<Expr, &'static str> {
        match self.next_token() {
            Ok(Token::LParen) => (),
            Ok(_) => return Err("expected lparen"),
            Err(err) => return Err(err),
        }
        let expr = self.parse_expr()?;
        match self.next_token() {
            Ok(Token::RParen) => (),
            Ok(_) => return Err("expected rparen"),
            Err(err) => return Err(err),
        }    
        Ok(Expr::UnrFn(op, Box::new(expr)))    
    }

    #[inline]
    fn parse_expr(&mut self) -> Result<Expr, &'static str> {
        let lhs = match self.next_token() {
            Ok(Token::Id(id)) => Expr::Id(id),
            Ok(Token::Int(val)) => Expr::Int(val),
            Ok(Token::Sum) => {
                match self.parse_unr_fn(UnrOp::Sum) {
                    Ok(expr) => expr,
                    Err(err) => return Err(err),
                }
            }
           Ok(Token::Product) => {
                match self.parse_unr_fn(UnrOp::Prod) {
                    Ok(expr) => expr,
                    Err(err) => return Err(err),
                }
            }
           Ok(Token::Min) => {
                match self.parse_unr_fn(UnrOp::Min) {
                    Ok(expr) => expr,
                    Err(err) => return Err(err),
                }
            }    
           Ok(Token::Max) => {
                match self.parse_unr_fn(UnrOp::Max) {
                    Ok(expr) => expr,
                    Err(err) => return Err(err),
                }
            }                        
            Ok(_) => unimplemented!(),
            Err(err) => return Err(err),
        };
        println!("lhs={:?}", lhs);
        let op = match self.peek_next_token() {
            Ok(Token::Add) => BinOp::Add,
            Ok(Token::Sub) => BinOp::Sub,
            Ok(_) => return Ok(lhs),
            Err(err) => return Err(err),
        };
        let _ = self.scanner.next_token().unwrap();
        println!("op={:?}", op);
        let rhs = match self.parse_expr() {
            Ok(expr) => expr,
            Err(err) => return Err(err),
        };
        println!("rhs={:?}", rhs);
        Ok(Expr::BinFn(Box::new(lhs), op, Box::new(rhs)))
    }

    fn parse_from(&mut self) -> Result<Expr, &'static str> {
        match self.next_token() {
            Ok(Token::From) => (),
            Ok(_) => return Err("unexpected tok: expected from"),
            Err(err) => return Err(err),
        }
        match self.parse_expr() {
            Ok(expr) => Ok(expr),
            Err(err) => Err(err),
        }
    }

    fn parse_by(&mut self) -> Result<Option<Vec<Expr>>, &'static str> {
        match self.peek_next_token() {
            Ok(Token::By) => {
                let _ = self.next_token().unwrap();
            },
            Ok(_) => return Ok(None),
            Err(err) => return Err(err),
        }
        let mut exprs = Vec::new();
        loop {
            match self.parse_expr() {
                Ok(expr) => exprs.push(expr),
                Err(err) => return Err(err),
            }
            match self.peek_next_token() {
                Ok(Token::Comma) => {
                    self.next_token().unwrap();
                }
                Ok(Token::EOF) | Ok(Token::From) => break,
                Ok(_) => return Err("unexpected tok in parsing by"),
                Err(err) => return Err(err),
            }
        }
        Ok(Some(exprs))
    }

    #[inline]
    fn peek_next_token(&mut self) -> Result<Token, &'static str> {
        match self.scanner.peek_next_token() {
            Ok(tok) => Ok(tok),
            Err(_) => Err("cannot peek next token"),
        }
    }

    #[inline]
    fn parse_where(&mut self) -> Result<Option<Vec<Expr>>, &'static str> {
        match self.peek_next_token() {
            Ok(Token::Where) => {
                let _ = self.next_token().unwrap();
            }
            Ok(_) => return Ok(None),
            Err(_) => return Err("cannot get next token parsing where"),
        }
        let mut exprs = Vec::new();
        loop {
            match self.parse_expr() {
                Ok(expr) => exprs.push(expr),
                Err(err) => return Err(err),
            }
            match self.peek_next_token() {
                Ok(Token::Comma) => {
                    self.next_token().unwrap();
                }
                Ok(Token::EOF) => break,
                Ok(_) => return Err("unexpected token: expecting EOF"),
                Err(err) => return Err(err),
            }
        }
        Ok(Some(exprs))
    }

    #[inline]
    fn next_token(&mut self) -> Result<Token, &'static str> {
        self.scanner.next_token()
    }
}

#[test]
fn scan_select_token() {
    let mut scanner = Scanner::new("select");
    assert_eq!(scanner.next_token(), Ok(Token::Select));
    scanner = Scanner::new("  select");
    assert_eq!(scanner.next_token(), Ok(Token::Select));    
    scanner = Scanner::new("  select   ");
    assert_eq!(scanner.next_token(), Ok(Token::Select));        
}

#[test]
fn scan_from_token() {
    let mut scanner = Scanner::new("from");
    assert_eq!(scanner.next_token(), Ok(Token::From));
}

#[test]
fn scan_int_token() {
    let mut scanner = Scanner::new("1234");
    assert_eq!(scanner.next_token(), Ok(Token::Int(1234)));
}

#[test]
fn scan_by_token() {
    let mut scanner = Scanner::new("by");
    assert_eq!(scanner.next_token(), Ok(Token::By));
}

#[test]
fn scan_where_token() {
    let mut scanner = Scanner::new("where");
    assert_eq!(scanner.next_token(), Ok(Token::Where));
}

#[test]
fn scan_id_token() {
    let mut scanner = Scanner::new("a");
    assert_eq!(scanner.next_token(), Ok(Token::Id(String::from("a"))));
}

#[test]
fn scan_comma_token() {
    let mut scanner = Scanner::new(",");
    assert_eq!(scanner.next_token(), Ok(Token::Comma));
}

#[test]
fn scan_lparen_token() {
    let mut scanner = Scanner::new("(");
    assert_eq!(scanner.next_token(), Ok(Token::LParen));
}

#[test]
fn scan_rparen_token() {
    let mut scanner = Scanner::new(")");
    assert_eq!(scanner.next_token(), Ok(Token::RParen));
}

#[test]
fn scan_query() {
    let mut scanner = Scanner::new("select a,b from t");
    assert_eq!(scanner.next_token(), Ok(Token::Select));
    assert_eq!(scanner.next_token(), Ok(Token::Id(String::from("a"))));
    assert_eq!(scanner.next_token(), Ok(Token::Comma));
    assert_eq!(scanner.next_token(), Ok(Token::Id(String::from("b"))));
    assert_eq!(scanner.next_token(), Ok(Token::From));
    assert_eq!(scanner.next_token(), Ok(Token::Id(String::from("t"))));
    assert_eq!(scanner.next_token(), Ok(Token::EOF));
}

#[test]
fn parse_select_expr() {
    let mut parser = Parser::new("select a, b");
    let a = Expr::Id(String::from("a"));
    let b = Expr::Id(String::from("b"));
    assert_eq!(parser.parse_select(), Ok(vec![a, b]));
}

#[test]
fn parse_by_expr() {
    let mut parser = Parser::new("by c");
    assert_eq!(parser.parse_by(), Ok(Some(vec![Expr::Id(String::from("c"))])));
}

#[test]
fn parse_by_none() {
    let mut parser = Parser::new("from t");
    assert_eq!(parser.parse_by(), Ok(None));
}

#[test]
fn parse_where_expr() {
    let mut parser = Parser::new("from t");
    assert_eq!(parser.parse_from(), Ok(Expr::Id(String::from("t"))));
}

#[test]
fn parse_query() {
    let mut parser = Parser::new("select b,c by a from t where f1,f2");
    let select = vec![Expr::Id(String::from("b")), Expr::Id(String::from("c"))];
    let by = Some(vec![Expr::Id(String::from("a"))]);
    let from = Expr::Id(String::from("t"));
    let filters = Some(vec![Expr::Id(String::from("f1")), Expr::Id(String::from("f2"))]);
    assert_eq!(parser.parse().unwrap(), Query::from(select, by, from, filters));
}

#[test]
fn parse_add_cols_expr() {
    let mut parser = Parser::new("a+b");
    let lhs = Box::new(Expr::Id(String::from("a")));
    let rhs = Box::new(Expr::Id(String::from("b")));
    let expr = Expr::BinFn(lhs, BinOp::Add, rhs);
    assert_eq!(parser.parse_expr().unwrap(), expr);
}

#[test]
fn parse_add_col_int_expr() {
    let mut parser = Parser::new("a+123");
    let lhs = Box::new(Expr::Id(String::from("a")));
    let rhs = Box::new(Expr::Int(123));
    let expr = Expr::BinFn(lhs, BinOp::Add, rhs);
    assert_eq!(parser.parse_expr().unwrap(), expr);
}

#[test]
fn parse_add_int_col_expr() {
    let mut parser = Parser::new("123+a");
    let rhs = Box::new(Expr::Id(String::from("a")));
    let lhs = Box::new(Expr::Int(123));
    let expr = Expr::BinFn(lhs, BinOp::Add, rhs);
    assert_eq!(parser.parse_expr().unwrap(), expr);
}

#[test]
fn parse_add_int_int_expr() {
    let mut parser = Parser::new("123+456");
    let rhs = Box::new(Expr::Int(456));
    let lhs = Box::new(Expr::Int(123));
    let expr = Expr::BinFn(lhs, BinOp::Add, rhs);
    assert_eq!(parser.parse_expr().unwrap(), expr);
}

#[test]
fn parse_sum_expr() {
    let mut parser = Parser::new("sum(a)");
    let arg = Box::new(Expr::Id(String::from("a")));
    let expr = Expr::UnrFn(UnrOp::Sum, arg);
    assert_eq!(parser.parse_expr().unwrap(), expr);
}