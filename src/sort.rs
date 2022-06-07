use crate::{array, gui};
use std::{cmp, sync, thread};
use strum::EnumIter;

type SyncLock = sync::Arc<sync::Mutex<SortLock>>;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum Sort {
    BubbleSort,
    InsertionSort,
}

impl Default for Sort {
    fn default() -> Self {
        Sort::BubbleSort
    }
}

impl std::fmt::Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Sort {
    fn sort(&self, size: usize, lock: SyncLock) {
        match self {
            Sort::BubbleSort => self.bubble_sort(size, &lock),
            Sort::InsertionSort => self.insertion_sort(size, &lock),
        }
    }

    fn bubble_sort(&self, size: usize, lock: &SyncLock) {
        for i in 1..size {
            for j in 0..size - i {
                if self.cmp_two(lock, j, j + 1).is_gt() {
                    self.swap(lock, j, j + 1);
                }
            }
        }
    }

    fn insertion_sort(&self, size: usize, lock: &SyncLock) {
        for i in 1..size {
            let current = self.get(lock, i);

            let mut j = i;
            while j > 0 && self.cmp(lock, j - 1, current).is_gt() {
                self.set(lock, j, self.get(lock, j - 1));
                j -= 1;
            }

            self.set(lock, j, current);
        }
    }

    pub fn max_speed(&self) -> u32 {
        match self {
            Sort::BubbleSort | Sort::InsertionSort => 500,
        }
    }

    pub fn calculate_ticks(&self, speed: u32) -> u64 {
        match self {
            Sort::BubbleSort | Sort::InsertionSort => (speed * speed) as u64,
        }
    }
}

impl Sort {
    fn add_comparisons(lock: &SyncLock, comparisons: u64) {
        lock.lock().unwrap().comparisons += comparisons;
    }

    fn add_accesses(lock: &SyncLock, accesses: u64) {
        lock.lock().unwrap().accesses += accesses;
    }

    fn perform_step<F, T>(&self, lock: &SyncLock, step: F) -> T
    where
        F: FnOnce(&mut array::ArrayState) -> T,
    {
        loop {
            let mut lock = lock.lock().unwrap();

            match lock.playing {
                Some(playing) => {
                    if lock.steps > 0 {
                        lock.steps -= 1;
                        return step(&mut lock.array_state);
                    } else {
                        if playing {
                            lock.steps = self.calculate_ticks(lock.speed);
                        }

                        drop(lock);
                        thread::sleep(crate::DELAY_TIME);
                        continue;
                    }
                }
                None => {
                    drop(lock);
                    panic!("Canceled sort: {}", self)
                }
            }
        }
    }
}

macro_rules! wrap_array_op {
    ($name:ident, comparisons: $c:expr, accesses:$a:expr, ($($arg:ident : $argty:ty),*) -> $ret:ty) => {
        fn $name(&self, lock: &SyncLock, $($arg:$argty),*) -> $ret {
            let r = self.perform_step(lock, |array_state_argument| {
                array_state_argument.$name($($arg),*)
            });
            Sort::add_comparisons(lock, $c);
            Sort::add_accesses(lock, $a);
            r
        }
    };
}

impl Sort {
    wrap_array_op!(cmp_two, comparisons: 1, accesses: 2, (a:usize, b:usize) -> cmp::Ordering);
    wrap_array_op!(swap, comparisons: 0, accesses: 2, (a:usize, b:usize) -> ());
    wrap_array_op!(cmp, comparisons: 1, accesses: 1, (index:usize, value:usize) -> cmp::Ordering);
    wrap_array_op!(get, comparisons: 0, accesses: 1, (index:usize) -> usize);
    wrap_array_op!(set, comparisons: 0, accesses: 1, (index:usize, value:usize) -> ());
}

pub struct Sorter {
    lock: SyncLock,
    sort: Sort,
    handle: Option<thread::JoinHandle<()>>,
}

impl Sorter {
    pub fn new(array_state: array::ArrayState) -> Sorter {
        Sorter {
            lock: sync::Arc::new(sync::Mutex::new(SortLock::new(array_state))),
            sort: Sort::default(),
            handle: None,
        }
    }

    pub fn start_sort(&mut self) {
        assert!(!self.check_alive(), "Sort already running");
        self.set_playing(false);

        self.lock.lock().unwrap().steps = 0;

        let sort = self.sort.clone();
        let lock = self.lock.clone();
        let size = lock.lock().unwrap().array_state.size();

        self.handle = Some(thread::spawn(move || sort.sort(size, lock)));
    }

    pub fn kill_sort(&mut self) {
        self.lock.lock().unwrap().playing = None;

        if self.check_alive() {
            std::mem::replace::<Option<thread::JoinHandle<()>>>(&mut self.handle, None)
                .unwrap()
                .join()
                .unwrap_or_default();
        }
    }

    pub fn set_sort(&mut self, sort: Sort) {
        assert!(!self.alive());

        self.sort = sort;
    }

    pub fn sort(&self) -> Sort {
        self.sort.clone()
    }

    pub fn operate_array<T>(&mut self, f: impl FnOnce(&mut array::ArrayState) -> T) -> T {
        f(&mut self.lock.lock().unwrap().array_state)
    }

    pub fn check_alive(&mut self) -> bool {
        if let Some(handle) = &self.handle {
            if handle.is_finished() {
                self.handle = None;
            }
        }

        self.alive()
    }

    pub fn alive(&self) -> bool {
        self.handle.is_some()
    }

    pub fn step(&self) {
        self.lock.lock().unwrap().steps = 1;
    }

    pub fn playing(&self) -> bool {
        self.lock.lock().unwrap().playing.unwrap_or(false)
    }

    pub fn set_playing(&self, playing: bool) {
        self.lock.lock().unwrap().playing = Some(playing);
    }

    pub fn speed(&self) -> u32 {
        self.lock.lock().unwrap().speed
    }

    pub fn set_speed(&self, speed: u32) {
        self.lock.lock().unwrap().speed = speed;
    }

    pub fn comparisons(&self) -> u64 {
        self.lock.lock().unwrap().comparisons
    }

    pub fn accesses(&self) -> u64 {
        self.lock.lock().unwrap().accesses
    }

    pub fn reset_stats(&self) {
        let mut lock = self.lock.lock().unwrap();

        lock.comparisons = 0;
        lock.accesses = 0;
    }

    pub fn set_view(&self, view: gui::View) {
        self.lock.lock().unwrap().array_state.set_view(view);
    }
}

struct SortLock {
    array_state: array::ArrayState,
    comparisons: u64,
    accesses: u64,
    speed: u32,
    steps: u64,
    playing: Option<bool>,
}

impl SortLock {
    fn new(array_state: array::ArrayState) -> SortLock {
        SortLock {
            array_state,
            comparisons: 0,
            accesses: 0,
            speed: 1,
            steps: 0,
            playing: Some(false),
        }
    }
}
