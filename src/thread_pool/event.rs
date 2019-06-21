use std::boxed::Box;
use std::fmt::{Debug, Error as FmtError, Formatter};
use std::marker::{Send, Sync};
use std::ops::FnOnce;

pub(crate) enum Event {
    Stop,
    Task(Box<dyn FnOnce() + 'static + Sync + Send>),
    Debug,
}

impl Event {
    pub(crate) fn with_task<T>(func: T) -> Self
    where
        T: FnOnce() + 'static + Sync + Send,
    {
        Event::Task(Box::new(func))
    }

    pub(crate) fn stop() -> Self {
        Event::Stop
    }

    pub(crate) fn debug() -> Self {
        Event::Debug
    }
}

impl Debug for Event {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Event::Stop => write!(f, "event stop"),
            Event::Task(_) => write!(f, "event task"),
            Event::Debug => write!(f, "event debug"),
        }
    }
}
