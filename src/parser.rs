use grammar::*;
use grammar_lexer::*;
use peruse::parsers::*;

use peruse::slice_parsers::*;

pub fn program() -> Box<Parser<I=[Token], O=Block>> {

    fn expression() -> Box<Parser<I=[Token], O=Expr>> {
        let simple_term = matcher(|token| match token {
            Token::Ident(name) => Some(Expr::Variable(name)),
            Token::Number(i)   => Some(Expr::Num(i)),
            _ => None
        });

        let paren_expr = || lit(Token::OpenParen).then_r(recursive(|| expression())).then_l(lit(Token::CloseParen));

        let factor = paren_expr().or(simple_term);

        let multop = matcher(|token| match token {
            Token::MultSign     => Some(MultOp::Multiply),
            Token::DivideSign   => Some(MultOp::Divide),
            Token::ModuloSign   => Some(MultOp::Modulo),
            _ => None
        });

        let addop = matcher(|token| match token {
            Token::PlusSign     => Some(AddOp::Add),
            Token::MinusSign    => Some(AddOp::Subtract),
            _ => None
        });

        let mult =  {
            let p = factor.clone().then(multop.then(factor).repeat()).map(|(first, seq)| {
                let mut ops = Vec::new();
                ops.push(MultTerm(MultOp::Start, first));
                for &(ref op, ref value) in seq.iter() {
                    ops.push(MultTerm(op.clone(), value.clone())); //maybe box the value instead
                }
                if ops.len() == 1 {
                    ops[0].1.clone()
                } else{
                    Expr::MultDiv(ops)
                }
            });
            boxed(p)
        };

        let plus = {
            let p = mult.clone().then(addop.then(mult).repeat()).map(|(first, seq)| {
                let mut ops = Vec::new();
                    ops.push(AddTerm(AddOp::Start, first));
                    for &(ref op, ref value) in seq.iter() {
                        ops.push(AddTerm(op.clone(), value.clone()));
                    }
                    if ops.len() == 1 {
                        ops[0].1.clone()
                    } else{
                        Expr::AddSub(ops)
                    }
            });
            boxed(p)
        };

        Box::new(plus)
    }

    let assignment = {
        let target = matcher(|token| match token {
            Token::Ident(name) => Some(name),
            _ => None
        });
        boxed(target
            .then_l(lit(Token::Equals))
            .then(recursive(|| expression()))
            .map(|(target, expr)| Statement::Assign(target, expr)))
        };


        let statements = assignment.then_l(lit(Token::NewLine)).repeat();

        Box::new(statements.map(|v| Block(v)))
    }
