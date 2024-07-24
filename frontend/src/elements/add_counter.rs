use super::*;
use leptos::*;
use stylance::import_style;

#[component]
pub fn NewCounterButton() -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let resource = expect_context::<StateResource>();
    let save_handler = expect_context::<SaveHandler>();

    let on_click = move |_| {
        let name = format!("Counter {}", store.get_untracked().root_nodes().len() + 1);
        store.update(|s| {
            let c_id = s.new_countable(&name, CountableKind::Counter, None);
            let p_id = s.new_countable("Phase 1", CountableKind::Phase, Some(c_id));
            let _ = save_handler.save(
                Box::new([s.get(&c_id).unwrap(), s.get(&p_id).unwrap()].to_vec()),
                move || resource.refetch(),
            );
        });
    };

    import_style!(style, "new-counter.module.scss");

    view! {
        <button on:click=on_click class=style::new_counter>
            New Counter
        </button>
    }
}
