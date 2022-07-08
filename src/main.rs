#![feature(int_log)]

use std::time;

const TITLE: &str = "Sorting Animations";
const PADDING: u16 = 15;
const INITIAL_NUMBERS: usize = 100;
const MIN_NUMBERS: usize = 10;
const DELAY_TIME: time::Duration = time::Duration::from_millis(10);
const MAX_SPEED: u32 = 100;
const TIME_OUT_CHECK: u64 = 10000;

mod array;
mod gui;
mod sorting;

pub trait EnumListable<E, const N: usize> {
    fn list() -> [E; N];
}

pub fn main() -> iced::Result {
    use iced::Application;

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
    Reverse,
    Step,
    Mute(bool),
    Tick(time::Instant),

    SortSelected(sorting::Sort),
    ViewSelected(gui::View),
    SpeedSelected(u32),
    NumbersInput(String),
    NumbersSelected,
}

struct SortingAnimations {
    controls: gui::Controls,
    sorter: sorting::Sorter,
    playing: bool,
    speed: u32,
    changed_numbers: Option<usize>,
    reset_stats: bool,
    muted: bool,
    sink: rodio::Sink,
    _stream: rodio::OutputStream,
}

impl iced::Application for SortingAnimations {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();
        sink.set_volume(0.1);
        sink.append(rodio::source::SineWave::new(440.0));
        sink.pause();

        let mut animations = SortingAnimations {
            controls: gui::Controls::default(),
            sorter: sorting::Sorter::new(array::ArrayState::new(
                INITIAL_NUMBERS,
                gui::View::default(),
            )),
            playing: false,
            speed: 1,
            changed_numbers: Some(INITIAL_NUMBERS),
            reset_stats: false,
            muted: true,
            sink,
            _stream,
        };
        animations.initialize_sort(sorting::Sort::default());

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

                self.playing = !self.playing;
                if !self.playing {
                    self.sink.pause();
                }
            }
            Message::Shuffle => {
                self.initialize_sort(self.sorter.sort());

                self.sorter.shuffle();
            }
            Message::Reverse => {
                self.initialize_sort(self.sorter.sort());

                self.sorter.reverse();
            }
            Message::Step => {
                if self.reset_stats {
                    self.sorter.reset_stats();
                    self.reset_stats = false;
                }

                self.sorter.step();
            }
            Message::Tick(_instant) => {
                self.sink.set_speed(match self.sorter.last_step() {
                    array::Step::None => self.sink.speed(),
                    _ => {
                        0.5 + (self.sorter.last_step().values().iter().sum::<usize>() as f32
                            / self.sorter.last_step().values().len() as f32)
                            / self.sorter.size() as f32
                            / 2.0
                    }
                });
                if !self.sorter.alive() {
                    self.playing = false;
                    self.initialize_sort(self.sorter.sort());
                } else if self.playing {
                    if self.sink.is_paused() && !self.muted {
                        self.sink.play()
                    }

                    self.sorter.tick(self.speed as f32 / MAX_SPEED as f32);
                }
            }
            Message::SortSelected(sort) => {
                self.initialize_sort(sort);
            }
            Message::ViewSelected(view) => {
                self.sorter.set_view(view);
            }
            Message::SpeedSelected(speed) => {
                self.speed = speed;
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
                self.playing = false;
                self.sorter.kill_sort();

                self.changed_numbers = self.changed_numbers.map_or(Some(INITIAL_NUMBERS), |n| {
                    Some(std::cmp::max(MIN_NUMBERS, n))
                });

                self.sorter.initialize(self.changed_numbers.unwrap());
                self.sorter.start_sort();
            }
            Message::Mute(muted) => {
                self.muted = muted;
                if self.muted {
                    self.sink.pause();
                }
            }
        }

        iced::Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(DELAY_TIME).map(Message::Tick)
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
                    )))
                    .push(iced::Space::new(iced::Length::Fill, iced::Length::Shrink))
                    .push(
                        iced::Toggler::new(self.muted, String::from("Mute "), Message::Mute)
                            .width(iced::Length::Shrink),
                    ),
            )
            .push(self.sorter.array_view())
            .push(
                self.controls.view(
                    self.sorter.sort(),
                    self.playing,
                    self.speed,
                    MAX_SPEED,
                    self.changed_numbers
                        .map_or(String::new(), |x| x.to_string()),
                    self.sorter.get_view(),
                ),
            );

        iced::Container::new(content).into()
    }
}

impl SortingAnimations {
    fn initialize_sort(&mut self, sort: sorting::Sort) {
        self.reset_stats = true;
        self.playing = false;
        self.sink.pause();

        self.sorter.kill_sort();
        self.sorter.clear_step();
        self.sorter.set_sort(sort);
        self.sorter.start_sort();
    }
}
