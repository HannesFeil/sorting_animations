use std::{cmp, sync, thread};

use super::sort;
use crate::{array, gui};

pub type ArrayResult<T> = Result<T, ()>;
type SyncArray = sync::Arc<sync::Mutex<array::ArrayState>>;

struct SenderHandle {
    thread: thread::JoinHandle<ArrayResult<()>>,
    sender: sync::mpsc::Sender<Message>,
}

pub struct Sorter {
    sort: sort::Sort,
    array_state: SyncArray,
    handle: Option<SenderHandle>,
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
        let sort = self.sort;
        let size = self.operate_array(|array| array.size());

        self.handle = Some(SenderHandle {
            thread: thread::spawn(move || sort.sort(array_lock, size)),
            sender,
        });
    }

    pub fn kill_sort(&mut self) {
        if self.alive() {
            let handle = std::mem::replace(&mut self.handle, None).unwrap();

            handle.sender.send(Message::Kill).unwrap();
            handle.thread.join().unwrap().unwrap_or_default();
        }
    }

    pub fn set_sort(&mut self, sort: sort::Sort) {
        assert!(!self.alive(), "Sort still running, cannot change");

        self.sort = sort;
    }

    pub fn sort(&self) -> sort::Sort {
        self.sort
    }

    pub fn operate_array<T>(&self, f: impl FnOnce(&mut array::ArrayState) -> T) -> T {
        f(&mut self.array_state.lock().unwrap())
    }

    pub fn alive(&mut self) -> bool {
        if let Some(ref handle) = self.handle {
            if handle.thread.is_finished() {
                self.handle = None;
            }
        }

        self.handle.is_some()
    }

    fn check_alive(&mut self, msg: &str) -> &SenderHandle {
        assert!(self.alive(), "Sort is not running: {}", msg);

        self.handle.as_ref().unwrap()
    }

    pub fn tick(&mut self, speed: f32) {
        let speed = (speed * self.sort.calculate_max_ticks(self.array_size()) as f32) as u64;

        self.check_alive("Sorting Tick")
            .sender
            .send(Message::Tick(cmp::min(
                crate::MAX_STEPS,
                cmp::max(1, speed),
            )))
            .unwrap();
    }

    fn array_size(&self) -> u64 {
        self.operate_array(|array| array.size()) as u64
    }

    pub fn step(&mut self) {
        self.check_alive("Sorting Step")
            .sender
            .send(Message::Step)
            .unwrap();
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
