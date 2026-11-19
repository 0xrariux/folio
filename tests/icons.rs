use folio::{
    document::PageContent,
    icons::{self, IconColor},
    model::Page,
    storage::Store,
    workspace::{self, Notebook},
};

#[test]
fn icon_and_color_survive_database_history_backup_and_legacy_documents() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("icons.db");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut page = Page::new(None);
            page.emoji = "folio:compass".into();
            page.icon_color = IconColor::Blue;
            let store = Store::open(&path).await.unwrap();
            store.save(&page).await.unwrap();
            store.checkpoint(&page).await.unwrap();
            let backup = temp.path().join("icons.folio");
            store
                .notebook(&workspace::asset_dir(&path))
                .await
                .unwrap()
                .write(&backup)
                .unwrap();
            drop(store);

            let reopened = Store::open(&path).await.unwrap();
            assert_eq!(reopened.load().await.unwrap(), vec![page.clone()]);
            assert_eq!(
                reopened.history(Some(&page.id)).await.unwrap()[0].page,
                page
            );
            let restored = Notebook::read(&backup).unwrap();
            assert_eq!(restored.pages, vec![page.clone()]);
            assert_eq!(restored.history[0].page, page);

            let mut old = serde_json::to_value(page.content()).unwrap();
            old.as_object_mut().unwrap().remove("icon_color");
            old["emoji"] = "🌿".into();
            let old: PageContent = serde_json::from_value(old).unwrap();
            assert_eq!(old.icon_color, IconColor::Default);
            assert_eq!(icons::choice(&old.emoji).unwrap().key, "leaf");
            let mut old_backup = serde_json::to_value(page).unwrap();
            old_backup.as_object_mut().unwrap().remove("icon_color");
            assert_eq!(
                serde_json::from_value::<Page>(old_backup)
                    .unwrap()
                    .icon_color,
                IconColor::Default
            );
            assert_eq!(
                serde_json::from_str::<IconColor>("\"future-color\"").unwrap(),
                IconColor::Default
            );
        });
}

#[test]
fn icon_colors_keep_visible_contrast_on_both_themes() {
    fn luminance(rgb: [u8; 3]) -> f64 {
        let linear = rgb.map(|v| {
            let v = f64::from(v) / 255.;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        });
        linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722
    }
    for (dark, background) in [(false, [253, 252, 249]), (true, [29, 31, 30])] {
        let background = luminance(background);
        for color in IconColor::ALL {
            if let Some(rgb) = color.rgb(dark) {
                let foreground = luminance(rgb);
                let contrast =
                    (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05);
                assert!(
                    contrast >= 3.,
                    "{} contrast {contrast}, dark={dark}",
                    color.name()
                );
            }
        }
    }
}
