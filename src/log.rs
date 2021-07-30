use std::cell::UnsafeCell;
use std::fmt;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::mem::MaybeUninit;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::{Mutex, Once};

struct OnceCell<T> {
    once: Once,
    cell: MaybeUninit<UnsafeCell<T>>,
}

impl<T> OnceCell<T> {
    const fn new() -> OnceCell<T> {
        OnceCell {
            once: Once::new(),
            cell: MaybeUninit::uninit(),
        }
    }

    fn set(&self, value: T) {
        self.once.call_once(|| {
            let ptr = self.cell.as_ptr() as *mut T;
            unsafe { std::ptr::write(ptr, value) };
        })
    }

    fn get(&self) -> Option<&T> {
        if !self.once.is_completed() {
            return None;
        }

        Some(unsafe { &*(self.cell.as_ptr() as *const T) })
    }
}

unsafe impl<T: Send> Send for OnceCell<T> {}

unsafe impl<T: Send + Sync> Sync for OnceCell<T> {}

pub struct Logger {
    writer: BufWriter<File>,
}

static GLOBAL_LOGGER: OnceCell<Mutex<Logger>> = OnceCell::new();

impl Logger {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(Self { writer })
    }

    pub fn set_as_global(self) {
        GLOBAL_LOGGER.set(Mutex::new(self));
    }

    pub fn get_global() -> impl DerefMut<Target = Self> {
        let mutex = GLOBAL_LOGGER.get().expect("global logger not set");
        mutex.lock().unwrap()
    }

    pub fn log<S: AsRef<str>>(&mut self, message: S) -> Result<(), io::Error> {
        self.writer.write_all(message.as_ref().as_bytes())?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()
    }

    pub fn log_fmt(&mut self, args: fmt::Arguments) -> Result<(), io::Error> {
        self.writer.write_fmt(args)?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()
    }
}

#[macro_export]
macro_rules! log {
    () => {
        Logger::get_global().log("").expect("cannot log");
    };

    ($s:literal) => {
        Logger::get_global().log($s).expect("cannot log");
    };

    ($($args:tt)+) => {
        Logger::get_global().log_fmt(format_args!($($args)+)).expect("cannot log");
    };
}
