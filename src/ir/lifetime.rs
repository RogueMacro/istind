use std::{collections::HashMap, ops::Range};

use colored::{Color, Colorize};

use crate::ir::VirtualReg;

#[derive(Debug, Clone)]
pub struct Interval {
    pub range: Range<usize>,
    pub register: Option<u32>,
}

/// The lifetime of a single variable/assignment.
#[derive(Default, Clone)]
pub struct Lifetime {
    intervals: Vec<Interval>,
}

impl Lifetime {
    pub fn intervals(&self) -> &[Interval] {
        &self.intervals
    }

    pub fn start(&self) -> Option<usize> {
        self.intervals.iter().map(|i| i.range.start).min()
    }

    pub fn end(&self) -> Option<usize> {
        self.intervals.iter().map(|i| i.range.end).max()
    }

    pub fn at(&self, position: usize) -> Option<&Interval> {
        self.intervals.iter().find(|i| i.range.contains(&position))
    }

    pub fn at_mut(&mut self, position: usize) -> Option<&mut Interval> {
        self.intervals
            .iter_mut()
            .find(|i| i.range.contains(&position))
    }

    pub fn set_register(&mut self, position: usize, location: Option<u32>) {
        for interval in self.intervals.iter_mut() {
            if interval.range.contains(&position) {
                interval.register = location;
            }
        }
    }

    pub fn next_use_after(&self, op_idx: usize) -> Option<usize> {
        self.intervals
            .iter()
            .map(|i| i.range.start)
            .find(|s| *s > op_idx)
    }

    /// Inserts the interval such that the vec keeps chronological order.
    pub fn insert_interval(&mut self, interval: Interval) {
        let insert_at = self
            .intervals
            .iter()
            .map(|i| i.range.start)
            .position(|s| s > interval.range.start);

        if let Some(insert_at) = insert_at {
            let next = self.intervals[insert_at].range.start;
            assert!(next >= interval.range.end);

            self.intervals.insert(insert_at, interval);
        } else {
            let last = self.intervals.last().unwrap().range.end;
            assert!(last <= interval.range.start);

            self.intervals.push(interval);
        }
    }
}

/// Prints a very simple debug version of a lifetime register with limited information.
pub fn print_lifetimes(lifetimes: &HashMap<VirtualReg, Lifetime>) {
    let end = lifetimes
        .values()
        .map(|l| l.end())
        .max()
        .unwrap_or_default();

    let Some(end) = end else {
        return;
    };

    let nlen = 4;

    let reg_colors = [
        Color::Blue,
        Color::Green,
        Color::Red,
        Color::Yellow,
        Color::Magenta,
        Color::Cyan,
    ];
    print!("{}", "Stack".cyan());
    for (i, c) in reg_colors.iter().enumerate() {
        print!("{}", format!(" X{}", i).color(*c));
    }

    print!("   {}", "No location".bright_black());
    println!();

    print!("{:width$}  ", "", width = nlen);
    for _ in 0..end {
        print!("| ");
    }

    println!();

    let mut lifetimes: Vec<(VirtualReg, Lifetime)> =
        lifetimes.iter().map(|(v, l)| (*v, l.to_owned())).collect();
    lifetimes.sort_by_key(|(_, l)| l.start());

    for (vreg, l) in lifetimes {
        print!("{:width$}: ", format!("{}", vreg), width = nlen);

        let mut prev_end = 0;
        for interval in &l.intervals {
            let fill = interval.range.start - prev_end;
            if fill > 0 {
                if prev_end == 0 {
                    print!("{:width$}", "", width = fill * 2);
                } else {
                    print!("{:width$}", "", width = fill * 2 + 1);
                }
            }

            let color = if let Some(r) = interval.register {
                reg_colors[r as usize % reg_colors.len()]
            } else {
                Color::BrightBlack
            };

            let less = if interval.range.len() == 1 { 1 } else { 2 };
            print!(
                "{}",
                format!(
                    "{:\u{2588}<width$}",
                    "",
                    width = interval.range.len() * 2 - less
                )
                .color(color)
            );

            prev_end = interval.range.end;
        }

        println!();
    }

    println!();
}
