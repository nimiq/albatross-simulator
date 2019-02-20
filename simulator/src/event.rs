use std::cmp::Ordering;
use std::fmt;
use std::ops::Deref;

use crate::time::Time;
use crate::unique_id::UniqueId;

/// An event is a message that can be sent over a channel.
/// It also carries time information.
pub struct Event<E> {
    inner: E,
    time: Time,
    from: UniqueId,
    pub(crate) to: UniqueId,
}

impl<E> Event<E> {
    pub(crate) fn new(inner: E, time: Time, from: UniqueId, to: UniqueId) -> Self {
        Event {
            inner,
            time,
            from,
            to,
        }
    }

    /// Returns a reference on the inner type.
    pub fn inner(&self) -> &E {
        &self.inner
    }

    /// Returns the time this event was received.
    pub fn receive_time(&self) -> Time {
        self.time
    }

    /// Returns the sender of the event.
    pub fn from(&self) -> UniqueId {
        self.from
    }
}

impl<E> Deref for Event<E> {
    type Target = E;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.inner
    }
}

impl<E: Clone> Clone for Event<E> {
    fn clone(&self) -> Self {
        Event {
            inner: self.inner.clone(),
            time: self.time,
            from: self.from,
            to: self.to,
        }
    }
}

impl<E> PartialEq for Event<E> {
    fn eq(&self, other: &Event<E>) -> bool {
        other.time.eq(&self.time)
    }
}

impl<E> Eq for Event<E> {}

impl<E> PartialOrd for Event<E> {
    fn partial_cmp(&self, other: &Event<E>) -> Option<Ordering> {
        other.time.partial_cmp(&self.time)
    }
}

impl<E> Ord for Event<E> {
    /// Orders by time in reverse!
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time)
    }
}

impl<E: fmt::Debug> fmt::Debug for Event<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}: {} -> {}", self.inner, self.from, self.to)
    }
}
