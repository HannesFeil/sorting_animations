use std::{cmp, sync, thread};

use super::sort;
use crate::{array, gui};

pub type ArrayResult<T> = Result<T, ()>;
type SyncArray = sync::Arc<sync::Mutex<array::ArrayState>>;

pub struct Sorter {
    sort: sort::Sort,
    array_state: SyncArray,
    handle: Option<(
        thread::JoinHandle<ArrayResult<()>>,
        sync::mpsc::Sender<Message>,
    )>,
}

impl Sorter {
    pub fn new(array_state: array::ArrayState) -> Sorter {
        Sorter {
            sort: sort::Sort::default(),
            array_state: sync::Arc::new(sync::Mutex::new(array_state)),
            handle: None,
        }
    }

    pub fn start_sort(&mut self) {
        assert!(!self.alive(), "Sort already running");

        let (sender, receiver) = sync::mpsc::channel();
        let array_lock = ArrayLock::new(self.array_state.clone(), receiver);
        let sort = self.sort.clone();
        let size = self.operate_array(|array| array.size());

        self.handle = Some((thread::spawn(move || sort.sort(array_lock, size)), sender));
    }

    pub fn kill_sort(&mut self) {
        if self.alive() {
            let (handle, sender) = std::mem::replace(&mut self.handle, None).unwrap();

            sender.send(Message::Kill).unwrap();
            handle.join().unwrap().unwrap_or_default();
        }
    }

    pub fn set_sort(&mut self, sort: sort::Sort) {
        assert!(!self.alive(), "Sort still running, cannot change");

        self.sort = sort;
    }

    pub fn sort(&self) -> sort::Sort {
        self.sort.clone()
    }

    pub fn operate_array<T>(&self, f: impl FnOnce(&mut array::ArrayState) -> T) -> T {
        f(&mut self.array_state.lock().unwrap())
    }

    pub fn alive(&mut self) -> bool {
        if let Some((ref handle, _)) = self.handle {
            if handle.is_finished() {
                self.handle = None;
            }
        }

        self.handle.is_some()
    }

    pub fn tick(&mut self, speed: f32) {
        assert!(self.alive(), "Sort is not alive, cannot tick");

        self.handle
            .as_ref()
            .unwrap()
            .1
            .send(Message::Tick(cmp::min(
                cmp::max(
                    1,
                    (speed
                        * self
                            .sort
                            .calculate_max_ticks(self.operate_array(|array| array.size()) as u64)
                            as f32) as u64,
                ),
                crate::MAX_STEPS,
            )))
            .unwrap();
    }

    pub fn step(&mut self) {
        assert!(self.alive(), "Sort is not alive, cannot step");

        self.handle.as_ref().unwrap().1.send(Message::Step).unwrap();
    }

    pub fn comparisons(&self) -> u64 {
        self.operate_array(|array| array.comparisons())
    }

    pub fn accesses(&self) -> u64 {
        self.operate_array(|array| array.accesses())
    }

    pub fn reset_stats(&self) {
        self.operate_array(|array| array.reset_stats())
    }

    pub fn view(&self) -> gui::View {
        self.operate_array(|array| array.get_view())
    }

    pub fn set_view(&self, view: gui::View) {
        self.operate_array(|array| array.set_view(view));
    }
}

#[derive(Copy, Clone)]
enum Message {
    Kill,
    Step,
    Tick(u64),
}

pub struct ArrayLock {
    array_state: sync::Arc<sync::Mutex<array::ArrayState>>,
    receiver: sync::mpsc::Receiver<Message>,
    counter: u64,
}

impl ArrayLock {
    fn new(array_state: SyncArray, receiver: sync::mpsc::Receiver<Message>) -> ArrayLock {
        ArrayLock {
            array_state,
            receiver,
            counter: 0,
        }
    }

    fn perform_step<F, T>(&mut self, step: F) -> ArrayResult<T>
    where
        F: FnOnce(&mut array::ArrayState) -> T,
    {
        if self.counter == 0 {
            match self.receiver.recv().unwrap_or(Message::Kill) {
                Message::Kill => return Err(()),
                Message::Step => self.counter = 1,
                Message::Tick(count) => self.counter = count,
            }
        }

        let mut array = self.array_state.lock().unwrap();
        self.counter -= 1;

        Ok(step(&mut array))
    }
}

macro_rules! wrap_array_op {
    ($name:ident, ($($arg:ident : $argtype:ty),*) -> $ret:ty) => {
        pub fn $name(&mut self, $($arg:$argtype),*) -> ArrayResult<$ret> {
            self.perform_step(|array_state_argument| {
                array_state_argument.$name($($arg),*)
            })
        }
    }
}

impl ArrayLock {
    wrap_array_op!(cmp_two, (a:usize, b:usize) -> cmp::Ordering);
    wrap_array_op!(swap, (a:usize, b:usize) -> ());
    wrap_array_op!(cmp, (index:usize, value:usize) -> cmp::Ordering);
    wrap_array_op!(get, (index:usize) -> usize);
    wrap_array_op!(set, (index:usize, value:usize) -> ());
}
