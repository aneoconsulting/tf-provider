use tf_provider::{Diagnostics, Value};

use crate::{connection::Connection, utils::WithNormalize};

use super::state::ResourceState;

impl<'a, T: Connection> WithNormalize for ResourceState<'a, T> {
    fn normalize(&mut self, _diags: &mut Diagnostics) {
        if self.id.is_null() {
            self.id = Value::Unknown;
        }
        if self.inputs.is_null() {
            self.inputs = Value::Value(Default::default());
        }
        if self.state.is_unknown() {
            self.state = Value::Value(
                self.read
                    .iter()
                    .flatten()
                    .map(|(name, _)| (name.clone(), Value::Unknown))
                    .collect(),
            );
        }
    }
}
