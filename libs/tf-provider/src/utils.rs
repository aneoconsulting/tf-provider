use crate::diagnostics::Diagnostics;

pub trait OptionFactor {
    type Output;
    fn factor(self) -> Self::Output;
}
pub trait OptionExpand {
    type Output;
    fn expand(self) -> Self::Output;
}

pub trait MapInto<U> {
    type Output;
    fn map_into(self) -> Self::Output;
}

impl<T, U> MapInto<U> for Option<T>
where
    T: Into<U>,
{
    type Output = Option<U>;
    fn map_into(self) -> Self::Output {
        if let Some(value) = self {
            Some(value.into())
        } else {
            None
        }
    }
}

pub trait NormalizeDiagnostics {
    fn normalize_diagnostics(self, diags: &mut Diagnostics) -> Self;
}

impl<T> NormalizeDiagnostics for Option<T> {
    fn normalize_diagnostics(self, diags: &mut Diagnostics) -> Self {
        if self.is_none() {
            diags.internal_error();
        }
        self
    }
}

macro_rules! impl_all {
    ($($e:ident)+) => {
        impl<$($e),+> OptionFactor for ($(Option<$e>,)+) {
            type Output = Option<($($e,)+)>;
            #[allow(non_snake_case)]
            fn factor(self) -> Self::Output {
                let ($($e,)+) = self;
                Some(($($e?,)+))
            }
        }
        impl<$($e),+> OptionExpand for Option<($($e,)+)> {
            type Output = ($(Option<$e>,)+);
            #[allow(non_snake_case)]
            fn expand(self) -> Self::Output {
                if let Some(($($e,)+)) = self {
                    ($(Some($e),)+)
                } else {
                    ($(Option::<$e>::None,)+)
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
