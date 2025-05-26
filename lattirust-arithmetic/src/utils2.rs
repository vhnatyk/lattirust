use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use nvtx::{range_pop, range_push};

thread_local! {
    pub static NVTX_STACK: RefCell<Vec<NvtxGuard>> = RefCell::new(Vec::new());
}

static TIMING_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct NvtxGuard {
    description: String,
    start: Instant,
    popped: bool,
    order: usize,
}

impl NvtxGuard {
    pub fn new(description: &str, indent: usize) -> Self {
        let indented_desc = format!("{}{}", " ".repeat(indent), description);
        println!("Starting: {}", indented_desc);
        range_push!("{}", indented_desc.as_str());
        Self {
            description: indented_desc,
            start: Instant::now(),
            popped: false,
            order: TIMING_COUNTER.fetch_add(1, Ordering::SeqCst),
        }
    }

    pub fn pop(&mut self) {
        if !self.popped {
            range_pop!();
            let duration = self.start.elapsed();
            println!("End of    {}: time: {:?} (order: {})", self.description, duration, self.order);
            self.popped = true;
        }
    }
}

// Implement Drop to ensure pop is always called
impl Drop for NvtxGuard {
    fn drop(&mut self) {
        if !self.popped {
            range_pop!();
            let duration = self.start.elapsed();
            println!("End of    {}: time: {:?} (order: {})", self.description, duration, self.order);
        }
    }
}

#[macro_export]
macro_rules! nvtx_timed {
    ($desc:expr) => {{
        let indent = crate::utils2::NVTX_STACK.with(|stack| stack.borrow().len());
        let guard = crate::utils2::NvtxGuard::new($desc, indent);
        crate::utils2::NVTX_STACK.with(|stack| {
            stack.borrow_mut().push(guard);
        });
        // Return a dummy guard that does nothing when dropped
        // The real timing is handled by the guard in the stack
        struct DummyGuard;
        impl Drop for DummyGuard {
            fn drop(&mut self) {}
        }
        DummyGuard
    }};
}

#[macro_export]
macro_rules! nvtx_timed_pop {
    () => {
        crate::utils2::NVTX_STACK.with(|stack| {
            if let Some(mut guard) = stack.borrow_mut().pop() {
                guard.pop();
            }
        });
    };
} 