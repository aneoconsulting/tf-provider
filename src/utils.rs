use serde::{Deserialize, Serialize};

use crate::diagnostics::Diagnostics;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename = "_ExtStruct")]
pub(crate) struct ExtStruct(pub (i8, serde_bytes::ByteBuf));

pub(crate) mod serde_unknown {
    use super::ExtStruct;
    use serde::{Deserialize, Serialize};

    pub fn serialize<S>(serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ExtStruct((0, serde_bytes::ByteBuf::from(vec![]))).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        ExtStruct::deserialize(deserializer).and(Ok(()))
    }
}

pub trait OptionFactor {
    type Output;
    fn factor(self) -> Self::Output;
}
pub trait OptionExpand {
    type Output;
    fn expand(self) -> Self::Output;
}

pub trait CollectDiagnostics {
    type Output;
    fn collect_diagnostics(self, diags: &mut Diagnostics) -> Self::Output;
}

impl<T> CollectDiagnostics for Option<T> {
    type Output = Self;
    fn collect_diagnostics(self, diags: &mut Diagnostics) -> Self::Output {
        if self.is_none() && diags.errors.is_empty() {
            diags.root_error_short("Internal error");
        }
        self
    }
}

impl<T, E> CollectDiagnostics for Result<T, E>
where
    E: ToString,
{
    type Output = Option<T>;
    fn collect_diagnostics(self, diags: &mut Diagnostics) -> Self::Output {
        match self {
            Ok(value) => Some(value),
            Err(err) => {
                diags.root_error("Internal error", err.to_string());
                None
            }
        }
    }
}

macro_rules! count{
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*))
}
macro_rules! impl_all {
    ($($e:ident)+) => {impl_all!(count!($($e)+); $($e)+);};
    ($n:expr; $($e:ident)+) => {

        impl<T> OptionFactor for [Option<T>; $n] {
            type Output = Option<[T; $n]>;
            #[allow(non_snake_case)]
            fn factor(self) -> Self::Output {
                let [$($e,)+] = self;
                Some([$($e?,)+])
            }
        }
        impl<$($e),+> OptionFactor for ($(Option<$e>,)+) {
            type Output = Option<($($e,)+)>;
            #[allow(non_snake_case)]
            fn factor(self) -> Self::Output {
                let ($($e,)+) = self;
                Some(($($e?,)+))
            }
        }
        impl<T> OptionExpand for Option<[T; $n]> {
            type Output = [Option<T>; $n];
            #[allow(non_snake_case)]
            fn expand(self) -> Self::Output {
                if let Some([$($e,)+]) = self {
                    [$(Some($e),)+]
                } else {
                    Default::default()
                }
            }
        }
        impl<$($e),+> OptionExpand for Option<($($e,)+)> {
            type Output = ($(Option<$e>,)+);
            #[allow(non_snake_case)]
            fn expand(self) -> Self::Output {
                if let Some(($($e,)+)) = self {
                    ($(Some($e),)+)
                } else {
                    Default::default()
                }
            }
        }
    };
}

impl_all!(A);
impl_all!(A B);
impl_all!(A B C);
impl_all!(A B C D);
impl_all!(A B C D E);
impl_all!(A B C D E F);
impl_all!(A B C D E F G);
impl_all!(A B C D E F G H);
impl_all!(A B C D E F G H I);
impl_all!(A B C D E F G H I J);
impl_all!(A B C D E F G H I J K);
impl_all!(A B C D E F G H I J K L);
