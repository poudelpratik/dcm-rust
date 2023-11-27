use crate::modules::cfd_analyzer::CodeFragmentDescription;

pub trait Visit {
    fn visit_function(&mut self, cfd_item: &CodeFragmentDescription);
    fn visit_impl(&mut self, cfd_item: &CodeFragmentDescription);
}
