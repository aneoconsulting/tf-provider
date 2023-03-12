use async_trait::async_trait;

use tf_provider::{AttributePath, Diagnostics, Schema};

pub(crate) trait WithSchema {
    fn schema() -> Schema;
}

#[async_trait]
pub(crate) trait WithValidate {
    async fn validate(&self, diags: &mut Diagnostics, attr_path: AttributePath);
}
