use std::borrow::Cow;

use tf_provider::{ValueMap, ValueString};

mod data_source;
mod normalize;
mod read;
mod resource;
mod state;
mod validate;

pub use data_source::CmdExecDataSource;
pub use resource::CmdExecResource;

fn prepare_envs<'a>(
    envs: &[(&'a ValueMap<'a, ValueString<'a>>, &'a str)],
) -> Vec<(Cow<'a, str>, Cow<'a, str>)> {
    envs.iter()
        .map(|(env, prefix)| {
            env.iter().flatten().filter_map(|(k, v)| {
                Some((
                    Cow::Owned(format!("{}{}", *prefix, k)),
                    Cow::Borrowed(v.as_deref_option()?),
                ))
            })
        })
        .flatten()
        .collect()
}

fn with_env<'a>(
    base_env: &'a Vec<(Cow<'a, str>, Cow<'a, str>)>,
    extra_env: &'a ValueMap<'a, ValueString<'a>>,
) -> impl Iterator<Item = (&'a Cow<'a, str>, &'a Cow<'a, str>)> {
    base_env.iter().map(|(k, v)| (k, v)).chain(
        extra_env
            .iter()
            .flatten()
            .filter_map(|(k, v)| Some((k, v.as_ref_option()?))),
    )
}
