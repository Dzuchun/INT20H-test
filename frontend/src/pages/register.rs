use std::marker::PhantomData;

use common::RegisterRequest;
use leptos::{
    component,
    prelude::{Action, Effect, Get, RwSignal, *},
    view, IntoView,
};
use leptos_flavour::GetOptionOverResultExt;
use thaw::{Button, Input, InputType};

use crate::{
    api::{error::RegisterError, Api},
    react_errors, tabs, AppRouter,
};

type RegisterAction = Action<RegisterRequest, Result<(), RegisterError>>;

#[component]
fn Email(register: RegisterAction) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let pass = RwSignal::new(String::new());

    view! {
        <Input value=name placeholder="Name" input_type=InputType::Text />
        <Input value=email placeholder="Email" input_type=InputType::Text />
        <Input value=pass placeholder="Password" input_type=InputType::Password />
        <Button
            on_click=move |_| {
                let _ = register
                    .dispatch(RegisterRequest {
                        email: email.get(),
                        name: name.get(),
                        pass: pass.get(),
                    });
            }
            disabled=register.pending()
        >
            "Register"
        </Button>
    }
}

#[component]
pub fn Page<A: Api>(#[prop(optional)] _ph: PhantomData<A>) -> impl IntoView {
    let api = expect_context::<A>();
    let router = expect_context::<AppRouter<A>>();

    let register: RegisterAction = Action::new(move |request: &RegisterRequest| {
        let api: A = api.clone();
        let request = request.clone();
        async move { api.register(request.clone()).await }
    });
    let (register_ok, register_err) = register.split();

    // on successful login, navigate to home page
    let home = router.nav_home();
    Effect::new(move || {
        if register_ok.get().is_some() {
            home();
        }
    });

    // react to error
    react_errors!(register_err, RegisterError);

    view! {
        {
            tabs! {
                + ("Email") => { view!{ <Email register /> } },
                - ("Other (something else entirely)") => { view!{<Button disabled=true>"(dummy button for other login methods)"</Button> } },
            }
        }
        <h3>"Have an account already?"</h3>
        {router.anchor_login()}
    }
}
