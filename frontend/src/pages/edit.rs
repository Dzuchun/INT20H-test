use common::{QuestId, QuestInfo};
use leptos::{component, prelude::*, view, IntoView};
use leptos_flavour::{GetAnyExt, GetOptionOverResultExt, GetResultExt};
use leptos_router::hooks::use_params;
use thaw::{Spinner, Textarea, TextareaResize, TextareaSize};

use crate::{
    api::Api,
    components::{EditableText, IconButton},
    react_errors, GeneralError,
};
use core::marker::PhantomData;

use leptos_router::params::Params;
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
fn Info(quest_info: RwSignal<QuestInfo>) -> impl IntoView {
    view! {
        <EditableText
            placeholder="Quest title"
            read=quest_info.anymap(|info| info.title)
            write=move |title| quest_info.update(|info| info.title = title)
        />
        <EditableText
            placeholder="Quest description"
            read=quest_info.anymap(|info| info.description)
            write=move |description| quest_info.update(|info| info.description = description)
        />
    }
}

#[component]
fn QuestPageRender(source: impl Fn() -> String + Copy + Send + Sync + 'static) -> impl IntoView {
    view! { {move || format!("{:#?}", common::parse_quest_page(source()))} }
}

#[component]
fn QuestPage<A: Api>(
    #[prop(optional)] _ph: PhantomData<A>,
    quest_id: QuestId,
    page: u32,
) -> impl IntoView {
    let api = expect_context::<A>();

    let (quest_page, quest_page_err) = Resource::new(move || (quest_id, page), {
        let api = api.clone();
        move |(quest_id, page)| {
            let api = api.clone();
            async move { api.get_page_source(quest_id, page).await }
        }
    })
    .split();

    let set_quest_page_source_action = Action::new(move |new_source: &String| {
        let new_source = new_source.clone();
        let api = api.clone();
        async move { api.set_page_source(quest_id, page, new_source).await }
    });
    let (_, set_quest_page_source_err) = set_quest_page_source_action.split();

    react_errors!(
        quest_page_err, GeneralError;
        set_quest_page_source_err, GeneralError;
    );

    view! {
        <Transition fallback=move || {
            view! { <Spinner /> }
        }>
            {move || {
                quest_page
                    .get()
                    .map(|quest_page_source| {
                        let value = RwSignal::new(quest_page_source);
                        view! {
                            <Textarea
                                disabled=set_quest_page_source_action.pending()
                                value
                                resize=TextareaResize::Vertical
                                size=TextareaSize::Large
                            />
                            <QuestPageRender source=move || value.get() />
                            <IconButton
                                text="Save page"
                                icon=icondata::AiSaveOutlined
                                on_click=move || {
                                    set_quest_page_source_action.dispatch(value.get());
                                }
                                disabled=set_quest_page_source_action.pending()
                            />
                        }
                    })
            }}
        </Transition>
    }
}

#[component]
fn Edit<A: Api>(#[prop(optional)] _ph: PhantomData<A>, quest_info: QuestInfo) -> impl IntoView {
    let api = expect_context::<A>();
    let quest_info = RwSignal::new(quest_info);

    // setting quest info
    let set_quest_info_action = Action::new(move |new_info: &QuestInfo| {
        let api = api.clone();
        let new_info = new_info.clone();
        async move { api.set_quest_info(new_info).await }
    });
    let (_, set_quest_info_err) = set_quest_info_action.split();

    react_errors!(set_quest_info_err, GeneralError);

    view! {
        <h1>"(quest editing)"</h1>
        <Info quest_info />
        <IconButton
            on_click=move || {
                set_quest_info_action.dispatch(quest_info.get());
            }
            icon=icondata::AiSaveFilled
            text="Save"
            disabled=set_quest_info_action.pending()
        />
        <hr />
        <h2>"Pages"</h2>
        <For
            each=move || 0..quest_info.get().pages
            key=move |page| (quest_info.get().id, *page)
            children=move |page| {
                view! {
                    <div>
                        <h3>{format!("Page {}", page + 1)}</h3>
                        <QuestPage<A> quest_id=quest_info.get().id page />
                        <hr />
                    </div>
                }
            }
        />
        <IconButton
            text="New page"
            icon=icondata::AiPlusOutlined
            on_click=move || {
                quest_info.update(|info| info.pages += 1);
            }
        />
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
    let quest_info = Resource::new(
        move || params.with(|pars| pars.as_ref().map(|pars| pars.id)),
        {
            let api = api.clone();
            move |id: Option<QuestId>| {
                let api = api.clone();
                async move { Some(api.get_quest_info(id?).await) }
            }
        },
    );
    let quest_info_err = quest_info.anymap(|v| v.flatten().and_then(Result::err));
    let quest_info = quest_info.anymap(|v| v.flatten().and_then(Result::ok));

    // react to errors
    react_errors!(
        params_err, GeneralError;
        quest_info_err, GeneralError;
    );

    view! {
        <Suspense fallback=move || {
            view! { <Spinner /> }
        }>
            {move || {
                quest_info
                    .get()
                    .map(|quest_info| {
                        view! { <Edit<A> quest_info /> }
                    })
            }}
        </Suspense>
    }
}
