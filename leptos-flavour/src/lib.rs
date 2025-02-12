#![allow(clippy::type_complexity, clippy::wrong_self_convention)]

use std::fmt::Debug;

use leptos::prelude::{Action, ArcAction, Storage};

type Ret<T> = leptos::prelude::Signal<T>;

pub trait GetAnyExt {
    type Value: Send + Sync + 'static;

    fn res_ok<E>(self) -> Ret<Result<Self::Value, E>>
    where
        E: Send + Sync + 'static;

    fn res_err<T>(self) -> Ret<Result<T, Self::Value>>
    where
        T: Send + Sync + 'static;

    fn anymap<T, F>(self, map: F) -> Ret<T>
    where
        F: Fn(Self::Value) -> T + Send + Sync + 'static,
        T: Send + Sync + 'static;
}

impl<G: GetExt> GetAnyExt for G
where
    G: Send + Sync + 'static,
    G::Value: Send + Sync + 'static,
{
    type Value = G::Value;

    fn res_ok<E>(self) -> Ret<Result<Self::Value, E>>
    where
        E: Send + Sync + 'static,
    {
        Ret::derive(move || Ok(self.get_ext()))
    }

    fn res_err<T>(self) -> Ret<Result<T, Self::Value>>
    where
        T: Send + Sync + 'static,
    {
        Ret::derive(move || Err(self.get_ext()))
    }

    fn anymap<T, F>(self, map: F) -> Ret<T>
    where
        F: Fn(Self::Value) -> T + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        Ret::derive(move || map(self.get_ext()))
    }
}

pub trait GetOptionExt: GetExt<Value = Option<Self::Some>> {
    type Some: Send + Sync + 'static;

    fn map<U, F>(self, map: F) -> Ret<Option<U>>
    where
        F: Fn(Self::Some) -> U + Clone + Send + Sync + 'static,
        U: Send + Sync + 'static;

    fn and_then<U, F>(self, map: F) -> Ret<Option<U>>
    where
        F: Fn(Self::Some) -> Option<U> + Clone + Send + Sync + 'static,
        U: Send + Sync + 'static;

    fn map_or<U, D, F>(self, default: D, map: F) -> Ret<U>
    where
        D: GetExt<Value = U> + Send + Sync + 'static,
        F: Fn(Self::Some) -> U + Clone + Send + Sync + 'static,
        U: Send + Sync + 'static;

    fn ok_or<E>(self, err: E) -> Ret<Result<Self::Some, E>>
    where
        E: Clone + Send + Sync + 'static;

    fn ok_or_else<E, F>(self, err: F) -> Ret<Result<Self::Some, E>>
    where
        F: GetExt<Value = E> + Send + Sync + 'static,
        E: Send + Sync + 'static;

    fn unwrap_or<F>(self, or: F) -> Ret<Self::Some>
    where
        F: GetExt<Value = Self::Some> + Send + Sync + 'static;

    fn unwrap(self) -> Ret<Self::Some>;

    fn expect<F>(self, if_none: F) -> Ret<Self::Some>
    where
        F: Fn() -> core::convert::Infallible + Clone + Send + Sync + 'static;

    fn transpose(
        self,
    ) -> Ret<Result<Option<<Self::Some as GetResultExt>::Ok>, <Self::Some as GetResultExt>::Err>>
    where
        Self: Sized,
        Self::Some: GetResultExt,
    {
        todo!()
    }

    fn map_into<U>(self) -> Ret<Option<U>>
    where
        Self::Some: Into<U>,
        U: Send + Sync + 'static;

    fn flatten(self) -> Ret<Option<<Self::Some as GetOptionExt>::Some>>
    where
        Self: Sized,
        Self::Some: GetOptionExt,
    {
        todo!()
    }

    fn is_none(self) -> Ret<bool>;

    fn is_some(self) -> Ret<bool>;
}

impl<Some, G> GetOptionExt for G
where
    Some: Send + Sync + 'static,
    G: GetExt<Value = Option<Some>> + Send + Sync + 'static,
{
    type Some = Some;

    fn map<U, F>(self, map: F) -> Ret<Option<U>>
    where
        F: Fn(Self::Some) -> U + Clone + Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        Ret::derive(move || self.get_ext().map(map.clone()))
    }

    fn and_then<U, F>(self, map: F) -> Ret<Option<U>>
    where
        F: Fn(Self::Some) -> Option<U> + Clone + Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        Ret::derive(move || self.get_ext().and_then(map.clone()))
    }

    fn map_or<U, D, F>(self, default: D, map: F) -> Ret<U>
    where
        D: GetExt<Value = U> + Send + Sync + 'static,
        F: Fn(Self::Some) -> U + Clone + Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        Ret::derive(move || {
            self.get_ext()
                .map_or_else(|| default.get_ext(), map.clone())
        })
    }

    fn ok_or<E>(self, err: E) -> Ret<Result<Self::Some, E>>
    where
        E: Clone + Send + Sync + 'static,
    {
        Ret::derive(move || self.get_ext().ok_or(err.clone()))
    }

    fn ok_or_else<E, F>(self, err: F) -> Ret<Result<Self::Some, E>>
    where
        F: GetExt<Value = E> + Send + Sync + 'static,
        E: Send + Sync + 'static,
    {
        Ret::derive(move || self.get_ext().ok_or_else(|| err.get_ext()))
    }

    fn unwrap_or<F>(self, or: F) -> Ret<Self::Some>
    where
        F: GetExt<Value = Self::Some> + Send + Sync + 'static,
    {
        Ret::derive(move || self.get_ext().unwrap_or_else(|| or.get_ext()))
    }

    fn unwrap(self) -> Ret<Self::Some> {
        Ret::derive(move || self.get_ext().unwrap())
    }

    fn expect<F>(self, if_none: F) -> Ret<Self::Some>
    where
        F: Fn() -> core::convert::Infallible + Clone + Send + Sync + 'static,
    {
        Ret::derive(move || {
            if let Some(some) = self.get_ext() {
                some
            } else {
                let _: core::convert::Infallible = if_none();
                unreachable!("Infallible is produced above")
            }
        })
    }

    fn map_into<U>(self) -> Ret<Option<U>>
    where
        Self::Some: Into<U>,
        U: Send + Sync + 'static,
    {
        self.map(Self::Some::into)
    }

    fn is_none(self) -> Ret<bool> {
        Ret::derive(move || self.get_ext().is_none())
    }

    fn is_some(self) -> Ret<bool> {
        Ret::derive(move || self.get_ext().is_some())
    }
}

pub trait GetResultExt: GetExt<Value = Result<Self::Ok, Self::Err>> {
    type Ok: Send + Sync + 'static;
    type Err: Send + Sync + 'static;

    fn map<U, F>(self, map: F) -> Ret<Result<U, Self::Err>>
    where
        F: Fn(Self::Ok) -> U + Clone + Send + Sync + 'static,
        U: Send + Sync + 'static;

    fn map_err<E, F>(self, map: F) -> Ret<Result<Self::Ok, E>>
    where
        F: Fn(Self::Err) -> E + Clone + Send + Sync + 'static,
        E: Send + Sync + 'static;

    fn ok(self) -> Ret<Option<Self::Ok>>;

    fn err(self) -> Ret<Option<Self::Err>>;

    fn split(self) -> (Ret<Option<Self::Ok>>, Ret<Option<Self::Err>>)
    where
        Self: Clone;

    fn and_then<U, F>(self, then: F) -> Ret<Result<U, Self::Err>>
    where
        F: Fn(Self::Ok) -> Result<U, Self::Err> + Clone + Send + Sync + 'static,
        U: Send + Sync + 'static;

    fn or_else<E, F>(self, or_else: F) -> Ret<Result<Self::Ok, E>>
    where
        F: Fn(Self::Err) -> Result<Self::Ok, E> + Clone + Send + Sync + 'static,
        E: Send + Sync + 'static;

    fn unwrap(self) -> Ret<Self::Ok>
    where
        Self::Err: Debug;

    fn unwrap_err(self) -> Ret<Self::Err>
    where
        Self::Ok: Debug;

    fn expect<F>(self, if_err: F) -> Ret<Self::Ok>
    where
        F: Fn(Self::Err) -> core::convert::Infallible + Clone + Send + Sync + 'static;

    fn expect_err<F>(self, if_ok: F) -> Ret<Self::Err>
    where
        F: Fn(Self::Ok) -> core::convert::Infallible + Clone + Send + Sync + 'static;

    fn is_ok(self) -> Ret<bool>;

    fn is_err(self) -> Ret<bool>;
}

impl<Ok, Err, G> GetResultExt for G
where
    Ok: Send + Sync + 'static,
    Err: Send + Sync + 'static,
    G: GetExt<Value = Result<Ok, Err>> + Send + Sync + 'static,
{
    type Ok = Ok;

    type Err = Err;

    fn map<U, F>(self, map: F) -> Ret<Result<U, Self::Err>>
    where
        F: Fn(Self::Ok) -> U + Clone + Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        Ret::derive(move || self.get_ext().map(map.clone()))
    }

    fn map_err<E, F>(self, map: F) -> Ret<Result<Self::Ok, E>>
    where
        F: Fn(Self::Err) -> E + Clone + Send + Sync + 'static,
        E: Send + Sync + 'static,
    {
        Ret::derive(move || self.get_ext().map_err(map.clone()))
    }

    fn ok(self) -> Ret<Option<Self::Ok>> {
        Ret::derive(move || self.get_ext().ok())
    }

    fn err(self) -> Ret<Option<Self::Err>> {
        Ret::derive(move || self.get_ext().err())
    }

    fn split(self) -> (Ret<Option<Self::Ok>>, Ret<Option<Self::Err>>)
    where
        Self: Clone,
    {
        let ok = self.clone();
        let err = self;
        (
            Ret::derive(move || ok.get_ext().ok()),
            Ret::derive(move || err.get_ext().err()),
        )
    }

    fn and_then<U, F>(self, then: F) -> Ret<Result<U, Self::Err>>
    where
        F: Fn(Self::Ok) -> Result<U, Self::Err> + Clone + Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        Ret::derive(move || self.get_ext().and_then(then.clone()))
    }

    fn or_else<E, F>(self, or_else: F) -> Ret<Result<Self::Ok, E>>
    where
        F: Fn(Self::Err) -> Result<Self::Ok, E> + Clone + Send + Sync + 'static,
        E: Send + Sync + 'static,
    {
        Ret::derive(move || self.get_ext().or_else(or_else.clone()))
    }

    fn unwrap(self) -> Ret<Self::Ok>
    where
        Self::Err: Debug,
    {
        Ret::derive(move || self.get_ext().unwrap())
    }

    fn unwrap_err(self) -> Ret<Self::Err>
    where
        Self::Ok: Debug,
    {
        Ret::derive(move || self.get_ext().unwrap_err())
    }

    fn expect<F>(self, if_err: F) -> Ret<Self::Ok>
    where
        F: Fn(Self::Err) -> core::convert::Infallible + Clone + Send + Sync + 'static,
    {
        Ret::derive(move || match self.get_ext() {
            Ok(ok) => ok,
            Err(err) => {
                let _: core::convert::Infallible = if_err(err);
                unreachable!("Infallible is produced above")
            }
        })
    }

    fn expect_err<F>(self, if_ok: F) -> Ret<Self::Err>
    where
        F: Fn(Self::Ok) -> core::convert::Infallible + Clone + Send + Sync + 'static,
    {
        Ret::derive(move || match self.get_ext() {
            Err(err) => err,
            Ok(ok) => {
                let _: core::convert::Infallible = if_ok(ok);
                unreachable!("Infallible is produced above")
            }
        })
    }

    fn is_ok(self) -> Ret<bool> {
        Ret::derive(move || self.get_ext().is_ok())
    }

    fn is_err(self) -> Ret<bool> {
        Ret::derive(move || self.get_ext().is_err())
    }
}

pub trait GetOptionOverResultExt
where
    Self: GetOptionExt,
{
    type SomeOk: Send + Sync + 'static;
    type SomeErr: Send + Sync + 'static;

    fn transpose(self) -> Ret<Result<Option<Self::SomeOk>, Self::SomeErr>>;

    fn split(self) -> (Ret<Option<Self::SomeOk>>, Ret<Option<Self::SomeErr>>)
    where
        Self: Clone;
}

impl<Ok, Err, G> GetOptionOverResultExt for G
where
    Self: GetOptionExt<Some = Result<Ok, Err>> + Send + Sync + 'static,
    Ok: Send + Sync + 'static,
    Err: Send + Sync + 'static,
{
    type SomeOk = Ok;

    type SomeErr = Err;

    fn transpose(self) -> Ret<Result<Option<Self::SomeOk>, Self::SomeErr>> {
        Ret::derive(move || self.get_ext().transpose())
    }

    fn split(self) -> (Ret<Option<Self::SomeOk>>, Ret<Option<Self::SomeErr>>)
    where
        Self: Clone,
    {
        let ok = self.clone();
        let err = self;
        (
            Ret::derive(move || ok.get_ext().and_then(Result::ok)),
            Ret::derive(move || err.get_ext().and_then(Result::err)),
        )
    }
}

pub trait GetExt {
    type Value;
    fn get_ext(&self) -> Self::Value;
}

impl<T, U> GetExt for (T, U)
where
    T: GetExt,
    U: GetExt,
{
    type Value = (T::Value, U::Value);

    fn get_ext(&self) -> Self::Value {
        (self.0.get_ext(), self.1.get_ext())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub struct v<V>(pub V);

impl<T> GetExt for v<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Value = T;

    fn get_ext(&self) -> Self::Value {
        self.0.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub struct f<F>(pub F);

impl<T, F: Fn() -> T> GetExt for f<F> {
    type Value = T;

    fn get_ext(&self) -> Self::Value {
        (self.0)()
    }
}

macro_rules! impl_from_get {
    ($type:ty) => {
        impl<T> GetExt for $type
        where
            T: Send + Sync + 'static,
            Self: leptos::prelude::Get,
        {
            type Value = <Self as leptos::prelude::Get>::Value;

            #[inline]
            fn get_ext(&self) -> Self::Value {
                <Self as leptos::prelude::Get>::get(self)
            }
        }
    };
}

impl_from_get!(leptos::prelude::Signal<T>);
impl_from_get!(leptos::prelude::RwSignal<T>);
impl_from_get!(leptos::prelude::Memo<T>);
impl_from_get!(leptos::server::Resource<T>);

impl<I, O, S> GetExt for Action<I, O, S>
where
    O: Send + Sync + Clone + 'static,
    S: Storage<ArcAction<I, O>>,
{
    type Value = Option<O>;

    #[inline]
    fn get_ext(&self) -> Self::Value {
        self.value().get_ext()
    }
}
