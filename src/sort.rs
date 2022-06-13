use crate::{array, gui};
use std::{cmp, sync, thread};
use strum::EnumIter;

type SyncLock = sync::Arc<sync::Mutex<SortLock>>;

pub const MAX_SPEED: u32 = 500;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum Sort {
    BubbleSort,
    ShakerSort,
    ExchangeSort,
    CycleSort,
    CombSort,
    OddEvenSort,
    InsertionSort,
    ShellSort,
    SelectionSort,
    DoubleSelectionSort,
    StrandSort,
    StoogeSort,
    SlowSort,
    QuickSort,
    MergeSort,
    HeapSort,
    CountingSort,
    RadixSort10,
    RadixSort2,
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
            Sort::ShakerSort => self.shaker_sort(size, &lock),
            Sort::InsertionSort => self.insertion_sort(size, &lock),
            Sort::SelectionSort => self.selection_sort(size, &lock),
            Sort::DoubleSelectionSort => self.double_selection_sort(size, &lock),
            Sort::StoogeSort => self.stooge_sort(0, size - 1, &lock),
            Sort::QuickSort => self.quick_sort(0, size - 1, &mut rand::thread_rng(), &lock),
            Sort::MergeSort => self.merge_sort(0, size - 1, &lock),
            Sort::HeapSort => self.heap_sort(size - 1, &lock),
            Sort::ExchangeSort => self.exchange_sort(size, &lock),
            Sort::CycleSort => self.cycle_sort(size, &lock),
            Sort::StrandSort => self.strand_sort(size, &lock),
            Sort::CombSort => self.comb_sort(size, &lock),
            Sort::OddEvenSort => self.odd_even_sort(size, &lock),
            Sort::ShellSort => self.shell_sort(size, &lock),
            Sort::CountingSort => self.counting_sort(size, size, |x| x, &lock),
            Sort::RadixSort10 => self.radix_sort(size, 10, &lock),
            Sort::RadixSort2 => self.radix_sort(size, 2, &lock),
            Sort::SlowSort => self.slow_sort(0, size - 1, &lock),
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

    fn shaker_sort(&self, size: usize, lock: &SyncLock) {
        for i in 1..size / 2 {
            for j in i - 1..size - i {
                if self.cmp_two(lock, j, j + 1).is_gt() {
                    self.swap(lock, j, j + 1);
                }
            }

            for j in (i..size - i).rev() {
                if self.cmp_two(lock, j, j - 1).is_lt() {
                    self.swap(lock, j, j - 1);
                }
            }
        }
    }

    fn exchange_sort(&self, size: usize, lock: &SyncLock) {
        for i in 0..size - 1 {
            for j in i + 1..size {
                if self.cmp_two(lock, i, j).is_gt() {
                    self.swap(lock, i, j);
                }
            }
        }
    }

    fn cycle_sort(&self, size: usize, lock: &SyncLock) {
        let mut buf = vec![false; size];
        for i in 0..size - 1 {
            if buf[i] {
                continue;
            }

            let mut current = self.get(lock, i);
            loop {
                let index = (0..size)
                    .filter(|x| *x != i && self.cmp(lock, *x, current).is_lt())
                    .count();

                if index != i {
                    buf[index] = true;

                    let new = self.get(lock, index);
                    self.set(lock, index, current);
                    current = new;
                } else {
                    self.set(lock, i, current);

                    break;
                }
            }
        }
    }

    fn comb_sort(&self, size: usize, lock: &SyncLock) {
        let mut gap = size;
        const SHRINK: f32 = 1.3;
        let mut sorted = false;

        while !sorted {
            gap = (gap as f32 / SHRINK) as usize;
            if gap <= 1 {
                gap = 1;
                sorted = true;
            }

            for i in 0..size - gap {
                if self.cmp_two(lock, i, i + gap).is_gt() {
                    self.swap(lock, i, i + gap);
                    sorted = false;
                }
            }
        }
    }

    fn odd_even_sort(&self, size: usize, lock: &SyncLock) {
        let mut sorted = false;

        while !sorted {
            sorted = true;

            for start in 0..=1 {
                for i in (start..size - 1).step_by(2) {
                    if self.cmp_two(lock, i, i + 1).is_gt() {
                        self.swap(lock, i, i + 1);
                        sorted = false;
                    }
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

    fn shell_sort(&self, size: usize, lock: &SyncLock) {
        let mut gap = size;

        while gap > 1 {
            gap = cmp::max(1, gap / 2);

            for i in gap..size {
                let tmp = self.get(lock, i);

                let mut j = i;

                while j >= gap && self.cmp(lock, j - gap, tmp).is_gt() {
                    self.set(lock, j, self.get(lock, j - gap));
                    j -= gap;
                }

                self.set(lock, j, tmp);
            }
        }
    }

    fn selection_sort(&self, size: usize, lock: &SyncLock) {
        for i in 0..size - 1 {
            let mut min = i;
            for j in i + 1..size {
                if self.cmp_two(lock, min, j).is_gt() {
                    min = j;
                }
            }
            if min != i {
                self.swap(lock, min, i);
            }
        }
    }

    fn double_selection_sort(&self, size: usize, lock: &SyncLock) {
        for i in 0..size / 2 {
            let mut min = i;
            let mut max = size - i - 1;

            for j in i + 1..size - i {
                if self.cmp_two(lock, min, j).is_gt() {
                    min = j;
                }
            }
            if min != i {
                self.swap(lock, min, i);
            }

            for j in (i + 1..size - i - 1).rev() {
                if self.cmp_two(lock, max, j).is_lt() {
                    max = j;
                }
            }
            if max != size - i - 1 {
                self.swap(lock, max, size - i - 1);
            }
        }
    }

    fn strand_sort(&self, size: usize, lock: &SyncLock) {
        let mut index = 0;
        while index < size {
            let mut len = 1;

            for j in index + 1..size {
                if self.cmp_two(lock, index + len - 1, j).is_lt() && j != index + len {
                    self.swap(lock, j, index + len);
                    len += 1;
                }
            }

            let old_index = index;
            index += len;

            let mut tmp = Vec::with_capacity(index);

            let mut x = 0;
            let mut y = 0;
            for _ in 0..index {
                if x >= old_index || y < len && self.cmp_two(lock, old_index + y, x).is_lt() {
                    tmp.push(self.get(lock, old_index + y));
                    y += 1;
                } else {
                    tmp.push(self.get(lock, x));
                    x += 1;
                }
            }

            for (i, v) in tmp.iter().enumerate() {
                self.set(lock, i, *v);
            }
        }
    }

    fn stooge_sort(&self, start: usize, end: usize, lock: &SyncLock) {
        if end == start + 1 && self.cmp_two(lock, start, end).is_gt() {
            self.swap(lock, start, end);
        }

        if end > start + 1 {
            let third = (end - start + 1) / 3;
            self.stooge_sort(start, end - third, lock);
            self.stooge_sort(start + third, end, lock);
            self.stooge_sort(start, end - third, lock);
        }
    }

    fn slow_sort(&self, start: usize, end: usize, lock: &SyncLock) {
        if start < end {
            let m = (start + end) / 2;
            self.slow_sort(start, m, lock);
            self.slow_sort(m + 1, end, lock);

            if self.cmp_two(lock, m, end).is_gt() {
                self.swap(lock, m, end);
            }

            self.slow_sort(start, end - 1, lock);
        }
    }

    fn quick_sort(
        &self,
        start: usize,
        end: usize,
        rng: &mut rand::prelude::ThreadRng,
        lock: &SyncLock,
    ) {
        if end <= start {
            return;
        }

        let mut l = start;
        let mut r = end - 1;

        while l < r {
            while l < end && self.cmp_two(lock, l, end).is_lt() {
                l += 1;
            }

            while r > start && self.cmp_two(lock, r, end).is_gt() {
                r -= 1;
            }

            if l < r {
                self.swap(lock, l, r);
            }
        }

        if self.cmp_two(lock, l, end).is_gt() {
            self.swap(lock, l, end);
        }

        if l > start {
            self.quick_sort(start, l - 1, rng, lock);
        }
        if l < end {
            self.quick_sort(l + 1, end, rng, lock);
        }
    }

    fn merge_sort(&self, start: usize, end: usize, lock: &SyncLock) {
        if end == start + 1 && self.cmp_two(lock, start, end).is_gt() {
            self.swap(lock, start, end);
        } else if end > start + 1 {
            let m = (start + end) / 2;
            self.merge_sort(start, m, lock);
            self.merge_sort(m + 1, end, lock);

            let mut tmp = Vec::with_capacity(end - start + 1);
            let mut l = start;
            let mut r = m + 1;
            while tmp.len() < tmp.capacity() {
                if r > end || l <= m && self.cmp_two(lock, l, r).is_lt() {
                    tmp.push(self.get(lock, l));
                    l += 1;
                } else {
                    tmp.push(self.get(lock, r));
                    r += 1;
                }
            }

            for (index, val) in tmp.iter().enumerate() {
                self.set(lock, start + index, *val);
            }
        }
    }

    fn heap_sort(&self, max: usize, lock: &SyncLock) {
        for i in (0..=max / 2).rev() {
            self.heapify_down(i, max, lock);
        }
        for i in (1..=max).rev() {
            self.swap(lock, 0, i);

            self.heapify_down(0, i - 1, lock);
        }
    }

    fn heapify_down(&self, index: usize, max: usize, lock: &SyncLock) {
        if 2 * index + 1 <= max {
            let tmp_max = if 2 * index + 2 <= max
                && self.cmp_two(lock, 2 * index + 1, 2 * index + 2).is_lt()
            {
                2 * index + 2
            } else {
                2 * index + 1
            };

            if self.cmp_two(lock, index, tmp_max).is_lt() {
                self.swap(lock, index, tmp_max);

                self.heapify_down(tmp_max, max, lock);
            }
        }
    }

    fn counting_sort(
        &self,
        size: usize,
        buckets: usize,
        transform: impl Fn(usize) -> usize,
        lock: &SyncLock,
    ) {
        let mut keys = vec![0; buckets];
        let mut vals = Vec::with_capacity(size);

        for i in 0..size {
            vals.push(self.get(lock, i));
            keys[transform(*vals.last().unwrap() - 1)] += 1;
        }

        vals.reverse();

        for i in 1..buckets {
            keys[i] += keys[i - 1];
        }

        for v in vals {
            let key = transform(v - 1);
            keys[key] -= 1;
            self.set(lock, keys[key], v);
        }
    }

    fn radix_sort(&self, size: usize, base: usize, lock: &SyncLock) {
        let mut i = 1;

        while size / i > 0 {
            self.counting_sort(size, base, |x| (x / i) % base, lock);
            i *= base;
        }
    }

    pub fn calculate_ticks(&self, speed: u64) -> u64 {
        match self {
            Sort::StoogeSort | Sort::SlowSort => speed.pow(3),
            Sort::BubbleSort
            | Sort::ShakerSort
            | Sort::ExchangeSort
            | Sort::CycleSort
            | Sort::CombSort
            | Sort::OddEvenSort
            | Sort::InsertionSort
            | Sort::ShellSort
            | Sort::SelectionSort
            | Sort::DoubleSelectionSort
            | Sort::StrandSort => speed.pow(2),
            Sort::QuickSort | Sort::MergeSort | Sort::HeapSort => speed * speed.log2() as u64,
            Sort::CountingSort | Sort::RadixSort10 | Sort::RadixSort2 => speed,
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
                            lock.steps = cmp::max(1, self.calculate_ticks(lock.speed as u64));
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

    pub fn view(&self) -> gui::View {
        self.lock.lock().unwrap().array_state.get_view()
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
