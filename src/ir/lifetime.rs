use std::{collections::HashMap, ops::Range};

use colored::{Color, Colorize};

use crate::ir::VirtualReg;

#[derive(Debug, Clone, Copy)]
pub enum Location {
    Register(u32),
    Stack(u32),
}

#[derive(Debug, Clone)]
pub struct Interval {
    pub range: Range<usize>,
    pub location: Option<Location>,
}

#[derive(Default, Clone)]
pub struct Lifetime {
    /// During construction is ensured to be in chronological order.
    pub intervals: Vec<Interval>,
}

impl Lifetime {
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

    pub fn set_location(&mut self, position: usize, location: Option<Location>) {
        for interval in self.intervals.iter_mut() {
            if interval.range.contains(&position) {
                interval.location = location;
            }
        }
    }

    pub fn next_use_after(&self, op_idx: usize) -> Option<usize> {
        self.intervals
            .iter()
            .filter_map(|i| {
                if !matches!(i.location, Some(Location::Stack(_))) {
                    Some(i.range.start)
                } else {
                    None
                }
            })
            .find(|s| *s > op_idx)
    }

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
            // println!(
            //     "{} -> prev_end: {}, start: {} (loc: {:?}",
            //     vreg, prev_end, interval.range.start, interval.location
            // );
            let fill = interval.range.start - prev_end;
            if fill > 0 {
                if prev_end == 0 {
                    print!("{:width$}", "", width = fill * 2);
                } else {
                    print!("{:width$}", "", width = fill * 2 + 1);
                }
            } else {
                // print!(" ");
            }

            let color = if let Some(l) = interval.location {
                if let Location::Register(r) = l {
                    reg_colors[r as usize]
                } else {
                    Color::Cyan
                }
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
