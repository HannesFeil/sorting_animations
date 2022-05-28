use crate::ArrayState;

#[derive(Clone, Debug, PartialEq, Eq, strum::EnumIter)]
pub enum SortSelection {
    BubbleSort,
    InsertionSort,
}

impl Default for SortSelection {
    fn default() -> Self {
        SortSelection::BubbleSort
    }
}

impl std::fmt::Display for SortSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            SortSelection::BubbleSort => "Bubble-Sort",
            SortSelection::InsertionSort => "Insertion-Sort",
        })
    }
}

impl SortSelection {
    pub fn initialize_sort(&self, numbers: usize) -> Box<dyn Sort> {
        match self {
            SortSelection::BubbleSort => Box::new(BubbleSort {
                upper_bound: numbers - 1,
                index: 0,
                cmp: None,
            }),
            SortSelection::InsertionSort => Box::new(InsertionSort {
               current_index: 1,
                current: None,
                temp_index: 0,
                cmp: None,
            }),
        }
    }
}

pub trait Sort {
    fn step(&mut self, numbers: &mut ArrayState) -> bool;

    fn max_speed(&self) -> u32;

    fn steps_per_tick(&self, speed: u32) -> u32;
}

pub struct BubbleSort {
    upper_bound: usize,
    index: usize,
    cmp: Option<bool>,
}

impl Sort for BubbleSort {
    fn step(&mut self, numbers: &mut ArrayState) -> bool {
        if self.index < self.upper_bound {
            if let Some(greater) = self.cmp {
                if greater {
                    numbers.swap(self.index, self.index + 1)
                }

                self.cmp = None;
                self.index += 1;
            } else {
                self.cmp = Some(numbers.compare(self.index, self.index + 1).is_gt());
            }
        } else {
            self.upper_bound -= 1;
            self.index = 0;
        }

        self.upper_bound == 0
    }

    fn max_speed(&self) -> u32 {
        500u32
    }

    fn steps_per_tick(&self, speed: u32) -> u32 {
        speed * speed
    }
}

pub struct InsertionSort {
    current: Option<usize>,
    current_index: usize,
    temp_index: usize,
    cmp: Option<bool>,
}

impl Sort for InsertionSort {
    fn step(&mut self, numbers: &mut ArrayState) -> bool {
        if self.current_index < numbers.count() {
            match self.current {
                Some(current) => {
                    match self.cmp {
                        Some(cmp) => {
                            if cmp {
                                numbers.swap(self.temp_index - 1, self.temp_index);
                                self.temp_index -= 1;
                            } else {
                                numbers.set(self.temp_index, current);
                                self.current = None;
                                self.current_index += 1;
                            }

                            self.cmp = None;
                        },
                        None => {
                            self.cmp = Some(self.temp_index > 0
                                && numbers.compare_to(self.temp_index - 1, current).is_gt())
                        }
                    }
                },
                None => {
                    self.current = Some(numbers.get(self.current_index));
                    self.temp_index = self.current_index;
                }
            }
        }

        self.current_index >= numbers.count()
    }

    fn max_speed(&self) -> u32 {
        500u32
    }

    fn steps_per_tick(&self, speed: u32) -> u32 {
        speed * speed
    }
}

