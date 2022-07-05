use crate::sorting::wrapping;
use std::cmp;

type SortResult = Result<(), ()>;

macro_rules! declare_sorts {
    (|$lock:ident, $size:ident| {
        $($sort:ident: $func:expr => O($speed:expr))+
    }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Sort {
            $($sort),+
        }

        impl Sort {
            pub const VALUES: &'static[Sort] = &[$(Sort::$sort),+];

            pub fn sort(&self, mut $lock: wrapping::ArrayLock, $size: usize) -> SortResult {
                let $lock = &mut $lock;
                match self {
                    $(Sort::$sort => {$func}),+
                }
            }

            pub fn calculate_max_ticks(&self, $size: u64) -> u64 {
                match self {
                    $(Sort::$sort => {$speed}),+
                }
            }
        }
    };
}

declare_sorts! {
    |lock, size| {
        BubbleSort:
            Sort::bubble_sort(lock, size) => O(size.pow(2) / 100)
        ShakerSort:
            Sort::shaker_sort(lock, size) => O(size.pow(2) / 100)
        ExchangeSort:
            Sort::exchange_sort(lock, size) => O(size.pow(2) / 100)
        CycleSort:
            Sort::cycle_sort(lock, size) => O(size.pow(2) / 100)
        CombSort:
            Sort::comb_sort(lock, size) => O(size.pow(2) / 10000)
        OddEvenSort:
            Sort::odd_even_sort(lock, size) => O(size.pow(2) / 100)
        InsertionSort:
            Sort::insertion_sort(lock, size) => O(size.pow(2) / 100)
        ShellSort:
            Sort::shell_sort(lock, size) => O(size.pow(2) / 10000)
        SelectionSort:
            Sort::selection_sort(lock, size) => O(size.pow(2) / 100)
        DoubleSelectionSort:
            Sort::double_selection_sort(lock, size) => O(size.pow(2) / 100)
        StrandSort:
            Sort::strand_sort(lock, size) => O(size.pow(2) / 1000)
        StoogeSort:
            Sort::stooge_sort(lock, 0, size - 1) => O(size.pow(3) / 1000)
        SlowSort:
            Sort::slow_sort(lock, 0, size - 1)  => O(size.pow(3) / 1000)
        QuickSort:
            Sort::quick_sort(lock, 0, size - 1, &mut rand::thread_rng()) => O(size * size.log2() as u64 / 100)
        MergeSort:
            Sort::merge_sort(lock, 0, size - 1) => O(size * size.log2() as u64 / 100)
        HeapSort:
            Sort::heap_sort(lock, size - 1) => O(size * size.log2() as u64 / 100)
        CountingSort:
            Sort::counting_sort(lock, size, size, |x| x) => O(size / 50)
        RadixSort10:
            Sort::radix_sort(lock, size, 10) => O(size / 50)
        RadixSort2:
            Sort::radix_sort(lock, size, 2) => O(size / 50)
    }
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
    fn bubble_sort(lock: &mut wrapping::ArrayLock, size: usize) -> SortResult {
        for i in 1..size {
            let mut abort = true;
            for j in 0..size - i {
                if lock.cmp_two(j, j + 1)?.is_gt() {
                    lock.swap(j, j + 1)?;
                    abort = false;
                }
            }

            if abort {
                break;
            }
        }

        Ok(())
    }

    fn shaker_sort(lock: &mut wrapping::ArrayLock, size: usize) -> SortResult {
        for i in 1..size / 2 + 1 {
            let mut abort = true;
            for j in i - 1..size - i {
                if lock.cmp_two(j, j + 1)?.is_gt() {
                    lock.swap(j, j + 1)?;
                    abort = false;
                }
            }

            if abort {
                break;
            }

            abort = true;
            for j in (i..size - i).rev() {
                if lock.cmp_two(j, j - 1)?.is_lt() {
                    lock.swap(j, j - 1)?;
                    abort = false;
                }
            }

            if abort {
                break;
            }
        }

        Ok(())
    }

    fn exchange_sort(lock: &mut wrapping::ArrayLock, size: usize) -> SortResult {
        for i in 0..size - 1 {
            for j in i + 1..size {
                if lock.cmp_two(i, j)?.is_gt() {
                    lock.swap(i, j)?;
                }
            }
        }

        Ok(())
    }

    fn cycle_sort(lock: &mut wrapping::ArrayLock, size: usize) -> SortResult {
        let mut buf = vec![false; size];
        for i in 0..size - 1 {
            if buf[i] {
                continue;
            }

            let mut current = lock.get(i)?;
            loop {
                let mut index = 0;
                for j in 0..size {
                    if j != i && lock.cmp(j, current)?.is_lt() {
                        index += 1;
                    }
                }

                if index != i {
                    buf[index] = true;

                    let new = lock.get(index)?;
                    lock.set(index, current)?;
                    current = new;
                } else {
                    lock.set(i, current)?;

                    break;
                }
            }
        }

        Ok(())
    }

    fn comb_sort(lock: &mut wrapping::ArrayLock, size: usize) -> SortResult {
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
                if lock.cmp_two(i, i + gap)?.is_gt() {
                    lock.swap(i, i + gap)?;
                    sorted = false;
                }
            }
        }

        Ok(())
    }

    fn odd_even_sort(lock: &mut wrapping::ArrayLock, size: usize) -> SortResult {
        let mut sorted = false;

        while !sorted {
            sorted = true;

            for start in 0..=1 {
                for i in (start..size - 1).step_by(2) {
                    if lock.cmp_two(i, i + 1)?.is_gt() {
                        lock.swap(i, i + 1)?;
                        sorted = false;
                    }
                }
            }
        }

        Ok(())
    }

    fn insertion_sort(lock: &mut wrapping::ArrayLock, size: usize) -> SortResult {
        for i in 1..size {
            let current = lock.get(i)?;

            let mut j = i;
            while j > 0 && lock.cmp(j - 1, current)?.is_gt() {
                let x = lock.get(j - 1)?;
                lock.set(j, x)?;
                j -= 1;
            }

            lock.set(j, current)?;
        }

        Ok(())
    }

    fn shell_sort(lock: &mut wrapping::ArrayLock, size: usize) -> SortResult {
        let mut gap = size;

        while gap > 1 {
            gap = cmp::max(1, gap / 2);

            for i in gap..size {
                let tmp = lock.get(i)?;

                let mut j = i;

                while j >= gap && lock.cmp(j - gap, tmp)?.is_gt() {
                    let x = lock.get(j - gap)?;
                    lock.set(j, x)?;
                    j -= gap;
                }

                lock.set(j, tmp)?;
            }
        }

        Ok(())
    }

    fn selection_sort(lock: &mut wrapping::ArrayLock, size: usize) -> SortResult {
        for i in 0..size - 1 {
            let mut min = i;
            for j in i + 1..size {
                if lock.cmp_two(min, j)?.is_gt() {
                    min = j;
                }
            }
            if min != i {
                lock.swap(min, i)?;
            }
        }

        Ok(())
    }

    fn double_selection_sort(lock: &mut wrapping::ArrayLock, size: usize) -> SortResult {
        for i in 0..size / 2 {
            let mut min = i;
            let mut max = size - i - 1;

            for j in i + 1..size - i {
                if lock.cmp_two(min, j)?.is_gt() {
                    min = j;
                }
            }
            if min != i {
                lock.swap(min, i)?;
            }

            for j in (i + 1..size - i - 1).rev() {
                if lock.cmp_two(max, j)?.is_lt() {
                    max = j;
                }
            }
            if max != size - i - 1 {
                lock.swap(max, size - i - 1)?;
            }
        }

        Ok(())
    }

    fn strand_sort(lock: &mut wrapping::ArrayLock, size: usize) -> SortResult {
        let mut index = 0;
        while index < size {
            let mut len = 1;

            for j in index + 1..size {
                if lock.cmp_two(index + len - 1, j)?.is_lt() && j != index + len {
                    lock.swap(j, index + len)?;
                    len += 1;
                }
            }

            let old_index = index;
            index += len;

            let mut tmp = Vec::with_capacity(index);

            let mut x = 0;
            let mut y = 0;
            for _ in 0..index {
                if x >= old_index || y < len && lock.cmp_two(old_index + y, x)?.is_lt() {
                    tmp.push(lock.get(old_index + y)?);
                    y += 1;
                } else {
                    tmp.push(lock.get(x)?);
                    x += 1;
                }
            }

            for (i, v) in tmp.iter().enumerate() {
                lock.set(i, *v)?;
            }
        }

        Ok(())
    }

    fn stooge_sort(lock: &mut wrapping::ArrayLock, start: usize, end: usize) -> SortResult {
        if end == start + 1 && lock.cmp_two(start, end)?.is_gt() {
            lock.swap(start, end)?;
        }

        if end > start + 1 {
            let third = (end - start + 1) / 3;
            Sort::stooge_sort(lock, start, end - third)?;
            Sort::stooge_sort(lock, start + third, end)?;
            Sort::stooge_sort(lock, start, end - third)?;
        }

        Ok(())
    }

    fn slow_sort(lock: &mut wrapping::ArrayLock, start: usize, end: usize) -> SortResult {
        if start < end {
            let m = (start + end) / 2;
            Sort::slow_sort(lock, start, m)?;
            Sort::slow_sort(lock, m + 1, end)?;

            if lock.cmp_two(m, end)?.is_gt() {
                lock.swap(m, end)?;
            }

            Sort::slow_sort(lock, start, end - 1)?;
        }

        Ok(())
    }

    fn quick_sort(
        lock: &mut wrapping::ArrayLock,
        start: usize,
        end: usize,
        rng: &mut rand::prelude::ThreadRng,
    ) -> SortResult {
        if end <= start {
            return Ok(());
        }

        let mut l = start;
        let mut r = end - 1;

        while l < r {
            while l < end && lock.cmp_two(l, end)?.is_lt() {
                l += 1;
            }

            while r > start && lock.cmp_two(r, end)?.is_gt() {
                r -= 1;
            }

            if l < r {
                lock.swap(l, r)?;
            }
        }

        if lock.cmp_two(l, end)?.is_gt() {
            lock.swap(l, end)?;
        }

        if l > start {
            Sort::quick_sort(lock, start, l - 1, rng)?;
        }
        if l < end {
            Sort::quick_sort(lock, l + 1, end, rng)?;
        }

        Ok(())
    }

    fn merge_sort(lock: &mut wrapping::ArrayLock, start: usize, end: usize) -> SortResult {
        if end == start + 1 && lock.cmp_two(start, end)?.is_gt() {
            lock.swap(start, end)?;
        } else if end > start + 1 {
            let m = (start + end) / 2;
            Sort::merge_sort(lock, start, m)?;
            Sort::merge_sort(lock, m + 1, end)?;

            let mut tmp = Vec::with_capacity(end - start + 1);
            let mut l = start;
            let mut r = m + 1;
            while tmp.len() < tmp.capacity() {
                if r > end || l <= m && lock.cmp_two(l, r)?.is_lt() {
                    tmp.push(lock.get(l)?);
                    l += 1;
                } else {
                    tmp.push(lock.get(r)?);
                    r += 1;
                }
            }

            for (index, val) in tmp.iter().enumerate() {
                lock.set(start + index, *val)?;
            }
        }

        Ok(())
    }

    fn heap_sort(lock: &mut wrapping::ArrayLock, max: usize) -> SortResult {
        for i in (0..=max / 2).rev() {
            Sort::heapify_down(lock, i, max)?;
        }
        for i in (1..=max).rev() {
            lock.swap(0, i)?;

            Sort::heapify_down(lock, 0, i - 1)?;
        }

        Ok(())
    }

    fn heapify_down(lock: &mut wrapping::ArrayLock, index: usize, max: usize) -> SortResult {
        if 2 * index + 1 <= max {
            let tmp_max =
                if 2 * index + 2 <= max && lock.cmp_two(2 * index + 1, 2 * index + 2)?.is_lt() {
                    2 * index + 2
                } else {
                    2 * index + 1
                };

            if lock.cmp_two(index, tmp_max)?.is_lt() {
                lock.swap(index, tmp_max)?;

                Sort::heapify_down(lock, tmp_max, max)?;
            }
        }

        Ok(())
    }

    fn counting_sort(
        lock: &mut wrapping::ArrayLock,
        size: usize,
        buckets: usize,
        transform: impl Fn(usize) -> usize,
    ) -> SortResult {
        let mut keys = vec![0; buckets];
        let mut vals = Vec::with_capacity(size);

        for i in 0..size {
            vals.push(lock.get(i)?);
            keys[transform(*vals.last().unwrap() - 1)] += 1;
        }

        vals.reverse();

        for i in 1..buckets {
            keys[i] += keys[i - 1];
        }

        for v in vals {
            let key = transform(v - 1);
            keys[key] -= 1;
            lock.set(keys[key], v)?;
        }

        Ok(())
    }

    fn radix_sort(lock: &mut wrapping::ArrayLock, size: usize, base: usize) -> SortResult {
        let mut i = 1;

        while size / i > 0 {
            Sort::counting_sort(lock, size, base, |x| (x / i) % base)?;
            i *= base;
        }

        Ok(())
    }
}
