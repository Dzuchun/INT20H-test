use leptos::{component, prelude::expect_context, IntoView};
use leptos_flavour::v;

use crate::{api::Api, react_errors, AppRouter, GeneralError};
use core::marker::PhantomData;

#[component]
pub fn Page<A: Api>(#[prop(optional)] _ph: PhantomData<A>) -> impl IntoView {
    let api = expect_context::<A>();
    let auth_user = api.auth_user();

    // react to errors in the api
    react_errors!(v(auth_user.clone().err()), GeneralError);

    let router = expect_context::<AppRouter<A>>();
    match api.auth_user() {
        Ok(Some(_)) => router.nav_home()(),
        Ok(None) => router.nav_login()(),
        Err(_) => {}
    };
}
