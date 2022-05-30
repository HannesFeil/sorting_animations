use iced::{button, canvas, executor, pick_list, slider, text_input};
use palette::FromColor;
use rand::Rng;
use strum::IntoEnumIterator;

mod sort;

fn main() -> iced::Result {
    use iced::Application;

    SortingAnimations::run(iced::Settings {
        antialiasing: true,
        window:  iced::window::Settings {
            position: iced::window::Position::Centered,

            ..iced::window::Settings::default()
        },

        ..iced::Settings::default()
    })
}

#[derive()]
struct SortingAnimations {
    array_state: ArrayState,
    controls: Controls,
    sort_selection: sort::SortSelection,
    sort: Box<dyn sort::Sort>,
    playing: bool,
    finished: bool,
    speed: u32,
    changed_numbers: String,
}

#[derive(Clone, Debug)]
enum Message {
    Play,
    Tick,
    Shuffle,
    Step,

    SelectedSort(sort::SortSelection),
    SelectedView(View),
    SelectedSpeed(u32),
    SelectedNumbersChanged(String),
    SelectedNumbers,
}

impl Message {}

impl iced::Application for SortingAnimations {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (SortingAnimations {
            array_state: ArrayState::new(1000),
            controls: Controls::default(),
            sort_selection: sort::SortSelection::BubbleSort,
            sort: sort::SortSelection::default().initialize_sort(1000),
            playing: false,
            finished: false,
            speed: 1,
            changed_numbers: String::from("1000"),
        }, iced::Command::none())
    }

    fn title(&self) -> String {
        String::from("Sorting Animations")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::Play => {
                if self.finished {
                    self.reset_sort();
                }

                self.playing = !self.playing
            },
            Message::Tick => {
                if self.playing {
                    for _ in 0..self.sort.steps_per_tick(self.speed) {
                        if self.sort.step(&mut self.array_state) {
                            self.finished = true;
                            self.playing = false;
                            self.array_state.last_step = Step::None;

                            break;
                        }
                    }
                }
            },
            Message::Shuffle => {
                self.array_state.shuffle();
                self.reset_sort();
            },
            Message::Step => {
                if self.finished {
                    self.reset_sort();
                }

                self.finished = self.sort.step(&mut self.array_state)
            },
            Message::SelectedSort(sort) => {
                self.sort_selection = sort;
                self.reset_sort();
            },
            Message::SelectedView(view) => self.array_state.view = view,
            Message::SelectedSpeed(speed) => self.speed = speed,
            Message::SelectedNumbersChanged(numbers) => {
                if numbers.chars().all(|x| x.is_ascii_digit()) || numbers.trim().is_empty() {
                    self.changed_numbers = numbers.trim().to_string();
                }
            },
            Message::SelectedNumbers => {
                if let Ok(numbers) = self.changed_numbers.parse::<usize>() {
                    self.array_state.reset_numbers(numbers);
                    self.reset_sort();
                } else {
                    self.changed_numbers = self.array_state.count().to_string();
                }
            },
        }

        iced::Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        if self.playing {
            iced::time::every(std::time::Duration::from_millis(10)).map(|_| Message::Tick)
        } else {
            iced::Subscription::none()
        }
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let view_mode = self.array_state.view.clone();

        let content = iced::Column::new()
            .spacing(10)
            .padding(10)
            .push(iced::Row::new()
                .push(iced::Text::new(format!("Comparisons: {}", self.array_state.comparisons())))
                .push(iced::Space::new(iced::Length::Fill, iced::Length::Shrink))
                .push(iced::Text::new(format!("Accesses: {}", self.array_state.accesses()))))
            .push(self.array_state.view())
            .push(self.controls.view(
                self.sort_selection.clone(),
                self.playing,
                self.speed,
                self.sort.max_speed(),
                self.changed_numbers.clone(),
                view_mode));

        iced::Container::new(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

impl SortingAnimations {
    fn reset_sort(&mut self) {
        self.playing = false;
        self.finished = false;
        self.sort = self.sort_selection.initialize_sort(self.array_state.count());
        self.array_state.reset_sort_stats();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, strum::EnumIter)]
enum View {
    Default,
    Colors,
}

impl std::fmt::Display for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            View::Default => "Default",
            View::Colors => "Colors",
        })
    }
}

impl View {
    const RED: iced::Color = iced::Color::from_rgb(1.0, 0.0, 0.0);
    const GREEN: iced::Color = iced::Color::from_rgb(0.0, 1.0, 0.0);
    const GRAY: iced::Color = iced::Color::from_rgb(0.3, 0.3, 0.3);

    pub fn compare_color(&self) -> iced::Color {
        match self {
            View::Default => View::GREEN,
            View::Colors => iced::Color::WHITE,
        }
    }

    pub fn access_color(&self) -> iced::Color {
        match self {
            View::Default => View::RED,
            View::Colors => View::GRAY,
        }
    }

    pub fn color(&self, ratio: f32) -> iced::Color {
        match self {
            View::Default => iced::Color::WHITE,
            View::Colors => {
                let (r, g, b) = palette::rgb::Rgb::from_color(
                    palette::Hsv::new(ratio * 360.0, 1.0, 1.0)).into();
                iced::Color::from_rgb(r, g, b)
            },
        }
    }
}

pub struct ArrayState {
    numbers: Vec<usize>,
    last_step: Step,
    comparisons: u32,
    accesses: u32,
    view: View,
}

pub enum Step {
    Compare(usize, usize),
    Swap(usize, usize),
    Get(usize),
    Set(usize),
    None,
}

impl ArrayState {
    pub fn new(numbers: usize) -> ArrayState {
        ArrayState {
            numbers: (1..=numbers).collect(),
            last_step: Step::None,
            comparisons: 0,
            accesses: 0,
            view: View::Default,
        }
    }

    pub fn reset_numbers(&mut self, numbers: usize) {
        self.numbers = (1..=numbers).collect();
        self.last_step = Step::None;
    }

    pub fn comparisons(&self) -> u32 {
        self.comparisons
    }

    pub fn accesses(&self) -> u32 {
        self.accesses
    }

    pub fn reset_sort_stats(&mut self) {
        self.comparisons = 0;
        self.accesses = 0;
    }

    pub fn count(&self) -> usize {
        self.numbers.len()
    }

    pub fn compare(&mut self, i: usize, j: usize) -> std::cmp::Ordering {
        self.last_step = Step::Compare(i, j);
        self.comparisons += 1;
        self.numbers[i].cmp(&self.numbers[j])
    }

    pub fn compare_to(&mut self, i: usize, number: usize) -> std::cmp::Ordering {
        self.last_step = Step::Get(i);
        self.comparisons += 1;
        self.numbers[i].cmp(&number)
    }

    pub fn get(&mut self, i: usize) -> usize {
        self.last_step = Step::Get(i);
        self.accesses += 1;
        self.numbers[i]
    }

    pub fn set(&mut self, i: usize, value: usize) {
        self.last_step = Step::Set(i);
        self.accesses += 1;
        self.numbers[i] = value;
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.last_step = Step::Swap(i, j);
        self.accesses += 2;
        self.numbers.swap(i, j)
    }

    pub fn shuffle(&mut self) {
        for i in 0..self.numbers.len() {
            let r = rand::thread_rng().gen_range(0..self.count());
            self.numbers.swap(i, r);
        }

        self.last_step = Step::None;
    }

    fn view(&mut self) -> iced::Element<Message> {
        iced::Canvas::new(self)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

impl canvas::Program<Message> for ArrayState {
    fn draw(&self, bounds: iced::Rectangle, _cursor: canvas::Cursor) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(bounds.size());

        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), iced::Color::BLACK);

        for x in 0..bounds.width as u32 {
            let index = ((x as f32 / bounds.width) * self.numbers.len() as f32) as usize;
            let height = (self.numbers[index] as f32 / self.numbers.len() as f32) * bounds.height;

            let color = match self.last_step {
                Step::Compare(i, j) if i == index || j == index => self.view.compare_color(),
                Step::Swap(i, j) if i == index || j == index => self.view.access_color(),
                Step::Get(i) | Step::Set(i) if i == index => self.view.access_color(),
                _ => self.view.color(self.numbers[index] as f32 / self.numbers.len() as f32)
            };

            frame.fill_rectangle(
                iced::Point::new(x as f32, bounds.height - height),
                iced::Size::new(1.0, height),
                color);
            }

        vec![frame.into_geometry()]
    }
}

#[derive(Default)]
struct Controls {
    algorithms: pick_list::State<sort::SortSelection>,
    play: button::State,
    step: button::State,
    speed: slider::State,
    numbers: text_input::State,
    shuffle: button::State,
    view: pick_list::State<View>,
}

impl Controls {
    fn view(
        &mut self,
        sort: sort::SortSelection,
        playing: bool,
        speed: u32,
        max_speed: u32,
        numbers: String,
        view: View,
    ) -> iced::Element<Message> {
        let play_button = iced::Button::new(
            &mut self.play,
            iced::Text::new(if playing { "Stop" } else { "Play" }))
            .on_press(Message::Play);

        let mut shuffle_button = iced::Button::new(
            &mut self.shuffle,
            iced::Text::new("Shuffle"));

        let mut step_button = iced::Button::new(
            &mut self.step,
            iced::Text::new("Step"));

        if !playing {
            shuffle_button = shuffle_button.on_press(Message::Shuffle);
            step_button = step_button.on_press(Message::Step);
        }

        let algorithm_controls = iced::Column::new()
            .spacing(10)
            .push(iced::Row::new()
                .spacing(10)
                .push(iced::PickList::new(
                    &mut self.algorithms,
                    sort::SortSelection::iter().collect::<Vec<sort::SortSelection>>(),
                    Some(sort),
                    Message::SelectedSort))
                .push(play_button)
                .push(shuffle_button)
                .push(step_button))
            .push(iced::Row::new()
                .spacing(10)
                .push(iced::Text::new(format!("Speed: {}", speed)))
                .push(iced::Slider::new(
                    &mut self.speed,
                    0..=max_speed,
                    speed,
                    Message::SelectedSpeed)))
            .width(iced::Length::Fill)
            .padding(20);

        let view_controls = iced::Column::new()
            .spacing(10)
            .push(iced::Row::new()
                .spacing(10)
                .push(iced::Text::new("Numbers:"))
                .push(iced::TextInput::new(
                    &mut self.numbers,
                    "Input number of elements",
                    &numbers,
                    Message::SelectedNumbersChanged)
                    .on_submit(Message::SelectedNumbers)))
            .push(iced::Row::new()
                .spacing(10)
                .push(iced::Text::new("View:"))
                .push(iced::PickList::new(
                    &mut self.view,
                    View::iter().collect::<Vec<View>>(),
                    Some(view),
                    Message::SelectedView)))
            .width(iced::Length::Units(250))
            .padding(20);

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