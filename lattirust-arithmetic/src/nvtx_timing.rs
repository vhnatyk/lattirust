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
    pub fn new(description: String) -> Self {
        let order = TIMING_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("Starting: {}", description);
        range_push!("{}", description.as_str());
        Self {
            description,
            start: Instant::now(),
            popped: false,
            order,
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
    ($description:expr) => {{
        let guard = $crate::nvtx_timing::NvtxGuard::new($description.to_string());
        $crate::nvtx_timing::NVTX_STACK.with(|stack| {
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
    () => {{
        $crate::nvtx_timing::NVTX_STACK.with(|stack| {
            if let Some(mut guard) = stack.borrow_mut().pop() {
                guard.pop();
                // Print NTT timing statistics only if there were NTT/INTT operations
                if $crate::ring::ntt::NTT_CALLS.load(std::sync::atomic::Ordering::Relaxed) > 0 || 
                   $crate::ring::ntt::INTT_CALLS.load(std::sync::atomic::Ordering::Relaxed) > 0 {
                    $crate::ring::ntt::print_ntt_times();
                }
            }
        });
    }};
} 