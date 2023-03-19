use serde::{Deserialize, Serialize};
use tf_provider::{Diagnostics, Value};

use crate::{connection::Connection, utils::WithNormalize};

use super::state::State;

impl<'a, T> WithNormalize for State<'a, T>
where
    T: Connection + Serialize + for<'b> Deserialize<'b>,
{
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
