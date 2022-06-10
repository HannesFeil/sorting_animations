#![feature(int_log)]

use std::time;

const TITLE: &str = "Sorting Animations";
const PADDING: u16 = 15;
const INITIAL_NUMBERS: usize = 100;
const MIN_NUMBERS: usize = 10;
const DELAY_TIME: time::Duration = time::Duration::from_millis(10);

mod array;
mod gui;
mod sort;

pub fn main() -> iced::Result {
    use iced::Application;

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        if !panic_info.to_string().contains("Canceled sort:") {
            hook(panic_info);
        }
    }));

    SortingAnimations::run(iced::Settings {
        antialiasing: true,
        window: iced::window::Settings {
            position: iced::window::Position::Centered,

            ..iced::window::Settings::default()
        },

        ..iced::Settings::default()
    })
}

#[derive(Debug, Clone)]
pub enum Message {
    Play,
    Shuffle,
    Step,
    Tick,

    SortSelected(sort::Sort),
    ViewSelected(gui::View),
    SpeedSelected(u32),
    NumbersInput(String),
    NumbersSelected,
}

struct SortingAnimations {
    controls: gui::Controls,
    sorter: sort::Sorter,
    changed_numbers: Option<usize>,
    reset_stats: bool,
}

impl iced::Application for SortingAnimations {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let mut animations = SortingAnimations {
            controls: gui::Controls::default(),
            sorter: sort::Sorter::new(array::ArrayState::new(
                INITIAL_NUMBERS,
                gui::View::default(),
            )),
            changed_numbers: Some(INITIAL_NUMBERS),
            reset_stats: false,
        };
        animations.initialize_sort(sort::Sort::default());

        (animations, iced::Command::none())
    }

    fn title(&self) -> String {
        String::from(TITLE)
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::Play => {
                if self.reset_stats {
                    self.sorter.reset_stats();
                    self.reset_stats = false;
                }

                self.sorter.set_playing(!self.sorter.playing());
            }
            Message::Shuffle => {
                self.initialize_sort(self.sorter.sort());

                self.sorter.operate_array(array::ArrayState::shuffle);
            }
            Message::Step => {
                if self.reset_stats {
                    self.sorter.reset_stats();
                    self.reset_stats = false;
                }

                self.sorter.step();
            }
            Message::Tick => {
                if !self.sorter.check_alive() {
                    self.initialize_sort(self.sorter.sort());
                }
            }
            Message::SortSelected(sort) => {
                self.initialize_sort(sort);
            }
            Message::ViewSelected(view) => {
                self.sorter.set_view(view);
            }
            Message::SpeedSelected(speed) => {
                self.sorter.set_speed(speed);
            }
            Message::NumbersInput(nums) => {
                if nums.trim().is_empty() {
                    self.changed_numbers = None;
                } else {
                    if let Ok(number) = nums.trim().parse::<usize>() {
                        self.changed_numbers = Some(number);
                    }
                }
            }
            Message::NumbersSelected => {
                self.sorter.kill_sort();

                self.changed_numbers = self.changed_numbers.map_or(Some(INITIAL_NUMBERS), |n| {
                    Some(std::cmp::max(MIN_NUMBERS, n))
                });

                self.sorter
                    .operate_array(|array| array.initialize(self.changed_numbers.unwrap()));

                self.sorter.start_sort();
            }
        }

        iced::Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(DELAY_TIME).map(|_| Message::Tick)
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let content = iced::Column::new()
            .push(
                iced::Row::new()
                    .padding(PADDING)
                    .push(iced::Text::new(format!(
                        "Comparisons: {}",
                        self.sorter.comparisons()
                    )))
                    .push(iced::Space::new(
                        iced::Length::Units(100),
                        iced::Length::Shrink,
                    ))
                    .push(iced::Text::new(format!(
                        "Accesses: {}",
                        self.sorter.accesses()
                    ))),
            )
            .push(self.sorter.operate_array(|array| array.view()))
            .push(
                self.controls.view(
                    self.sorter.sort(),
                    self.sorter.playing(),
                    self.sorter.speed(),
                    sort::MAX_SPEED,
                    self.changed_numbers
                        .map_or(String::new(), |x| x.to_string()),
                    self.sorter.view(),
                ),
            );

        iced::Container::new(content).into()
    }
}

impl SortingAnimations {
    fn initialize_sort(&mut self, sort: sort::Sort) {
        self.reset_stats = true;

        self.sorter.kill_sort();
        self.sorter.operate_array(|a| a.clear_step());
        self.sorter.set_sort(sort);
        self.sorter.start_sort();
    }
}
