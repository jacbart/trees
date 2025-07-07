use iocraft::prelude::*;
use std::time::Duration;

#[derive(Default, Clone, Props)]
struct Worktree<'a> {
    name: &'a str,
    _git_ref: &'a str,
    _status: &'a str,
}

impl<'a> Worktree<'a> {
    fn new(name: &'a str, git_ref: &'a str, status: &'a str) -> Self {
        Self {
            name: name,
            _git_ref: git_ref,
            _status: status,
        }
    }
}

#[derive(Default, Props)]
struct WorktreeProps<'a> {
    worktree: Option<&'a Vec<Worktree<'a>>>,
}

#[component]
#[allow(non_snake_case)]
fn Selector<'a>(props: &WorktreeProps<'a>, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (stdout, _stderr) = hooks.use_output();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut scroll_offset = hooks.use_state(|| 0i32);
    let mut should_exit = hooks.use_state(|| false);

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Char('q') => should_exit.set(true),
                    KeyCode::Up => scroll_offset.set((scroll_offset.get() - 1).max(0)),
                    KeyCode::Down => scroll_offset.set(scroll_offset.get() + 1),
                    _ => {}
                }
            }
            _ => {}
        }
    });
    hooks.use_future(async move {
        loop {
            smol::Timer::after(Duration::from_secs(1)).await;
            stdout.println("")
        }
    });

    if should_exit.get() {
        system.exit();
    }

    let default = vec![Worktree::new("default", "default", "default")];
    let wt = props.worktree.unwrap_or(&default);

    let mut content = "".to_owned();
    let mut first_run: bool = true;
    wt.iter().for_each(|worktree| {
        match first_run {
            true => first_run = false,
            false => content.push_str("\n"),
        }
        content.push_str(worktree.name)
    });

    element! {
        View(
            flex_direction: FlexDirection::Column,
            padding: 1,
            align_items: AlignItems::FlexStart,
        ) {
            Text(content: "Use arrow keys to scroll. Press \"q\" to exit.")
            View(
                margin: 1,
                width: 78,
                height: 10,
                overflow: Overflow::Hidden,
            ) {
                View(
                    position: Position::Absolute,
                    top: -scroll_offset.get(),
                ) {
                    Text(content: content)
                }
            }
        }
    }
}
