pub mod rayon;

pub trait ThreadManager<T: Sync> {
    fn process<F>(&self, data: &Vec<T>, operation: F)
    where
        F: Fn(&T) + Sync + Send;

    fn process_mut<F>(&self, data: &mut Vec<T>, operation: F)
    where
        T: Send,
        F: Fn(&mut T) + Sync + Send;
}
