use std::marker::PhantomData;

use crate::{
    api::{static_get_user_info, Api},
    api_resource,
    components::{NewQuestButton, Paginated, QuestInfo},
    react_errors, tabs, use_logout, AppRouter, GeneralError,
};

use common::{UserId, UserInfo, UserOwnedQuestRecord};
use leptos::{component, prelude::*, view, IntoView};
use leptos_flavour::v;
use thaw::{Button, Spinner};

#[component]
fn UserInfo<A: Api>(#[prop(optional)] _ph: PhantomData<A>) -> impl IntoView {
    let api = expect_context::<A>();

    let (user_info, user_info_err) = api_resource::<A, _, UserId, UserInfo, GeneralError>(
        v(api.require_auth_user()),
        static_get_user_info,
    );

    // handle errors
    react_errors!(user_info_err, GeneralError);

    view! {
        <Suspense fallback=|| {
            view! { <Spinner /> }
        }>
            {move || {
                user_info
                    .get()
                    .map(|user_info| {
                        view! {
                            <div>
                                <h1>
                                    <span>"(home page of a user)"</span>
                                    <span>{user_info.name.clone()}</span>
                                </h1>
                                <p>"full info: "</p>
                                <p>{format!("{user_info:#?}")}</p>
                            </div>
                        }
                    })
            }}
        </Suspense>
    }
}

#[component(transparent)]
fn Quests<A: Api>(#[prop(optional)] _ph: PhantomData<A>) -> impl IntoView {
    let api = PhantomData::<A>;
    let item = move |record: UserOwnedQuestRecord| {
        view! { <QuestInfo<A> quest_id=record.id show_edit=true show_start=true /> }
    };

    view! {
        <Paginated
            api
            fetcher=|api: &A, page| {
                let api: A = api.clone();
                async move {
                    let id = api.require_auth_user()?;
                    api.user_quests(id, u32::try_from(page).unwrap()).await
                }
            }
            key=|record| record.id
            item
        />
    }
}

#[component(transparent)]
fn History<A: Api>(#[prop(optional)] _ph: PhantomData<A>) -> impl IntoView {
    let api = PhantomData::<A>;
    view! {
        <Paginated
            api
            fetcher=|api: &A, page| {
                let api: A = api.clone();
                async move {
                    let id = api.require_auth_user()?;
                    api.quest_history(id, u32::try_from(page).unwrap()).await
                }
            }
            key=|record| (record.quest_id, record.user_id, record.started_at)
            item=|record| {
                view! {
                    <div>
                        <h3>"(history record)"</h3>
                        <h4>{format!("Record: {record:?}")}</h4>
                        <p>"(some text, idk)"</p>
                    </div>
                }
            }
        />
    }
}

#[component]
pub fn Page<A: Api>(#[prop(optional)] _ph: PhantomData<A>) -> impl IntoView {
    view! {
        <UserInfo<A> />
        <NewQuestButton<A> />
        <Button on_click={
            let logout = use_logout::<A>();
            move |_| logout()
        }>"Log out"</Button>
        <h2>"Quest history"</h2>
        {
            tabs! {
                + ("Quest history") => { view! {<History<A> />} },
                - ("Created quests") => { view!{ <Quests<A> /> } },
            }
        }
    }
}
