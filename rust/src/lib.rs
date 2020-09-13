use std;
use std::ops::Add;
use std::io::{self, prelude::*};
use std::convert::TryInto;

use std::ops::{AddAssign, SubAssign};

extern crate num;
use num::Zero;

type Position = usize;

#[derive(Debug)]
pub enum InvalidProgramError {
    ExcessiveOpeningBrackets(Position),
    UnexpectedClosingBracket(Position),
}

#[derive(Debug)]
pub enum BFEvalError {
    InvalidProgramError(InvalidProgramError),
    IOError(std::io::Error),
}

impl std::convert::From<std::io::Error> for BFEvalError {
    fn from(err: std::io::Error) -> BFEvalError {
        BFEvalError::IOError(err)
    }
}

impl std::convert::From<InvalidProgramError> for BFEvalError {
    fn from(err: InvalidProgramError) -> BFEvalError {
        BFEvalError::InvalidProgramError(err)
    }
}

pub struct Buffer<T> {
    buf: Vec<T>,
    ptr: usize,
}

impl<T> Buffer<T> 
    where T: Zero + Copy + AddAssign + SubAssign {
    pub fn new(buf_size: usize) -> Self {
        let mut buffer = Self {
            buf: Vec::with_capacity(buf_size),
            ptr: 0 
        };

        for _ in 0..buf_size {
            buffer.buf.push(T::zero())
        };

        buffer
    }

    pub fn clone(buf: &[T]) -> Self {
        let mut buffer = Self {
            buf: Vec::with_capacity(buf.len()),
            ptr: 0 
        };

        for i in 0..buf.len() {
            buffer.buf.push(buf[i]);
        };

        buffer
    }

    pub fn buf(&self) -> &[T] {
        &self.buf[..]
    }

    pub fn fwd(&mut self, offset: usize) {
        self.ptr += offset;
    }

    pub fn bwd(&mut self, offset: usize) {
        self.ptr -= offset;
    }

    pub fn inc(&mut self, offset: T) {
        self.buf[self.ptr] += offset;
    }

    pub fn dec(&mut self, offset: T) {
        self.buf[self.ptr] -= offset;
    }

    pub fn read(&self) -> T {
        self.buf[self.ptr]
    }

    pub fn write(&mut self, val: T) {
        self.buf[self.ptr] = val;
    }
}

pub fn read_mem() -> Result<u32, std::io::Error> {
    let mut input: [u8; 1] = [0];
    io::stdin().read(&mut input)?;
    Ok(input[0].into())
}

pub fn print_mem(mem: u32) -> Result<(), std::io::Error> {
    let x: u8 = mem.try_into().unwrap();
    print!("{}", x as char);
    io::stdout().flush()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    ProgramStart,
    ProgramEnd,
    LoopStart(Position),
    LoopEnd(Position),
    IncValue(Position),
    DecValue(Position),
    MoveForward(Position),
    MoveBack(Position),
    InputValue(Position),
    OutputValue(Position)
}

pub fn tokenize(program: &Vec<char>) -> Vec<Token> {
    let mut tokens = Vec::new();

    tokens.push(Token::ProgramStart);

    for (pos, opcode) in program.iter().enumerate() {
        match opcode {
            '[' => tokens.push(Token::LoopStart(pos)),
            ']' => tokens.push(Token::LoopEnd(pos)),
            '>' => tokens.push(Token::MoveForward(pos)),
            '<' => tokens.push(Token::MoveBack(pos)),
            '+' => tokens.push(Token::IncValue(pos)),
            '-' => tokens.push(Token::DecValue(pos)),
            '.' => tokens.push(Token::OutputValue(pos)),
            ',' => tokens.push(Token::InputValue(pos)),
            _ => (),
        }
    }

    tokens.push(Token::ProgramEnd);

    return tokens;
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    IncValue(u32),
    DecValue(u32),
    MoveForward(usize),
    MoveBack(usize),
    InputValue,
    OutputValue,
    Loop(Vec<Expression>),
}

impl Clone for Expression {
    fn clone(self: &Expression) -> Expression {
        match self {
            &Expression::IncValue(n)    => Expression::IncValue(n),
            &Expression::DecValue(n)    => Expression::DecValue(n),
            &Expression::MoveForward(n) => Expression::MoveForward(n),
            &Expression::MoveBack(n)    => Expression::MoveBack(n),
            &Expression::InputValue     => Expression::InputValue,
            &Expression::OutputValue    => Expression::OutputValue,
             Expression::Loop(sub_exp)  => Expression::Loop(sub_exp.clone()),
        }
    }
}

pub fn parse(tokens: &Vec<Token>) 
    -> Result<Vec<Expression>, InvalidProgramError> {
    let (expressions, _) = do_parse(tokens.iter(), 0)?;
    Ok(expressions)
}

fn do_parse(mut tokens: std::slice::Iter<Token>, level: u32) 
    -> Result<(Vec<Expression>, std::slice::Iter<Token>), InvalidProgramError>
{
    let mut expressions = Vec::new();

    loop {
        if let Some(token) = tokens.next() {
            match token {
                Token::LoopStart(_) => {
                    let (sub_exp, next_tok) = do_parse(tokens, level + 1)?;
                    expressions.push(Expression::Loop(sub_exp));
                    tokens = next_tok;
                }
                Token::LoopEnd(pos) =>
                    if level == 0 {
                        return Err(InvalidProgramError::
                                   UnexpectedClosingBracket(*pos))
                    } else {
                        return Ok((expressions, tokens))
                    },
                Token::MoveForward(_) =>
                    expressions.push(Expression::MoveForward(1)),
                Token::MoveBack(_) =>
                    expressions.push(Expression::MoveBack(1)),
                Token::IncValue(_) =>
                    expressions.push(Expression::IncValue(1)),
                Token::DecValue(_) =>
                    expressions.push(Expression::DecValue(1)),
                Token::OutputValue(_) =>
                    expressions.push(Expression::OutputValue),
                Token::InputValue(_) =>
                    expressions.push(Expression::InputValue),
                Token::ProgramStart => (),
                Token::ProgramEnd =>
                    if level > 0 {
                        return Err(InvalidProgramError::
                                   ExcessiveOpeningBrackets(0))
                    }
            }
        } else {
            break;
        }
    }

    Ok((expressions, tokens))
}

fn replace_top<T>(v: &mut Vec<T>, e: T) {
    v.pop();
    v.push(e);
}

pub fn optimize(expressions: &Vec<Expression>) -> Vec<Expression> {
    let mut optimized: Vec<Expression> = Vec::new();

    for expression in expressions {
        match (optimized.last(), expression) {
            (_, Expression::Loop(sub_exp)) => {
                optimized.push(
                    Expression::Loop(optimize(sub_exp))
                );
            },

            (Some(&Expression::IncValue(n)),    Expression::IncValue(1)) =>
                replace_top(&mut optimized,     Expression::IncValue(n+1)),

            (Some(&Expression::DecValue(n)),    Expression::DecValue(1)) =>
                replace_top(&mut optimized,     Expression::DecValue(n+1)),

            (Some(&Expression::MoveForward(n)), Expression::MoveForward(1)) =>
                replace_top(&mut optimized,     Expression::MoveForward(n+1)),

            (Some(&Expression::MoveBack(n)),    Expression::MoveBack(1)) =>
                replace_top(&mut optimized,     Expression::MoveBack(n+1)),

            (_, e) => 
                optimized.push(e.clone()),
        }
    }

    optimized
}

pub fn run(expressions: &Vec<Expression>) 
    -> Result<Buffer<u32>, BFEvalError> {
    let mut mem = Buffer::<u32>::new(30000);

    do_run(expressions, &mut mem)?;

    Ok(mem)
}

fn do_run(expressions: &Vec<Expression>, mem: &mut Buffer<u32>)
    -> Result<(), BFEvalError> {
    for expression in expressions {
        match expression {
            &Expression::MoveForward(n) => mem.fwd(n),
            &Expression::MoveBack(n)    => mem.bwd(n),
            &Expression::IncValue(n)    => mem.inc(n),
            &Expression::DecValue(n)    => mem.dec(n),
             Expression::OutputValue    => print_mem(mem.read())?,
             Expression::InputValue     => mem.write(read_mem()?),
             Expression::Loop(sub_exp)  => {
                while mem.read() > 0 {
                    do_run(sub_exp, mem)?;
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Default)]
pub struct Stats {
    pub fwd_count: usize,
    pub bwd_count: usize,
    pub inc_count: usize,
    pub dec_count: usize,
    pub output_count: usize,
    pub input_count: usize,
    pub loop_count: usize
}

impl Add for Stats {
    type Output = Stats;

    fn add(self: Stats, other: Stats) -> Stats {
        Stats {
            fwd_count: self.fwd_count + other.fwd_count,
            bwd_count: self.bwd_count + other.bwd_count,
            inc_count: self.inc_count + other.inc_count,
            dec_count: self.dec_count + other.dec_count,
            output_count: self.output_count + other.output_count,
            input_count: self.input_count + other.input_count,
            loop_count: self.loop_count + other.loop_count
        }
    }
}

pub fn stats(expressions: &Vec<Expression>) -> Stats {
    let mut acc = Stats {..Default::default()};

    for expression in expressions {
        match expression {
            Expression::MoveForward(_) => acc.fwd_count += 1,
            Expression::MoveBack(_)    => acc.bwd_count += 1,
            Expression::IncValue(_)    => acc.inc_count += 1,
            Expression::DecValue(_)    => acc.dec_count += 1,
            Expression::OutputValue    => acc.output_count += 1,
            Expression::InputValue     => acc.input_count += 1,
            Expression::Loop(_)        => acc.loop_count += 1,
        }
    }

    expressions
        .iter()
        .fold(
            acc,
            |acc, x| match x {
                Expression::Loop(sub_exp) => acc + stats(sub_exp), 
                _ => acc
            }
        )
}
