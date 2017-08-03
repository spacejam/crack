//! A model checking implementation based on ideas from TLA+.
//!
//! # Examples

use std::collections::HashMap;

#[derive(Debug)]
struct Runs {
    choices: Vec<usize>,
    on_tail: bool,
}

impl Runs {
    fn new<I>(inner: &Vec<Vec<I>>) -> Runs {
        let choices = inner.iter()
            .enumerate()
            .flat_map(|(i, v)| vec![i; v.len()])
            .collect();

        Runs {
            choices: choices,
            on_tail: false,
        }
    }
}

impl Iterator for Runs {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        // from: "The Art of Computer Programming" by Donald Knuth
        if !self.on_tail {
            self.on_tail = true;
            return Some(self.choices.clone());
        }

        let size = self.choices.len();

        if self.on_tail && size < 2 {
            return None;
        }

        let mut j = size - 2;
        while j > 0 && self.choices[j] >= self.choices[j + 1] {
            j -= 1;
        }

        if self.choices[j] < self.choices[j + 1] {
            let mut l = size - 1;
            while self.choices[j] >= self.choices[l] {
                l -= 1;
            }
            self.choices.swap(j, l);
            self.choices[j + 1..size].reverse();
            Some(self.choices.clone())
        } else {
            None
        }
    }
}

pub struct Process<A>(Vec<(Option<String>, Box<Fn(A) -> A>)>);

impl<A> Process<A> {
    fn new<'a>(steps: Vec<(&'a str, Box<Fn(A) -> A>)>) -> Process<A> {
        Process(steps.into_iter().map(|(m, f)| (Some(m.to_owned()), f)).collect())
    }
}

pub struct System<A> {
    pub state: A,
    pub invariants: Vec<Box<Fn(&A) -> bool>>,
    pub processes: Vec<Vec<(Option<String>, Box<Fn(A) -> A>)>>,
}

impl<A: Clone> System<A> {
    fn test(&self) -> Result<(), String> {
        let runs = Runs::new(&self.processes);
        for run in runs {
            let mut indices = HashMap::new();
            let mut state = self.state.clone();
            for choice in run {
                let index = indices.entry(choice).or_insert(0);
                let (ref note, ref f) = self.processes[choice][*index];
                *index += 1;

                state = f(state);

                for invariant in &self.invariants {
                    if !invariant(&state) {
                        return Err("failed invariant".to_owned());
                    }
                }
            }
        }
        Ok(())
    }
}


fn fact(n: usize) -> usize {
    (1..n + 1).fold(1, |p, n| p * n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let a = vec![11, 12];
        let b = vec![21, 22];
        let c = vec![31, 32];

        let runs = Runs::new(&vec![a, b, c]);
        for i in runs {
            println!("i: {:?}", i);
        }

        let p_a = Process::new(vec![
            ("a1", Box::new(|()| println!("a1"))),
            ("a2", Box::new(|()| println!("a2"))),
            ("a3", Box::new(|()| println!("a3"))),
        ]);

        let p_b = Process::new(vec![
            ("b1", Box::new(|()| println!("b1"))),
            ("b2", Box::new(|()| println!("b2"))),
            ("b3", Box::new(|()| println!("b3"))),
        ]);

        let system = System {
            state: (),
            invariants: vec![],
            processes: vec![p_a.0, p_b.0],
        };

        system.test().unwrap();

    }
}
