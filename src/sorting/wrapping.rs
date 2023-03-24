use std::{cmp, marker::PhantomPinned, pin::Pin, sync, thread, time};

use super::sort;
use crate::{
    array::{self, ArrayState},
    gui,
};

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
        let array_state = self.array_state.clone();
        let sort = self.sort;
        let size = self.operate_array(|array| array.size());

        self.handle = Some(SenderHandle {
            thread: thread::spawn(move || sort.sort(ArrayLock::new(array_state, receiver), size)),
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

    pub fn alive(&mut self) -> bool {
        if let Some(ref handle) = self.handle {
            if handle.thread.is_finished() {
                self.handle = None;
            }
        }

        self.handle.is_some()
    }

    fn check_alive(&mut self, msg: &str) -> &SenderHandle {
        assert!(self.alive(), "Sort is not running: {msg}");

        self.handle.as_ref().unwrap()
    }

    pub fn tick(&mut self, speed: f32) {
        let speed = (speed * self.sort.calculate_max_ticks(self.size() as u64) as f32) as u64;

        self.check_alive("Sorting Tick")
            .sender
            .send(Message::Tick(cmp::max(1, speed), time::Instant::now()))
            .unwrap();
    }

    pub fn step(&mut self) {
        self.check_alive("Sorting Step")
            .sender
            .send(Message::Step)
            .unwrap();
    }
}

macro_rules! wrap_sorter_array_ops {
    ($(fn $name:ident($($arg:ident: $typ:ty),*) -> $ret:ty;)+) => {
        $(pub fn $name(&self, $($arg: $typ),*) -> $ret {
            self.operate_array(|array| array.$name($($arg),*))
        })+
    };
}

impl Sorter {
    pub fn operate_array<T>(&self, f: impl FnOnce(&mut array::ArrayState) -> T) -> T {
        f(&mut self.array_state.lock().unwrap())
    }

    wrap_sorter_array_ops! {
        fn size() -> usize;
        fn clear_step() -> ();
        fn last_step() -> array::Step;
        fn shuffle() -> ();
        fn reverse() -> ();
        fn initialize(size: usize) -> ();
        fn array_view() -> array::ArrayView;
        fn comparisons() -> u64;
        fn accesses() -> u64;
        fn reset_stats() -> ();
        fn get_view() -> gui::View;
        fn set_view(view: gui::View) -> ();
    }
}

#[derive(Copy, Clone)]
enum Message {
    Kill,
    Step,
    Tick(u64, time::Instant),
}

pub struct ArrayLock {
    array_lock: Option<sync::MutexGuard<'static, ArrayState>>,
    array_state: sync::Arc<sync::Mutex<array::ArrayState>>,
    receiver: sync::mpsc::Receiver<Message>,
    counter: u64,
    instant: time::Instant,
    _pinned: PhantomPinned,
}

impl ArrayLock {
    fn new(array_state: SyncArray, receiver: sync::mpsc::Receiver<Message>) -> Pin<Box<ArrayLock>> {
        // safety: Since this returns an owning pointer with exclusive access to the lock it will not move.
        unsafe {
            Pin::new_unchecked(Box::new(ArrayLock {
                array_state,
                array_lock: None,
                receiver,
                counter: 0,
                instant: time::Instant::now(),
                _pinned: PhantomPinned,
            }))
        }
    }

    fn perform_step<F, T>(self: &mut Pin<Box<Self>>, step: F) -> ArrayResult<T>
    where
        F: FnOnce(&mut array::ArrayState) -> T,
    {
        // safety: The lock won't be moved, so this is safe.
        let this = unsafe { self.as_mut().get_unchecked_mut() };

        if this.counter == 0
            || this.counter % crate::TIME_OUT_CHECK == 0
                && this.instant.elapsed() > crate::DELAY_TIME
        {
            this.array_lock = None;

            match this.receiver.recv().unwrap_or(Message::Kill) {
                Message::Kill => return Err(()),
                Message::Step => this.counter = 1,
                Message::Tick(count, instant) => {
                    this.counter = count;
                    this.instant = instant;
                }
            }

            // This changes the mutex guards lifetime to be unbound.
            //
            // safety: This type can only be aquired in a Pin<Box<_>>, so the guard will not be invalidated.
            // the mutex will also never be moved stay alive until the guard is dropped.
            this.array_lock =
                unsafe { std::mem::transmute(Some(this.array_state.lock().unwrap())) };
        }

        this.counter -= 1;

        Ok(step(&mut *this.array_lock.as_deref_mut().unwrap()))
    }
}

macro_rules! wrap_array_op {
    ($name:ident, ($($arg:ident : $argtype:ty),*) -> $ret:ty) => {
        pub fn $name(self: &mut Pin<Box<Self>>, $($arg:$argtype),*) -> ArrayResult<$ret> {
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
