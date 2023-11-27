use crate::client_registry::client_event_listener::UpdateFragmentData;
use crate::fragment_registry::fragment::Fragment;
use crate::util::error::ApplicationError;
use serde_derive::Serialize;

pub mod fragment;

#[derive(Debug, Default, Clone, Serialize)]
pub(crate) struct FragmentRegistry {
    pub(crate) fragments: Vec<Fragment>,
}

impl FragmentRegistry {
    pub fn new(fragments: Vec<Fragment>) -> Self {
        Self { fragments }
    }

    pub fn update_fragments(
        &mut self,
        update_fragments_data: &Vec<UpdateFragmentData>,
    ) -> Result<(), ApplicationError> {
        for update_fragment_data in update_fragments_data {
            if let Some(index) = self
                .fragments
                .iter()
                .position(|f| f.id == update_fragment_data.fragment_id)
            {
                self.fragments[index].execution_location =
                    update_fragment_data.execution_location.clone();
            }
        }
        Ok(())
    }
}
