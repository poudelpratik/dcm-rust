use crate::modules::util::thread_manager::ThreadManager;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator};
use rayon::prelude::*;

use rayon::{ThreadPool, ThreadPoolBuilder};

pub struct RayonThreadManager {
    thread_pool: Option<ThreadPool>,
}

impl RayonThreadManager {
    pub fn new() -> Self {
        Self { thread_pool: None }
    }

    pub fn set_max_threads(&mut self, num_threads: usize) {
        self.thread_pool = Some(
            ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .expect("Failed to create thread pool"),
        );
    }
}

impl<T: Sync> ThreadManager<T> for RayonThreadManager {
    fn process<F>(&self, data: &Vec<T>, operation: F)
    where
        F: Fn(&T) + Sync + Send,
    {
        if let Some(ref pool) = self.thread_pool {
            pool.install(|| {
                data.par_iter().for_each(operation);
            });
        } else {
            data.par_iter().for_each(operation);
        }
    }

    fn process_mut<F>(&self, data: &mut Vec<T>, operation: F)
    where
        T: Send,
        F: Fn(&mut T) + Sync + Send,
    {
        if let Some(ref pool) = self.thread_pool {
            pool.install(|| {
                data.par_iter_mut().for_each(operation);
            });
        } else {
            data.par_iter_mut().for_each(operation);
        }
    }
}
