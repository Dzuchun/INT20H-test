#![allow(async_fn_in_trait, clippy::struct_field_names)]

pub mod api;

mod pages;
use derive_more::{Display, From};
use leptos_flavour::GetExt;
use pages::{Edit, Home, Login, Play, Register, Root};
use thiserror::Error;

mod components;

mod quest;

use std::{future::Future, sync::Arc};

use api::Api;
use common::{
    QuestHistoryPage, QuestHistoryRecord, QuestId, UserOwnedQuestRecord, UserOwnedQuestsPage,
};

use leptos::{component, prelude::*, view, IntoView};
use leptos_router::{components::Routes, hooks::use_navigate, path, NavigateOptions};

use leptos_router::components::{Route, Router};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use slice_dst::SliceWithHeader;
use thaw::{
    Button, ConfigProvider, Icon, Spinner, Theme, ToastIntent, ToastOptions, ToastPosition,
    ToasterProvider,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Warn,
    Error,
}

impl From<ToastKind> for ToastIntent {
    fn from(value: ToastKind) -> Self {
        match value {
            ToastKind::Info => ToastIntent::Info,
            ToastKind::Warn => ToastIntent::Warning,
            ToastKind::Error => ToastIntent::Error,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ToastInfo {
    pub title: String,
    pub message: String,
    pub intent: ToastKind,
}

impl ToastInfo {
    pub fn new(title: impl Into<String>, message: impl Into<String>, intent: ToastKind) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            intent,
        }
    }

    pub fn options(&self) -> ToastOptions {
        ToastOptions::default()
            .with_position(ToastPosition::Top)
            .with_intent(self.intent.into())
    }

    pub fn into_toast(self) -> impl IntoView {
        use thaw::{Toast, ToastBody, ToastTitle};
        view! {
            <Toast>
                <ToastTitle>{self.title}</ToastTitle>
                <ToastBody>{self.message}</ToastBody>
            </Toast>
        }
    }
}

pub trait ErrorAction {
    fn should_logout(&self) -> bool;

    fn should_log(&self) -> bool;

    fn toast_info(&self) -> Option<ToastInfo>;

    fn is_bug(&self) -> bool;
}

#[derive(Debug, Serialize, Deserialize, Display, Clone, PartialEq)]
pub enum EntityKind {
    #[display("user")]
    User,
    #[display("quest")]
    Quest,
    #[display("quest page")]
    QuestPage,
}

#[derive(Debug, Error, From, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeneralError {
    #[error("Unknown {_0}")]
    UnknownEntity(EntityKind),
    #[error("Operation is not authorized")]
    Unauthorized,
    #[error("Please log in")]
    RequestLogIn,
    #[error("Failed parsing path params")]
    ParamsError,
    #[error(transparent)]
    SourceParse(common::PageParseError),
    /// Implementation-specific
    #[error("Unknown error")]
    Unknown,
}

impl ErrorAction for GeneralError {
    fn toast_info(&self) -> Option<ToastInfo> {
        match self {
            GeneralError::UnknownEntity(kind) => Some(ToastInfo::new(
                format!("Unknown {kind}"),
                "please reload the page, or something",
                ToastKind::Warn,
            )),
            GeneralError::Unauthorized => Some(ToastInfo::new(
                "Oops!",
                "You are not allowed to access that",
                ToastKind::Error,
            )),
            GeneralError::RequestLogIn => Some(ToastInfo::new(
                "Who are you?",
                "This operation need you to be logged in.",
                ToastKind::Info,
            )),
            GeneralError::Unknown => Some(ToastInfo::new(
                "Error occurred!",
                "try reloading the page",
                ToastKind::Error,
            )),
            GeneralError::ParamsError => Some(ToastInfo::new(
                "We don't know where you are",
                "url path you've navigated to does not seem correct. please go back",
                ToastKind::Info,
            )),
            GeneralError::SourceParse(_) => None,
        }
    }

    fn should_log(&self) -> bool {
        match self {
            GeneralError::Unknown
            | GeneralError::UnknownEntity(_)
            | GeneralError::Unauthorized
            | GeneralError::SourceParse(_) => true,
            GeneralError::RequestLogIn | GeneralError::ParamsError => false,
        }
    }

    fn should_logout(&self) -> bool {
        match self {
            GeneralError::Unauthorized
            | GeneralError::Unknown
            | GeneralError::UnknownEntity(_)
            | GeneralError::ParamsError
            | GeneralError::SourceParse(_) => false,
            GeneralError::RequestLogIn => true,
        }
    }

    fn is_bug(&self) -> bool {
        match self {
            GeneralError::UnknownEntity(_)
            | GeneralError::Unauthorized
            | GeneralError::Unknown
            | GeneralError::ParamsError => true,
            GeneralError::RequestLogIn | GeneralError::SourceParse(_) => false,
        }
    }
}

fn not_found<A: Api>() -> impl IntoView {
    view! {
        <div>
            <p>"Not found :("</p>
            {expect_context::<AppRouter<A>>().anchor_root()}
        </div>
    }
}

#[derive(Clone)]
pub struct AppRouter<A: Api>(A);

impl<A: Api> Default for AppRouter<A> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! endpoint_nav {
    ($name:ident, $path:literal) => {
        pub fn $name(&self) -> impl Fn() + Clone + 'static {
            let navigate = use_navigate();
            move || navigate($path, NavigateOptions::default())
        }
    };
}

macro_rules! endpoint_anchor {
    ($name:ident, $path:literal, $($children:tt)+) => {
        pub fn $name(&self) -> impl IntoView {
            view! {
                <a href=$path >$($children)+</a>
            }
        }
    };
}

impl<A: Api> AppRouter<A> {
    /// ### Panics
    /// If corresponding API context was not provided
    #[inline]
    pub fn new() -> Self {
        Self(use_context::<A>().expect("Api should be provided"))
    }

    pub fn routes(&self) -> impl IntoView {
        let fallback = not_found::<A>;
        view! {
            <Routes fallback>
                <Route path=path!("/") view=move || view! { <Root<A> /> } />
                <Route path=path!("/home") view=move || view! { <Home<A> /> } />
                <Route path=path!("/login") view=move || view! { <Login<A> /> } />
                <Route path=path!("/register") view=move || view! { <Register<A> /> } />
                <Route path=path!("/edit/:id") view=move || view! { <Edit<A> /> } />
                <Route path=path!("/play/:id") view=move || view! { <Play<A> /> } />
            </Routes>
        }
    }
    pub fn log_out(&self) -> impl Fn() + Clone + 'static {
        let navigate = use_navigate();
        let api = self.0.clone();
        move || {
            api.logout();
            navigate("/login", NavigateOptions::default());
        }
    }

    endpoint_anchor!(anchor_root, "/", <Spinner />"Home");
    endpoint_anchor!(anchor_home, "/home", "Home");
    endpoint_anchor!(anchor_login, "/login", "Log in");
    endpoint_anchor!(anchor_register, "/register", "Register");

    endpoint_nav!(nav_root, "/");
    endpoint_nav!(nav_login, "/login");
    endpoint_nav!(nav_home, "/home");
    endpoint_nav!(nav_register, "/register");

    pub fn nav_edit(
        &self,
        quest_id: impl GetExt<Value = QuestId> + Clone + 'static,
    ) -> impl Fn() + Clone + 'static {
        let navigate = use_navigate();
        move || {
            navigate(
                format!("/edit/{}", quest_id.get_ext()).as_str(),
                NavigateOptions::default(),
            );
        }
    }

    pub fn anchor_edit(
        &self,
        quest_id: impl GetExt<Value = QuestId> + Clone + 'static,
    ) -> impl IntoView {
        let path = format!("/edit/{}", quest_id.get_ext());
        view! {
            <a href=path>
                <Icon icon=icondata::AiEditFilled />
            </a>
        }
    }

    pub fn anchor_play(
        &self,
        quest_id: impl GetExt<Value = QuestId> + Clone + 'static,
    ) -> impl IntoView {
        let path = format!("/play/{}", quest_id.get_ext());
        view! {
            <a href=path>
                <Icon icon=icondata::AiPlayCircleFilled />
            </a>
        }
    }
}

#[component]
pub fn App<A: Api>(api: A) -> impl IntoView {
    provide_context(api);
    provide_context(AppRouter::<A>::new());
    let theme = RwSignal::new(Theme::dark()); // TODO: persist theme
    let toggle_theme = move || {
        theme.update(|theme| {
            *theme = match theme.name.as_str() {
                "light" => Theme::dark(),
                "dark" => Theme::light(),
                _ => unreachable!(),
            };
        });
    };
    view! {
        <ConfigProvider theme>
            <ToasterProvider>
                <Router>
                    <nav>
                        <p>"Fererum test!"</p>
                        {expect_context::<AppRouter<A>>().anchor_root()}
                        <Button on_click=move |_| toggle_theme()>"(toggle theme)"</Button>
                    </nav>
                    <main>{expect_context::<AppRouter<A>>().routes()}</main>
                </Router>
            </ToasterProvider>
        </ConfigProvider>
    }
}

#[macro_export]
macro_rules! react_errors {
    ($( $error_signal:expr $(,$error_type:ty)?);+ $(;)?) => { #[allow(unused_imports)] {
        use std::collections::HashSet;
        use $crate::{ErrorAction, ToastInfo, ToastKind, use_logout};
        use leptos_flavour::GetExt;
        use leptos::{logging::warn, prelude::Effect};
        use thaw::{ToasterInjection};
        let logout = use_logout::<A>();
        let toaster = ToasterInjection::expect_context();
        Effect::new(move || {
            let mut toasts = HashSet::<ToastInfo>::new();
            $(
                if let Some(error) = $error_signal.get_ext() {
                    $(let error: $error_type = error; )?
                    if error.should_log() {
                        warn!("{error}");
                    }

                    if error.should_logout() {
                        logout();
                    }

                    if let Some(toast_info) = error.toast_info() {
                        toasts.insert(toast_info);
                    }

                    if error.is_bug() {
                        toasts.insert(ToastInfo::new("Please note", "Application encountered an error, it's likely to misbehave", ToastKind::Warn));
                    }
                }
            )+

            for toast in toasts {
                let options = toast.options();
                toaster.dispatch_toast(
                    move || toast.into_toast(),
                    options,
                );
            }
        });
    }};
}

/// ### Panics
/// If app router was not provided
pub fn use_logout<A: Api>() -> impl Fn() + Clone {
    let app_router = use_context::<AppRouter<A>>().expect("App router must be provided");
    app_router.log_out()
}

/*

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValV<T>(pub T);

pub trait Val<V> {
    fn val(&self) -> V;
}

impl<T: Clone> Val<T> for ValV<T> {
    fn val(&self) -> T {
        self.0.clone()
    }
}

impl<T1, S1: Val<T1>, T2, S2: Val<T2>> Val<(T1, T2)> for (S1, S2) {
    fn val(&self) -> (T1, T2) {
        (self.0.val(), self.1.val())
    }
}

impl<T: Clone> Val<Option<T>> for T {
    fn val(&self) -> Option<T> {
        Some(self.clone())
    }
}

impl<T> Val<Option<T>> for Resource<T>
where
    Self: DefinedAt,
    T: Clone + Send + Sync + 'static,
{
    fn val(&self) -> Option<T> {
        self.get()
    }
}

impl<I, O, S> Val<Option<O>> for Action<I, O, S>
where
    O: Send + Sync + Clone + 'static,
    S: Storage<ArcAction<I, O>>,
{
    fn val(&self) -> Option<O> {
        self.value().get()
    }
}

impl<T> Val<T> for RwSignal<T>
where
    Self: DefinedAt,
    T: Clone + Send + Sync + 'static,
{
    fn val(&self) -> T {
        self.get()
    }
}

impl<T> Val<T> for Memo<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn val(&self) -> T {
        self.get()
    }
}

impl<T> Val<T> for Signal<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn val(&self) -> T {
        self.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValE<T>(pub T);

impl<T, E, S: Val<T>> Val<Result<T, E>> for ValE<S> {
    fn val(&self) -> Result<T, E> {
        Ok(self.0.val())
    }
}

impl<T1, T2, E, S1: Val<Result<T1, E>>, S2: Val<Result<T2, E>>> Val<Result<(T1, T2), E>>
    for (S1, S2)
{
    fn val(&self) -> Result<(T1, T2), E> {
        Ok((self.0.val()?, self.1.val()?))
    }
}

*/

/*
/// ### Panics
/// If api context was never provided
pub fn api_resource<A, Fut, Source, Ok, Err>(
    source: impl Val<Result<Source, Err>> + Sync + Send + 'static,
    op: impl (for<'a> Fn(&'a A, Source) -> Fut) + Sync + Send + Clone + 'static,
) -> Resource<Result<Ok, Err>>
where
    A: Api,
    Fut: Future<Output = Result<Ok, Err>> + Send,
    Err: Serialize + DeserializeOwned + Clone + Sync + Send + PartialEq,
    Ok: Serialize + DeserializeOwned + Clone + Sync + Send,
    Source: Clone + Sync + Send + PartialEq + 'static,
{
    let api = use_context::<A>().expect("Api must be provided");
    Resource::new(
        move || source.val(),
        move |source| {
            let api = api.clone();
            let op = op.clone();
            async move {
                let source = source?;
                op(&api, source).await
            }
        },
    )
}
*/

/// ### Panics
/// If api context was never provided
pub fn api_resource<A, Fut, Source, Ok, Err>(
    source: impl GetExt<Value = Result<Source, Err>> + Sync + Send + 'static,
    op: impl (for<'a> Fn(&'a A, Source) -> Fut) + Sync + Send + Clone + 'static,
) -> (Signal<Option<Ok>>, Signal<Option<Err>>)
where
    A: Api,
    Fut: Future<Output = Result<Ok, Err>> + Send,
    Err: Serialize + DeserializeOwned + Clone + Sync + Send + PartialEq + 'static,
    Ok: Serialize + DeserializeOwned + Clone + Sync + Send + 'static,
    Source: Clone + Sync + Send + PartialEq + 'static,
{
    let api = use_context::<A>().expect("Api must be provided");
    let resource = Resource::new(
        move || source.get_ext(),
        move |source| {
            let api = api.clone();
            let op = op.clone();
            async move {
                let source = source?;
                op(&api, source).await
            }
        },
    );
    (
        Signal::derive(move || resource.get().and_then(Result::ok)),
        Signal::derive(move || resource.get().and_then(Result::err)),
    )
}

/*
pub fn split_reactive<Ok, Err>(
    reactor: impl Val<Result<Ok, Err>> + Clone + Send + Sync + 'static,
) -> (Signal<Option<Ok>>, Signal<Option<Err>>)
where
    Err: Send + Sync + 'static,
    Ok: Send + Sync + 'static,
{
    let ok_reactor = reactor.clone();
    let err_reactor = reactor;
    (
        Signal::derive(move || ok_reactor.val().ok()),
        Signal::derive(move || err_reactor.val().err()),
    )
}

pub fn split_reactive_opt<Ok, Err>(
    reactor: impl Val<Option<Result<Ok, Err>>> + Clone + Send + Sync + 'static,
) -> (Signal<Option<Ok>>, Signal<Option<Err>>)
where
    Err: Send + Sync + 'static,
    Ok: Send + Sync + 'static,
{
    let ok_reactor = reactor.clone();
    let err_reactor = reactor;
    (
        Signal::derive(move || ok_reactor.val().and_then(Result::ok)),
        Signal::derive(move || err_reactor.val().and_then(Result::err)),
    )
}

pub fn and_then_reactive<T, U, E>(
    input: impl Val<Result<T, E>> + Send + Sync + 'static,
    and_then: impl Fn(T) -> Result<U, E> + Send + Sync + Clone + 'static,
) -> Signal<Result<U, E>>
where
    U: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    Signal::derive(move || input.val().and_then(and_then.clone()))
}

pub fn map_err_reactive<T, E, V>(
    input: impl Val<Result<T, E>> + Send + Sync + 'static,
    map_err: impl Fn(E) -> V + Send + Sync + Clone + 'static,
) -> Signal<Result<T, V>>
where
    T: Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    Signal::derive(move || input.val().map_err(map_err.clone()))
}

pub fn map_some_reactive<T, U>(
    input: impl Val<Option<T>> + Send + Sync + 'static,
    map_some: impl Fn(T) -> U + Send + Sync + Clone + 'static,
) -> Signal<Option<U>>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
{
    Signal::derive(move || input.val().map(map_some.clone()))
}

pub fn map_reactive<T, U>(
    input: impl Val<T> + Send + Sync + 'static,
    map: impl Fn(T) -> U + Send + Sync + Clone + 'static,
) -> Signal<U>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
{
    Signal::derive(move || map(input.val()))
}
*/

#[derive(Debug, Clone)]
pub struct ArcPage<Item>(Arc<SliceWithHeader<(u32, u32), Item>>);

impl<Item> ArcPage<Item> {
    pub fn items(&self) -> &[Item] {
        &self.0.slice
    }

    pub fn page(&self) -> u32 {
        self.0.header.0
    }

    pub fn total_pages(&self) -> u32 {
        self.0.header.1
    }
}

pub struct ArcPageIter<Item> {
    page: ArcPage<Item>,
    pos: usize,
}

impl<Item: Clone> Iterator for ArcPageIter<Item> {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.page.items().get(self.pos).cloned();
        self.pos += 1;
        res
    }
}

impl<Item: Clone> IntoIterator for ArcPage<Item> {
    type Item = Item;

    type IntoIter = ArcPageIter<Item>;

    fn into_iter(self) -> Self::IntoIter {
        ArcPageIter { page: self, pos: 0 }
    }
}

pub type QuestHistoryArcPage = ArcPage<QuestHistoryRecord>;

impl From<QuestHistoryPage> for QuestHistoryArcPage {
    fn from(value: QuestHistoryPage) -> Self {
        Self(SliceWithHeader::new(
            (value.page, value.total_pages),
            value.data,
        ))
    }
}

pub type UserQuestsArcPage = ArcPage<UserOwnedQuestRecord>;

impl From<UserOwnedQuestsPage> for UserQuestsArcPage {
    fn from(value: UserOwnedQuestsPage) -> Self {
        Self(SliceWithHeader::new(
            (value.page, value.total_pages),
            value.data,
        ))
    }
}

/// This macro IS correct.
///
/// Inline via lsp
#[macro_export]
macro_rules! path_params {
    {
        $(#[derive($($derive_ident:ident),+)])?
        struct $struct_name:ident {
            $($field_name:ident: $field_type: ty),+$(,)?
        }
    } => {
        ::paste::paste!{
        use leptos_router::params::{Params};
        #[derive(Params $($(,$derive_ident)+)?)]
        struct [< $struct_name Opt>] {
            $($field_name: Option<$field_type>),+
        }

        $(#[derive($($derive_ident),+)])?
        struct $struct_name {
            $($field_name: $field_type),+
        }

        impl core::convert::TryFrom<[< $struct_name Opt >]> for $struct_name {
            type Error = $crate::GeneralError;

            fn try_from(value: [< $struct_name Opt >]) -> Result<Self, Self::Error> {
                let [< $struct_name Opt >] { $($field_name),+ } = value;
                Ok(Self { $($field_name: $field_name.ok_or($crate::GeneralError::ParamsError)?),+ })
            }
        }
        }
    }
}

#[macro_export]
macro_rules! tabs {
    {$(
        $oc:tt ($($t:tt)+) => {$($c:tt)+}
    ),+$(,)?} => {{
        use rapidhash::rapidhash;
        use leptos::{view, prelude::Show};
        use thaw::{TabList, Tab};
        let tab: RwSignal<String>;
        $(tabs!{@h tab $oc ($($t)+) => {$($c)+} })+
        view! {
            <TabList selected_value=tab> $({ tabs!{@t ($($t)+) => {$($c)+} } })+ </TabList>
            <div> $({ tabs!{@c tab ($($t)+) => {$($c)+}} })+ </div>
        }
    }};
    {@h $tab:ident + $($tt:tt)+} => {{
        const TAB: u64 = rapidhash(stringify!($($tt)+).as_bytes());
        $tab = RwSignal::new(TAB.to_string());
    }};
    {@h $tab:ident - $($tt:tt)+} => {};
    {@t ($($t:tt)+) => $($c:tt)+} => {{
        const TAB: u64 = rapidhash(stringify!(($($t)+) => $($c)+).as_bytes());
        let tab_name = TAB.to_string();
        view!{ <Tab value=tab_name > {$($t)+} </Tab> }
    }};
    {@c $tab:ident ($($t:tt)+) => $($c:tt)+} => {{
        const TAB: u64 = rapidhash(stringify!(($($t)+) => $($c)+).as_bytes());
        let tab_name = TAB.to_string();
        view!{ <Show when=move|| $tab.get()==tab_name> $($c)+ </Show> }
    }};
}
