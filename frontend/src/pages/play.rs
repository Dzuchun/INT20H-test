use common::{QuestId, QuestInfo, QuestPage, Timestamp};
use leptos::prelude::*;
use leptos_flavour::{GetAnyExt, GetOptionExt, GetOptionOverResultExt, GetResultExt};
use thaw::Spinner;

use crate::{
    api::{error::GameError, Api},
    react_errors, AppRouter, GeneralError,
};

use core::marker::PhantomData;

use leptos_router::{hooks::use_params, params::Params};
#[derive(Params, Clone, Debug, PartialEq)]
struct PathParamsOpt {
    id: Option<QuestId>,
}
#[derive(Debug, Clone, PartialEq)]
struct PathParams {
    id: QuestId,
}
impl core::convert::TryFrom<PathParamsOpt> for PathParams {
    type Error = crate::GeneralError;
    fn try_from(value: PathParamsOpt) -> Result<Self, Self::Error> {
        let PathParamsOpt { id } = value;
        Ok(Self {
            id: id.ok_or(crate::GeneralError::ParamsError)?,
        })
    }
}

#[component]
pub fn Game<A: Api>(#[prop(optional)] _ph: PhantomData<A>) -> impl IntoView {
    let api = expect_context::<A>();
    let router = expect_context::<AppRouter<A>>();

    let (active_quest, active_quest_err) = Memo::new({
        let api = api.clone();
        move |_| {
            api.active_quest()
                .map_err(GameError::from)
                .and_then(|i| i.ok_or(GameError::NoActiveQuest))
        }
    })
    .split();

    let quest_page_resource = Resource::new(move || active_quest.get(), {
        let api = api.clone();
        move |active: Option<(QuestId, u32, Timestamp)>| {
            let api = api.clone();
            async move {
                let _ = active?;
                Some(api.quest_page().await)
            }
        }
    });
    let (quest_page, quest_page_err) = quest_page_resource
        .and_then(core::convert::identity)
        .split();

    react_errors!(
        active_quest_err, GameError;
        quest_page_err, GameError;
    );

    let nav_home = router.nav_home();
    Effect::new(move || {
        if active_quest_err.get().is_some() || quest_page_err.get().is_some() {
            nav_home();
        }
    });

    view! {
        <Transition fallback=||view!{<Spinner/>}>
        {move|| quest_page.get().map(|quest_page| {
            view!{
                <p>{format!("{:#?}", quest_page)}</p>
            }
        })}
        </Transition>
    }
}

#[component]
pub fn Page<A: Api>(#[prop(optional)] _ph: PhantomData<A>) -> impl IntoView {
    let api = expect_context::<A>();

    let (params, params_err) = use_params::<PathParamsOpt>()
        .map_err(|_| GeneralError::ParamsError)
        .and_then(PathParams::try_from)
        .split();

    // loading quest info
    let quest_start = Resource::new(
        move || params.with(|pars| pars.as_ref().map(|pars| pars.id)),
        {
            let api = api.clone();
            move |id: Option<QuestId>| {
                let api = api.clone();
                async move {
                    let id = id?;
                    Some(api.start_quest(id).await)
                }
            }
        },
    );
    let (quest_start, quest_start_err) = quest_start.and_then(core::convert::identity).split();

    // react to errors
    react_errors!(
        params_err, GeneralError;
        quest_start_err, GameError;
    );

    view! {
        <Suspense fallback=move || {
            view! { <Spinner /> }
        }>
            {move || {
                quest_start
                    .get()
                    .map(|()| {
                        view! { <Game<A> /> }
                    })
            }}
        </Suspense>
    }
}
