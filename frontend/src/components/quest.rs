use leptos::prelude::*;

use common::QuestId;
use leptos_flavour::{v, GetOptionOverResultExt};
use thaw::Spinner;

use crate::{api::Api, react_errors, AppRouter, GeneralError};

use core::marker::PhantomData;

#[component]
pub fn Quest<A: Api>(
    #[prop(optional)] _ph: PhantomData<A>,
    quest_id: QuestId,
    #[prop(optional)] show_edit: bool,
    #[prop(optional)] show_start: bool,
) -> impl IntoView {
    let api = expect_context::<A>();

    let (quest_info, quest_info_err) = Resource::new(move || quest_id, {
        let api = api.clone();
        move |quest_id| {
            let api = api.clone();
            async move { api.get_quest_info(quest_id).await }
        }
    })
    .split();

    react_errors!(quest_info_err, GeneralError);

    view! {
        <div>
            <Suspense fallback=|| {
                view! { <Spinner /> }
            }>
                {
                    let router = expect_context::<AppRouter<A>>();
                    move || {
                        quest_info
                            .get()
                            .map(|quest_info| {
                                view! {
                                    <h3>{quest_info.title}</h3>
                                    <p>{quest_info.description}</p>
                                    <p prop:color="grey">{format!("by {:?}", quest_info.owner)}</p>
                                    {show_edit.then_some(router.anchor_edit(v(quest_info.id)))}
                                    {show_start.then_some(router.anchor_play(v(quest_info.id)))}
                                }
                            })
                    }
                }
            </Suspense>
        </div>
    }
}
