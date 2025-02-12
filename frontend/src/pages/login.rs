use common::LoginRequest;
use core::marker::PhantomData;
use leptos::{component, prelude::*, view, IntoView};
use leptos_flavour::GetOptionOverResultExt;
use thaw::{Button, Input, InputType};

use crate::{
    api::{error::LoginError, Api},
    react_errors, tabs, AppRouter,
};

type LoginAction = Action<LoginRequest, Result<(), LoginError>>;

#[component]
fn Email(login: LoginAction) -> impl IntoView {
    let name_or_email = RwSignal::new(String::new());
    let pass = RwSignal::new(String::new());

    view! {
        <Input value=name_or_email placeholder="Name or email" input_type=InputType::Text />
        <Input value=pass placeholder="Password" input_type=InputType::Password />
        <Button
            on_click=move |_| {
                let _ = login
                    .dispatch(LoginRequest {
                        name_or_email: name_or_email.get(),
                        pass: pass.get(),
                    });
            }
            disabled=login.pending()
        >
            "Log in"
        </Button>
    }
}

#[component]
pub fn Page<A: Api>(#[prop(optional)] _ph: PhantomData<A>) -> impl IntoView {
    let api = expect_context::<A>();
    let router = expect_context::<AppRouter<A>>();

    let login: LoginAction = Action::new(move |request: &LoginRequest| {
        let api: A = api.clone();
        let request = request.clone();
        async move { api.login(request.clone()).await }
    });
    let (login_ok, login_err) = login.split();

    // on successful login, navigate to home page
    let home = router.nav_home();
    Effect::new(move || {
        if login_ok.get().is_some() {
            home();
        }
    });

    // react to error
    react_errors!(login_err, LoginError);

    view! {
        {
            tabs! {
                + ("Email") => { view!{ <Email login /> } },
                - ("Other (something else entirely)") => { view!{ <Button disabled=true>"(dummy button for other login methods)"</Button> } },
            }
        }
        <h3>"Don't have account yet?"</h3>
        {router.anchor_register()}
    }
}
