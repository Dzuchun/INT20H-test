use frontend::{api::dummy::DummyApi, App};
use leptos::{mount::mount_to_body, view};

fn main() {
    console_error_panic_hook::set_once();
    let api = DummyApi::new();
    mount_to_body(move || {
        view! { <App api /> }
    });
}
