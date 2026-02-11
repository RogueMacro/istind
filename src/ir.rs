use std::{collections::HashMap, ops::Range};

pub mod codegen;

#[derive(Debug)]
pub struct IR {
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub enum Item {
    Function { name: String, bb: BasicBlock },
}

pub type StackOffset = u32;

#[derive(Debug)]
pub struct BasicBlock {
    pub stack_size: u32,
    pub ops: Vec<Operation>,
    pub lifetimes: HashMap<StackOffset, Lifetime>,
}

pub type Op = Operation;

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Store {
        stack_offset: StackOffset,
        value: SourceVal,
    },
    Add {
        a: SourceVal,
        b: SourceVal,
        dest: DestVal,
    },
    Return {
        value: SourceVal,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum SourceVal {
    Immediate(i64),
    Temporary(u32),
    Stack(StackOffset),
}

#[derive(Debug, Clone, Copy)]
pub enum DestVal {
    Temporary(u32),
    Stack(StackOffset),
}

#[derive(Debug)]
pub struct Lifetime {
    intervals: Vec<Range<usize>>,
    start: usize,
    end: usize,
}
