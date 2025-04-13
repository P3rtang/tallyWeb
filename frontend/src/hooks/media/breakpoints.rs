use super::*;

pub fn use_breakpoint(viewport: ViewPort, down: bool) -> Memo<bool> {
    let screen = use_screen();

    Memo::new(move |_| (screen.get().viewport() > viewport) ^ down)
}
