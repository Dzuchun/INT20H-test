use leptos::{component, prelude::*, IntoView};
use thaw::{Button, ComponentRef, Icon, Input, InputRef, InputType};
use thaw_utils::Model;

#[component]
pub fn EditableText(
    read: Signal<String>,
    write: impl Fn(String) + Copy + Send + Sync + 'static,
    #[prop(into)] placeholder: String,
) -> impl IntoView {
    let (edited, set_edited) = signal(false);
    let placeholder = Memo::new(move |_| placeholder.clone()); // TODO: omg this is UGLY
    view! {
        {move || {
            if edited.get() {
                let inner_value = Model::<String>::from(read.get());
                let comp_ref = ComponentRef::<InputRef>::new();
                Effect::new(move || {
                    if let Some(text) = comp_ref.get() {
                        text.focus();
                    }
                });
                // hack to focus to the input while editing
                // focus to the text field, while it's present

                view! {
                    <Input
                        input_type=InputType::Text
                        on_blur=move |_| {
                            let inner_value = inner_value.get();
                            if !inner_value.is_empty() {
                                write(inner_value);
                            }
                            set_edited.set(false);
                        }
                        value=inner_value
                        placeholder
                        comp_ref
                    />
                }
                    .into_any()
            } else {
                view! {
                    <span>
                        <Button on_click=move |_| {
                            set_edited.set(true);
                        }>
                            {move || view! { <p>{read.get()}</p> }}
                            <Icon icon=icondata::AiEditFilled />
                        </Button>
                    </span>
                }
                    .into_any()
            }
        }}
    }
}
