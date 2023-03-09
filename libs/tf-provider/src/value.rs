use std::{
    collections::{HashMap, HashSet},
    iter::FusedIterator,
    mem,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::utils::serde_unknown;

#[derive(
    Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Default, Serialize, Deserialize,
)]
#[serde(untagged)]
pub enum Value<T> {
    Value(T),
    #[default]
    Null,
    #[serde(with = "serde_unknown")]
    Unknown,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueAny {
    String(String),
    Number(i64),
    Bool(bool),
    List(Vec<ValueAny>),
    Map(HashMap<String, ValueAny>),
    #[default]
    Null,
    #[serde(with = "serde_unknown")]
    Unknown,
}

impl ValueAny {
    pub fn json(&self) -> String {
        serde_json::to_string(self).unwrap_or("<invalid>".into())
    }
    pub fn json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or("<invalid>".into())
    }
}

#[derive(
    Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Default, Serialize, Deserialize,
)]
pub struct StructEmpty {}

pub type ValueEmpty = Value<StructEmpty>;
pub type ValueString = Value<String>;
pub type ValueNumber = Value<i64>;
pub type ValueList<T> = Value<Vec<T>>;
pub type ValueSet<T> = Value<HashSet<T>>;
pub type ValueMap<T> = Value<HashMap<String, T>>;

impl<T> Value<T> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////
    #[inline]
    pub const fn is_value(&self) -> bool {
        match self {
            Value::Value(_) => true,
            _ => false,
        }
    }
    #[inline]
    pub const fn is_null(&self) -> bool {
        match self {
            Value::Null => true,
            _ => false,
        }
    }
    #[inline]
    pub const fn is_unknown(&self) -> bool {
        match self {
            Value::Unknown => true,
            _ => false,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Adapter for working with references
    /////////////////////////////////////////////////////////////////////////
    #[inline]
    pub const fn as_ref(&self) -> Value<&T> {
        match *self {
            Value::Value(ref x) => Value::Value(x),
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn as_mut(&mut self) -> Value<&mut T> {
        match *self {
            Value::Value(ref mut x) => Value::Value(x),
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Getting to contained values
    /////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn expect(self, msg: &str) -> T {
        match self {
            Value::Value(val) => val,
            Value::Null => panic!("{} (Null)", msg),
            Value::Unknown => panic!("{} (Unknown)", msg),
        }
    }
    #[inline]
    pub fn unwrap(self) -> T {
        match self {
            Value::Value(x) => x,
            Value::Null => panic!("called `Value::unwrap()` on a `Null` value"),
            Value::Unknown => panic!("called `Value::unwrap()` on an `Unknown` value"),
        }
    }
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Value::Value(x) => x,
            _ => default,
        }
    }
    #[inline]
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Value::Value(x) => x,
            _ => f(),
        }
    }
    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Value::Value(x) => x,
            _ => Default::default(),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Transforming contained values
    /////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn map<U, F>(self, f: F) -> Value<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Value::Value(x) => Value::Value(f(x)),
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Value::Value(ref x) = self {
            f(x);
        }
        self
    }
    #[inline]
    pub fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Value::Value(x) => f(x),
            _ => default,
        }
    }
    #[inline]
    pub fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        F: FnOnce(T) -> U,
        D: FnOnce() -> U,
    {
        match self {
            Value::Value(x) => f(x),
            _ => default(),
        }
    }
    #[inline]
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Value::Value(x) => Ok(x),
            _ => Err(err),
        }
    }
    #[inline]
    pub fn ok_or_else<E, F>(self, err: F) -> Result<T, E>
    where
        F: FnOnce() -> E,
    {
        match self {
            Value::Value(x) => Ok(x),
            _ => Err(err()),
        }
    }
    #[inline]
    pub fn as_deref(&self) -> Value<&T::Target>
    where
        T: Deref,
    {
        match self.as_ref() {
            Value::Value(x) => Value::Value(x.deref()),
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn as_deref_mut(&mut self) -> Value<&mut T::Target>
    where
        T: DerefMut,
    {
        match self.as_mut() {
            Value::Value(x) => Value::Value(x.deref_mut()),
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn into_option(self) -> Option<T> {
        match self {
            Value::Value(x) => Some(x),
            _ => None,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Iterator constructors
    /////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: Item { val: self.as_ref() },
        }
    }
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            inner: Item { val: self.as_mut() },
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Boolean operations on the values, eager and lazy
    /////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn and<U>(self, rhs: Value<U>) -> Value<U> {
        match self {
            Value::Value(_) => rhs,
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn and_then<U, F>(self, f: F) -> Value<U>
    where
        F: FnOnce(T) -> Value<U>,
    {
        match self {
            Value::Value(x) => f(x),
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: FnOnce(&T) -> bool,
    {
        match self {
            Value::Value(x) => {
                if predicate(&x) {
                    Value::Value(x)
                } else {
                    Value::Null
                }
            }
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn or(self, rhs: Self) -> Self {
        match self {
            Value::Value(x) => Value::Value(x),
            Value::Null => rhs,
            Value::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        match self {
            Value::Value(x) => Value::Value(x),
            Value::Null => f(),
            Value::Unknown => Value::Unknown,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Misc
    /////////////////////////////////////////////////////////////////////////
    pub fn take(&mut self) -> Self {
        mem::replace(self, Value::Null)
    }
    pub fn replace(&mut self, val: T) -> Self {
        mem::replace(self, Value::Value(val))
    }
    pub fn contains<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            Value::Value(y) => x.eq(y),
            _ => false,
        }
    }
}

impl<T> Value<&T> {
    pub fn copied(self) -> Value<T>
    where
        T: Copy,
    {
        match self {
            Value::Value(&x) => Value::Value(x),
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }
    pub fn cloned(self) -> Value<T>
    where
        T: Copy,
    {
        match self {
            Value::Value(x) => Value::Value(x.clone()),
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }
}
impl<T> Value<&mut T> {
    pub fn copied(self) -> Value<T>
    where
        T: Copy,
    {
        match self {
            Value::Value(&mut x) => Value::Value(x),
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }
    pub fn cloned(self) -> Value<T>
    where
        T: Copy,
    {
        match self {
            Value::Value(x) => Value::Value(x.clone()),
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }
}

impl<T> IntoIterator for Value<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            inner: Item { val: self },
        }
    }
}

impl<'a, T> IntoIterator for &'a Value<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Value<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T> From<T> for Value<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}
impl<'a, T> From<&'a Value<T>> for Value<&'a T> {
    #[inline]
    fn from(value: &'a Value<T>) -> Self {
        value.as_ref()
    }
}
impl<'a, T> From<&'a mut Value<T>> for Value<&'a mut T> {
    #[inline]
    fn from(value: &'a mut Value<T>) -> Self {
        value.as_mut()
    }
}
impl<T> From<Option<T>> for Value<T> {
    #[inline]
    fn from(value: Option<T>) -> Self {
        match value {
            Some(x) => Self::Value(x),
            None => Self::Null,
        }
    }
}

#[derive(Clone, Debug)]
struct Item<A> {
    val: Value<A>,
}

impl<A> Iterator for Item<A> {
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A> {
        self.val.take().into_option()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.val {
            Value::Value(_) => (1, Some(1)),
            _ => (0, Some(0)),
        }
    }
}

impl<A> DoubleEndedIterator for Item<A> {
    #[inline]
    fn next_back(&mut self) -> Option<A> {
        self.val.take().into_option()
    }
}

impl<A> ExactSizeIterator for Item<A> {}
impl<A> FusedIterator for Item<A> {}

#[derive(Debug)]
pub struct Iter<'a, A: 'a> {
    inner: Item<&'a A>,
}

impl<'a, A> Iterator for Iter<'a, A> {
    type Item = &'a A;

    #[inline]
    fn next(&mut self) -> Option<&'a A> {
        self.inner.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, A> DoubleEndedIterator for Iter<'a, A> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a A> {
        self.inner.next_back()
    }
}

impl<A> ExactSizeIterator for Iter<'_, A> {}

impl<A> FusedIterator for Iter<'_, A> {}

impl<A> Clone for Iter<'_, A> {
    #[inline]
    fn clone(&self) -> Self {
        Iter {
            inner: self.inner.clone(),
        }
    }
}

#[derive(Debug)]
pub struct IterMut<'a, A: 'a> {
    inner: Item<&'a mut A>,
}

impl<'a, A> Iterator for IterMut<'a, A> {
    type Item = &'a mut A;

    #[inline]
    fn next(&mut self) -> Option<&'a mut A> {
        self.inner.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, A> DoubleEndedIterator for IterMut<'a, A> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut A> {
        self.inner.next_back()
    }
}

impl<A> ExactSizeIterator for IterMut<'_, A> {}

impl<A> FusedIterator for IterMut<'_, A> {}

pub struct IntoIter<A> {
    inner: Item<A>,
}

impl<A> Iterator for IntoIter<A> {
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A> {
        self.inner.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<A> DoubleEndedIterator for IntoIter<A> {
    #[inline]
    fn next_back(&mut self) -> Option<A> {
        self.inner.next_back()
    }
}

impl<A> ExactSizeIterator for IntoIter<A> {}

impl<A> FusedIterator for IntoIter<A> {}
