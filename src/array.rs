use crate::gui;
use iced::canvas;
use std::cmp;

#[derive(Clone)]
pub enum Step {
    ComparisonTwo(usize, usize),
    Comparison(usize),
    AccessTwo(usize, usize),
    Access(usize),
    None,
}

impl Step {
    pub fn contains(&self, index: usize) -> bool {
        match self {
            Step::ComparisonTwo(x, y) | Step::AccessTwo(x, y) => *x == index || *y == index,
            Step::Comparison(x) | Step::Access(x) => *x == index,
            Step::None => false,
        }
    }

    pub fn is_comparison(&self) -> bool {
        match self {
            Step::Comparison(_) | Step::ComparisonTwo(_, _) => true,
            _ => false,
        }
    }

    pub fn is_access(&self) -> bool {
        match self {
            Step::Access(_) | Step::AccessTwo(_, _) => true,
            _ => false,
        }
    }

    pub fn values(&self) -> Vec<usize> {
        match *self {
            Step::ComparisonTwo(x, y) | Step::AccessTwo(x, y) => vec![x, y],
            Step::Comparison(x) | Step::Access(x) => vec![x],
            Step::None => Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct ArrayState {
    numbers: Vec<usize>,
    view: gui::View,
    step: Step,
    comparisons: u64,
    accesses: u64,
}

impl ArrayState {
    pub fn new(size: usize, view: gui::View) -> ArrayState {
        ArrayState {
            numbers: (1..=size).collect(),
            view,
            step: Step::None,
            comparisons: 0,
            accesses: 0,
        }
    }

    pub fn initialize(&mut self, size: usize) {
        self.numbers = (1..=size).collect();
        self.step = Step::None;
    }

    pub fn get_view(&self) -> gui::View {
        self.view
    }

    pub fn set_view(&mut self, view: gui::View) {
        self.view = view;
    }

    pub fn view(&self) -> iced::Element<'static, crate::Message> {
        iced::Canvas::new(self.clone())
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    pub fn shuffle(&mut self) {
        use rand::prelude::SliceRandom;

        self.numbers.shuffle(&mut rand::thread_rng());
        self.step = Step::None;
    }

    pub fn size(&self) -> usize {
        self.numbers.len()
    }
}

impl ArrayState {
    pub fn clear_step(&mut self) {
        self.step = Step::None;
    }

    pub fn comparisons(&self) -> u64 {
        self.comparisons
    }

    pub fn accesses(&self) -> u64 {
        self.accesses
    }

    pub fn reset_stats(&mut self) {
        self.comparisons = 0;
        self.accesses = 0;
    }

    pub fn cmp_two(&mut self, a: usize, b: usize) -> cmp::Ordering {
        self.step = Step::ComparisonTwo(a, b);
        self.comparisons += 1;
        self.accesses += 2;
        self.numbers[a].cmp(&self.numbers[b])
    }

    pub fn cmp(&mut self, index: usize, value: usize) -> cmp::Ordering {
        self.comparisons += 1;
        self.accesses += 1;
        self.step = Step::Comparison(index);
        self.numbers[index].cmp(&value)
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.accesses += 4;
        self.step = Step::AccessTwo(a, b);
        self.numbers.swap(a, b);
    }

    pub fn get(&mut self, index: usize) -> usize {
        self.accesses += 1;
        self.step = Step::Access(index);
        self.numbers[index]
    }

    pub fn set(&mut self, index: usize, value: usize) {
        self.accesses += 1;
        self.step = Step::Access(index);
        self.numbers[index] = value;
    }
}

impl canvas::Program<crate::Message> for ArrayState {
    fn draw(&self, bounds: iced::Rectangle, _: canvas::Cursor) -> Vec<canvas::Geometry> {
        self.view.draw(bounds, &self.numbers, self.step.clone())
    }
}
