use crate::{array, sort, Message, PADDING};
use iced::{button, canvas, pick_list, slider, text_input};
use strum::IntoEnumIterator;

const WHITE: iced::Color = iced::Color::WHITE;
const BLACK: iced::Color = iced::Color::BLACK;
const RED: iced::Color = iced::Color {
    r: 1f32,
    g: 0f32,
    b: 0f32,
    a: 1f32,
};
const GREEN: iced::Color = iced::Color {
    r: 0f32,
    g: 1f32,
    b: 0f32,
    a: 1f32,
};

const CIRCLE_ACC: u32 = 100;

#[derive(Clone, Debug, PartialEq, Eq, strum::EnumIter)]
pub enum View {
    Default,
    Colors,
    Circle,
}

impl Default for View {
    fn default() -> Self {
        View::Default
    }
}

impl std::fmt::Display for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl View {
    pub fn draw(
        &self,
        bounds: iced::Rectangle,
        numbers: &Vec<usize>,
        step: array::Step,
    ) -> Vec<canvas::Geometry> {
        match self {
            View::Default => {
                let mut frame = canvas::Frame::new(bounds.size());

                frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), BLACK);

                for x in 0..bounds.width as u32 {
                    let index = ((x as f32 / bounds.width) * numbers.len() as f32) as usize;
                    let height = (numbers[index] as f32 / numbers.len() as f32) * bounds.height;

                    let color = if step.contains(index) {
                        if step.is_comparison() {
                            GREEN
                        } else {
                            RED
                        }
                    } else {
                        WHITE
                    };

                    frame.fill_rectangle(
                        iced::Point::new(x as f32, bounds.height - height),
                        iced::Size::new(1.0, height),
                        color,
                    );
                }

                vec![frame.into_geometry()]
            }
            View::Colors => {
                use palette::FromColor;

                let mut frame = canvas::Frame::new(bounds.size());

                for x in 0..bounds.width as u32 {
                    let index = ((x as f32 / bounds.width) * numbers.len() as f32) as usize;
                    let height = bounds.height; //numbers[index] as f32 / numbers.len() as f32) * bounds.height;

                    let color = if step.contains(index) {
                        if step.is_comparison() {
                            WHITE
                        } else {
                            BLACK
                        }
                    } else {
                        palette::rgb::Rgb::from_color(palette::Hsv::new(
                            numbers[index] as f32 / numbers.len() as f32 * 360.0,
                            1f32,
                            1f32,
                        ))
                        .into()
                    };

                    frame.fill_rectangle(
                        iced::Point::new(x as f32, bounds.height - height),
                        iced::Size::new(1.0, height),
                        color,
                    );
                }

                vec![frame.into_geometry()]
            }
            View::Circle => {
                use std::f32::consts::FRAC_PI_4;

                let mut frame = canvas::Frame::new(bounds.size());
                frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), BLACK);

                let circ = canvas::Path::circle(
                    iced::Point::new(bounds.width / 2.0, bounds.height / 2.0),
                    3.0,
                );

                let l = 0.4
                    * std::cmp::min_by(bounds.width, bounds.height, |a, b| {
                        a.partial_cmp(&b).unwrap()
                    });

                for i in 0..CIRCLE_ACC {
                    let r = i as f32 / CIRCLE_ACC as f32;

                    let (mut sin, mut cos) = (r * FRAC_PI_4).sin_cos();
                    sin *= l;
                    cos *= l;

                    let mut index = 0.0;
                    let rn = r / 8.0;

                    let mut flip = true;
                    for (x, y) in [
                        (sin, -cos),
                        (cos, -sin),
                        (cos, sin),
                        (sin, cos),
                        (-sin, cos),
                        (-cos, sin),
                        (-cos, -sin),
                        (-sin, -cos),
                    ] {
                        let c_index;
                        if flip {
                            c_index = ((index + rn) * numbers.len() as f32) as usize;
                            index += 0.25;
                        } else {
                            c_index = ((index - 0.001 - rn) * numbers.len() as f32) as usize;
                        }

                        flip = !flip;

                        let d = numbers[c_index] as f32 / numbers.len() as f32;
                        let color = if step.contains(c_index) {
                            if step.is_comparison() {
                                GREEN
                            } else {
                                RED
                            }
                        } else {
                            WHITE
                        };

                        let translation = iced::Vector::new(x * d, y * d);

                        frame.translate(translation.clone());
                        frame.fill(&circ, color);
                        frame.translate(translation * -1.0);
                    }
                }

                vec![frame.into_geometry()]
            }
        }
    }
}

#[derive(Default)]
pub struct Controls {
    algorithms: pick_list::State<sort::Sort>,
    play: button::State,
    step: button::State,
    speed: slider::State,
    numbers: text_input::State,
    shuffle: button::State,
    view: pick_list::State<View>,
}

impl Controls {
    pub fn view(
        &mut self,
        sort: sort::Sort,
        playing: bool,
        speed: u32,
        max_speed: u32,
        numbers: String,
        view: View,
    ) -> iced::Element<Message> {
        let play_button = iced::Button::new(
            &mut self.play,
            iced::Text::new(if playing { "Stop" } else { "Play" }),
        )
        .on_press(Message::Play);

        let mut shuffle_button = iced::Button::new(&mut self.shuffle, iced::Text::new("Shuffle"));

        let mut step_button = iced::Button::new(&mut self.step, iced::Text::new("Step"));

        if !playing {
            shuffle_button = shuffle_button.on_press(Message::Shuffle);
            step_button = step_button.on_press(Message::Step);
        }

        let algorithm_controls = iced::Column::new()
            .spacing(PADDING)
            .padding(PADDING)
            .width(iced::Length::Fill)
            .push(
                iced::Row::new()
                    .spacing(PADDING)
                    .push(iced::PickList::new(
                        &mut self.algorithms,
                        Vec::from_iter(sort::Sort::iter()),
                        Some(sort),
                        Message::SortSelected,
                    ))
                    .push(play_button)
                    .push(shuffle_button)
                    .push(step_button),
            )
            .push(
                iced::Row::new()
                    .spacing(PADDING)
                    .push(iced::Text::new(format!("Speed: {}", speed)))
                    .push(iced::Slider::new(
                        &mut self.speed,
                        1..=max_speed,
                        speed,
                        Message::SpeedSelected,
                    )),
            );

        let view_controls = iced::Column::new()
            .spacing(PADDING)
            .padding(PADDING)
            .width(iced::Length::Units(250))
            .push(
                iced::Row::new()
                    .spacing(PADDING)
                    .push(iced::Text::new("Numbers:"))
                    .push(
                        iced::TextInput::new(
                            &mut self.numbers,
                            "Input number of elements",
                            &numbers,
                            Message::NumbersInput,
                        )
                        .on_submit(Message::NumbersSelected),
                    ),
            )
            .push(
                iced::Row::new()
                    .spacing(10)
                    .push(iced::Text::new("View:"))
                    .push(iced::PickList::new(
                        &mut self.view,
                        Vec::from_iter(View::iter()),
                        Some(view),
                        Message::ViewSelected,
                    )),
            );

        iced::Row::new()
            .height(iced::Length::Units(100))
            .spacing(5)
            .align_items(iced::Alignment::Center)
            .push(algorithm_controls)
            .push(iced::Rule::vertical(5))
            .push(view_controls)
            .into()
    }
}
