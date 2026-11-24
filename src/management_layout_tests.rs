use crate::state::State;
use rune::{Env, Runtime};

fn app(settings: bool) -> Runtime<State> {
    let mut state = State::demo();
    if settings {
        state.open_settings();
    } else {
        state.open_library();
    }
    let mut rt = Runtime::new(state, crate::ui::view, (1920., 1080.), Env::new(false, 1.));
    rt.frame(0.);
    rt
}

fn rect(rt: &Runtime<State>, key: &str) -> [f32; 4] {
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key))
        .unwrap_or_else(|| panic!("missing {key}"))
        .rect
}

fn resize(rt: &mut Runtime<State>, size: (f32, f32)) {
    rt.set_viewport(size, 1.);
    for _ in 0..40 {
        rt.frame(1. / 60.);
    }
}

#[test]
fn library_fills_the_content_pane_and_reflows_when_the_window_resizes() {
    let mut rt = app(false);
    let pane = rect(&rt, "main");
    let page = rect(&rt, "library-page");
    let modules = rect(&rt, "library-modules");
    let context = rect(&rt, "library-context");
    assert!((page[2] - pane[2]).abs() < 1.);
    assert!(context[0] > modules[0] + modules[2]);
    assert!((modules[1] - context[1]).abs() < 1.);
    assert!(
        (pane[0] + pane[2] - context[0] - context[2] - 28.).abs() < 1.,
        "unused space on the right"
    );
    resize(&mut rt, (920., 620.));
    let pane = rect(&rt, "main");
    let modules = rect(&rt, "library-modules");
    let context = rect(&rt, "library-context");
    assert!(
        context[1] >= modules[1] + modules[3],
        "pane={pane:?} modules={modules:?} context={context:?}"
    );
    for key in [
        "library-add",
        "library-close",
        "library-search",
        "library-tab-retained",
    ] {
        let r = rect(&rt, key);
        assert!(
            r[0] >= pane[0] && r[0] + r[2] <= pane[0] + pane[2] + 1.,
            "{key} overflows"
        );
        assert!(rt.find(key).is_some(), "{key} is not reachable");
    }
    resize(&mut rt, (1440., 900.));
    assert!((rect(&rt, "library-context")[1] - rect(&rt, "library-modules")[1]).abs() < 1.);
    assert!(rt.state.library.registry.is_empty());
}

#[test]
fn settings_pages_are_distinct_and_navigation_stays_visible_when_resizing_and_scrolling() {
    use crate::settings::SettingsPage;
    let mut rt = app(true);
    let section_keys = [
        "settings-data",
        "settings-security",
        "settings-appearance",
        "settings-updates",
    ];
    for size in [(1920., 1080.), (920., 620.), (1440., 900.)] {
        resize(&mut rt, size);
        for (page, key) in SettingsPage::ALL.into_iter().zip(section_keys) {
            let nav = format!("settings-nav-{}", page.key());
            let (x, y) = rt.find(&nav).expect("navigation must stay reachable");
            rt.pointer_down(x, y);
            rt.pointer_up(x, y);
            for _ in 0..40 {
                rt.frame(1. / 60.);
            }
            assert_eq!(rt.state.settings.page.key(), page.key());
            let pane = rect(&rt, "main");
            let content = rect(&rt, key);
            assert!((rect(&rt, "settings-page")[2] - pane[2]).abs() < 1.);
            assert!(
                (pane[0] + pane[2] - content[0] - content[2]).abs() < 1.,
                "unused space on the right"
            );
            for other in section_keys {
                if other != key {
                    assert!(
                        !rt.elements()
                            .iter()
                            .any(|e| e.key.as_deref() == Some(other)),
                        "inactive page was rendered"
                    );
                }
            }
            if page == SettingsPage::Appearance {
                rt.update(|_, tx| tx.scroll_into_view("settings-palette-gruvbox"));
                for _ in 0..40 {
                    rt.frame(1. / 60.);
                }
                assert!(rt.find("settings-palette-gruvbox").is_some());
            }
            for nav_page in SettingsPage::ALL {
                assert!(
                    rt.find(&format!("settings-nav-{}", nav_page.key()))
                        .is_some(),
                    "navigation scrolled away"
                );
            }
            assert!(rt.find("close-settings").is_some());
        }
    }
    rt.update(|_, tx| tx.focus("settings-nav-appearance", false));
    rt.frame(0.);
    rt.key(rune::Key::Enter.into());
    for _ in 0..40 {
        rt.frame(1. / 60.);
    }
    assert_eq!(
        rt.state.settings.page.key(),
        "appearance",
        "navigation must work from the keyboard"
    );
    rt.key(rune::Key::Right.into());
    rt.frame(0.);
    assert_eq!(rt.state.settings.page.key(), "updates");
    rt.key(rune::Key::Home.into());
    rt.frame(0.);
    assert_eq!(rt.state.settings.page.key(), "notebook");
    assert!(!rt.state.settings.checking);
}
