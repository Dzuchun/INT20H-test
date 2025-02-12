use std::{future::Future, hash::Hash, marker::PhantomData};

use crate::{api::Api, api_resource, react_errors, ArcPage, ErrorAction};
use leptos::{component, prelude::*, view, IntoView};
use leptos_flavour::{GetAnyExt, GetOptionExt};
use serde::{de::DeserializeOwned, Serialize};
use thaw::{Pagination, Scrollbar, Spinner};

#[component]
pub fn Paginated<A, ApiPage, Fetcher, FetchErr, Fetch, Item, ItemView, ItemKey>(
    #[prop(optional)] _ph: PhantomData<(Item, ItemView, ItemKey)>,
    #[allow(unused)] api: PhantomData<A>,
    fetcher: Fetcher,
    item: impl Fn(Item) -> ItemView + Clone + Send + Sync + 'static,
    key: impl Fn(&Item) -> ItemKey + Clone + Send + Sync + 'static,
) -> impl IntoView
where
    A: Api,
    Fetcher: Fn(&A, usize) -> Fetch + Clone + Send + Sync + 'static,
    Fetch: Future<Output = Result<ApiPage, FetchErr>> + Send + Sync + 'static,
    ApiPage: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
    FetchErr: Clone
        + PartialEq
        + Serialize
        + DeserializeOwned
        + ErrorAction
        + core::fmt::Display
        + Send
        + Sync
        + 'static,
    ArcPage<Item>: From<ApiPage>,
    Item: Clone + Send + Sync + 'static,
    ItemView: IntoView + 'static,
    ItemKey: Eq + Hash + 'static,
{
    // store functions
    let item = StoredValue::new(item);
    let key = StoredValue::new(key);

    let page_no = RwSignal::new(1usize);
    let (page, page_err) =
        api_resource::<A, _, usize, ApiPage, FetchErr>(page_no.res_ok(), fetcher);
    let page = page.map_into::<ArcPage<Item>>();

    // handle errors
    react_errors!(page_err, FetchErr);

    view! {
        <Suspense fallback=|| {
            view! { <Spinner /> }
        }>
            {move || {
                page.get()
                    .map(|page| {
                        view! {
                            <Pagination
                                page_count=page.total_pages() as usize
                                page=page_no
                                sibling_count=2
                            />
                            <Scrollbar>
                                <ul>
                                    <For
                                        each=move || page.clone()
                                        key=key.get_value()
                                        children=item.get_value()
                                    />
                                </ul>
                            </Scrollbar>
                        }
                    })
            }}
        </Suspense>
    }
}
