use core::borrow::{Borrow, BorrowMut};

use crate::{read::ReadState, write::WriteState};

#[derive(Debug)]
pub struct ReadWriteState<'buf> {
    read: ReadState<'buf>,
    write: WriteState<'buf>,
}

impl<'buf> ReadWriteState<'buf> {
    pub const fn new(read: ReadState<'buf>, write: WriteState<'buf>) -> Self {
        Self { read, write }
    }
}

impl<'buf> Borrow<ReadState<'buf>> for ReadWriteState<'buf> {
    fn borrow(&self) -> &ReadState<'buf> {
        &self.read
    }
}

impl<'buf> BorrowMut<ReadState<'buf>> for ReadWriteState<'buf> {
    fn borrow_mut(&mut self) -> &mut ReadState<'buf> {
        &mut self.read
    }
}

impl<'buf> Borrow<WriteState<'buf>> for ReadWriteState<'buf> {
    fn borrow(&self) -> &WriteState<'buf> {
        &self.write
    }
}

impl<'buf> BorrowMut<WriteState<'buf>> for ReadWriteState<'buf> {
    fn borrow_mut(&mut self) -> &mut WriteState<'buf> {
        &mut self.write
    }
}
