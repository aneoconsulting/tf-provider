// This file is part of the tf-provider project
//
// Copyright (C) ANEO, 2024-2024. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License")
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fmt::{Debug, Display},
    iter::FusedIterator,
    mem,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::utils::serde_unknown;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value<T> {
    Value(T),
    #[default]
    Null,
    #[serde(with = "serde_unknown")]
    Unknown,
}

#[derive(Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueAny {
    String(String),
    Number(i64),
    Bool(bool),
    List(Vec<ValueAny>),
    Map(BTreeMap<String, ValueAny>),
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
pub type ValueString<'a> = Value<Cow<'a, str>>;
pub type ValueNumber = Value<i64>;
pub type ValueBool = Value<bool>;
pub type ValueList<T> = Value<Vec<T>>;
pub type ValueSet<T> = Value<BTreeSet<T>>;
pub type ValueMap<'a, T> = Value<BTreeMap<Cow<'a, str>, T>>;

pub mod serde_as_vec {
    use anyhow::anyhow;
    use serde::{de::Error, ser::SerializeSeq, Deserialize, Serialize};

    use super::Value;

    /// Serialize a nullable Value into a Vec of Values with 0 or 1 element
    pub fn serialize<T, S>(value: &Value<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: Serialize,
    {
        let mut seq = serializer.serialize_seq(Some(value.is_value() as usize))?;
        if let Value::Value(value) = value {
            seq.serialize_element(value)?;
        }
        seq.end()
    }

    /// Deserialize a Vec of values into a single, nullable, Value
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Value<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let vec: Vec<Value<T>> = Deserialize::deserialize(deserializer)?;
        let mut iter = vec.into_iter();
        if let Some(value) = iter.next() {
            if iter.next().is_none() {
                Ok(value)
            } else {
                Err(anyhow!("Try to store multiple elements in a single Value"))
                    .map_err(D::Error::custom)
            }
        } else {
            Ok(Value::Null)
        }
    }
}

impl<T> Value<T> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////
    #[inline]
    pub const fn is_value(&self) -> bool {
        matches!(self, Self::Value(_))
    }
    #[inline]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
    #[inline]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    /////////////////////////////////////////////////////////////////////////
    // Adapter for working with references
    /////////////////////////////////////////////////////////////////////////
    #[inline]
    pub const fn as_ref(&self) -> Value<&T> {
        match *self {
            Self::Value(ref x) => Value::Value(x),
            Self::Null => Value::Null,
            Self::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn as_mut(&mut self) -> Value<&mut T> {
        match *self {
            Self::Value(ref mut x) => Value::Value(x),
            Self::Null => Value::Null,
            Self::Unknown => Value::Unknown,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Getting to contained values
    /////////////////////////////////////////////////////////////////////////
    #[inline]
    pub fn expect(self, msg: &str) -> T {
        match self {
            Self::Value(val) => val,
            Self::Null => panic!("{} (Null)", msg),
            Self::Unknown => panic!("{} (Unknown)", msg),
        }
    }
    #[inline]
    pub fn unwrap(self) -> T {
        match self {
            Self::Value(x) => x,
            Self::Null => panic!("called `Value::unwrap()` on a `Null` value"),
            Self::Unknown => panic!("called `Value::unwrap()` on an `Unknown` value"),
        }
    }
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Value(x) => x,
            _ => default,
        }
    }
    #[inline]
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Self::Value(x) => x,
            _ => f(),
        }
    }
    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Self::Value(x) => x,
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
            Self::Value(x) => Value::Value(f(x)),
            Self::Null => Value::Null,
            Self::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Self::Value(ref x) = self {
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
            Self::Value(x) => f(x),
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
            Self::Value(x) => f(x),
            _ => default(),
        }
    }
    #[inline]
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Self::Value(x) => Ok(x),
            _ => Err(err),
        }
    }
    #[inline]
    pub fn ok_or_else<E, F>(self, err: F) -> Result<T, E>
    where
        F: FnOnce() -> E,
    {
        match self {
            Self::Value(x) => Ok(x),
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
    pub fn as_option(self) -> Option<T> {
        match self {
            Self::Value(x) => Some(x),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ref_option(&self) -> Option<&T> {
        match self {
            Self::Value(x) => Some(x),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_option(&mut self) -> Option<&mut T> {
        match self {
            Self::Value(x) => Some(x),
            _ => None,
        }
    }
    #[inline]
    pub fn as_deref_option(&self) -> Option<&T::Target>
    where
        T: Deref,
    {
        match self {
            Self::Value(x) => Some(x),
            _ => None,
        }
    }
    #[inline]
    pub fn as_deref_mut_option(&mut self) -> Option<&mut T::Target>
    where
        T: DerefMut,
    {
        match self {
            Self::Value(x) => Some(x),
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
            Self::Value(_) => rhs,
            Self::Null => Value::Null,
            Self::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn and_then<U, F>(self, f: F) -> Value<U>
    where
        F: FnOnce(T) -> Value<U>,
    {
        match self {
            Self::Value(x) => f(x),
            Self::Null => Value::Null,
            Self::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: FnOnce(&T) -> bool,
    {
        match self {
            Self::Value(x) => {
                if predicate(&x) {
                    Value::Value(x)
                } else {
                    Value::Null
                }
            }
            Self::Null => Value::Null,
            Self::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn or(self, rhs: Self) -> Self {
        match self {
            Self::Value(x) => Value::Value(x),
            Self::Null => rhs,
            Self::Unknown => Value::Unknown,
        }
    }
    #[inline]
    pub fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        match self {
            Self::Value(x) => Value::Value(x),
            Self::Null => f(),
            Self::Unknown => Value::Unknown,
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
            Self::Value(y) => x.eq(y),
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
            Self::Value(&x) => Value::Value(x),
            Self::Null => Value::Null,
            Self::Unknown => Value::Unknown,
        }
    }
    pub fn cloned(self) -> Value<T>
    where
        T: Clone,
    {
        match self {
            Self::Value(x) => Value::Value(x.clone()),
            Self::Null => Value::Null,
            Self::Unknown => Value::Unknown,
        }
    }
}
impl<T> Value<&mut T> {
    pub fn copied(self) -> Value<T>
    where
        T: Copy,
    {
        match self {
            Self::Value(&mut x) => Value::Value(x),
            Self::Null => Value::Null,
            Self::Unknown => Value::Unknown,
        }
    }
    pub fn cloned(self) -> Value<T>
    where
        T: Clone,
    {
        match self {
            Self::Value(x) => Value::Value(x.clone()),
            Self::Null => Value::Null,
            Self::Unknown => Value::Unknown,
        }
    }
}

impl<'a> Value<Cow<'a, str>> {
    #[inline]
    pub fn as_str(&'a self) -> &'a str {
        match self {
            Self::Value(x) => x.as_ref(),
            _ => "",
        }
    }
    #[inline]
    pub fn as_bytes(&'a self) -> &'a [u8] {
        match self {
            Self::Value(x) => x.as_bytes(),
            _ => "".as_bytes(),
        }
    }
    #[inline]
    pub fn extend<'b>(self) -> Value<Cow<'b, str>> {
        match self {
            Value::Value(Cow::Borrowed(s)) => Value::Value(Cow::from(s.to_owned())),
            Value::Value(Cow::Owned(s)) => Value::Value(Cow::from(s)),
            Value::Null => Value::Null,
            Value::Unknown => Value::Unknown,
        }
    }
}
impl<'a> Value<&'a Cow<'a, str>> {
    #[inline]
    pub fn as_str(&self) -> &'a str {
        match *self {
            Self::Value(x) => x.as_ref(),
            _ => "",
        }
    }
    #[inline]
    pub fn as_bytes(&self) -> &'a [u8] {
        match *self {
            Self::Value(x) => x.as_bytes(),
            _ => "".as_bytes(),
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

impl<T: Debug> Debug for Value<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(value) => Debug::fmt(value, f),
            Self::Null => f.write_str("Null"),
            Self::Unknown => f.write_str("Unknown"),
        }
    }
}

impl Debug for ValueAny {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => Debug::fmt(value, f),
            Self::Number(value) => Debug::fmt(value, f),
            Self::Bool(value) => Debug::fmt(value, f),
            Self::List(value) => Debug::fmt(value, f),
            Self::Map(value) => Debug::fmt(value, f),
            Self::Null => f.write_str("Null"),
            Self::Unknown => f.write_str("Unknown"),
        }
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

impl<'a> From<&'a str> for Value<Cow<'a, str>> {
    fn from(value: &'a str) -> Self {
        Self::Value(Cow::Borrowed(value))
    }
}
impl From<String> for Value<Cow<'_, str>> {
    fn from(value: String) -> Self {
        Self::Value(Cow::Owned(value))
    }
}

impl<'a> Deref for Value<Cow<'a, str>> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Display for Value<Cow<'_, str>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
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
        self.val.take().as_option()
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
        self.val.take().as_option()
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
