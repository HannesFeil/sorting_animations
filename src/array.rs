use crate::gui;
use iced::canvas;
use std::cmp;

#[derive(Clone)]
pub enum Step {
    Comparison(usize, usize),
    Access(usize, usize),
    None,
}

#[derive(Clone)]
pub struct ArrayState {
    numbers: Vec<usize>,
    view: gui::View,
    step: Step,
}

impl ArrayState {
    pub fn new(size: usize, view: gui::View) -> ArrayState {
        ArrayState {
            numbers: (1..=size).collect(),
            view,
            step: Step::None,
        }
    }

    pub fn initialize(&mut self, size: usize) {
        self.numbers = (1..=size).collect();
        self.step = Step::None;
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

    pub fn cmp_two(&mut self, a: usize, b: usize) -> cmp::Ordering {
        self.step = Step::Comparison(a, b);
        self.numbers[a].cmp(&self.numbers[b])
    }

    pub fn cmp(&mut self, index: usize, value: usize) -> cmp::Ordering {
        self.step = Step::Comparison(index, index);
        self.numbers[index].cmp(&value)
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.step = Step::Access(a, b);
        self.numbers.swap(a, b);
    }

    pub fn get(&mut self, index: usize) -> usize {
        self.step = Step::Access(index, index);
        self.numbers[index]
    }

    pub fn set(&mut self, index: usize, value: usize) {
        self.step = Step::Access(index, index);
        self.numbers[index] = value;
    }
}

impl canvas::Program<crate::Message> for ArrayState {
    fn draw(&self, bounds: iced::Rectangle, _: canvas::Cursor) -> Vec<canvas::Geometry> {
        self.view.draw(bounds, &self.numbers, self.step.clone())
    }
}
