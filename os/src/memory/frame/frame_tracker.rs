use crate::memory::{address::*, FRAME_ALLOCATOR};

pub struct FrameTracker(pub(super) PhysicalPageNumber);

impl FrameTracker {
    pub fn address(&self) -> PhysicalAddress {
        self.0.into()
    }

    pub fn page_number(&self) -> PhysicalPageNumber {
        self.0
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(self);
    }
}