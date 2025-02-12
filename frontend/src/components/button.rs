use core::marker::PhantomData;
use leptos::{component, prelude::*, view, IntoView};
use leptos_flavour::{v, GetOptionOverResultExt};
use thaw::{Button, Icon};

use crate::{api::Api, react_errors, AppRouter};

#[component]
pub fn NewQuestButton<A: Api>(#[prop(optional)] _ph: PhantomData<A>) -> impl IntoView {
    let api = expect_context::<A>();
    let new_quest = Action::new(move |(): &()| {
        let api = api.clone();
        async move { api.create_quest().await }
    });

    let (new_quest_id, new_quest_err) = new_quest.split();

    // navigate to edit page, if ok
    let router = expect_context::<AppRouter<A>>();
    Effect::new(move || {
        if let Some(quest_id) = new_quest_id.get() {
            router.nav_edit(v(quest_id))();
        }
    });

    // react to error
    react_errors!(new_quest_err);

    view! {
        <Button
            on_click=move |_| {
                new_quest.dispatch(());
            }
            disabled=new_quest.pending()
        >
            <Icon icon=icondata::AiPlusOutlined />
            <p>"New quest"</p>
        </Button>
    }
}

#[component]
pub fn IconButton(
    on_click: impl Fn() + Send + Sync + 'static,
    icon: icondata::Icon,
    #[prop(into)] text: String,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <Button on_click=move |_| on_click() disabled>
            <Icon icon />
            <p>{text}</p>
        </Button>
    }
}
