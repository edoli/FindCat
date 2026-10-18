//! Browser readiness and opt-in, read-only telemetry for hands-free release checks.
use crate::*;
use bevy::scene::SceneInstance;

pub fn status(
    game: Res<Session>,
    media: Option<Res<Media>>,
    assets: Res<AssetServer>,
    scenes: Query<&SceneInstance>,
    spawner: Res<SceneSpawner>,
    buttons: Query<(
        &ui::Click,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
    cat: Query<&Transform, With<Cat>>,
    panels: Query<(&ui::UiPanel, &ComputedNode, &UiGlobalTransform)>,
    director: Res<presentation::Director>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut ready: Local<bool>,
) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    if !*ready {
        let loaded = media.as_ref().is_some_and(|m| {
            assets.is_loaded_with_dependencies(&m.font)
                && assets.is_loaded_with_dependencies(&m.bold)
                && assets.is_loaded_with_dependencies(&m.photo)
        }) && scenes.iter().count() >= 4
            && scenes.iter().all(|s| spawner.instance_is_ready(**s));
        if loaded {
            *ready = true;
            if let Some(boot) = doc.get_element_by_id("boot") {
                let _ = boot.set_attribute("style", "display:none");
            }
            if let Some(canvas) = doc.get_element_by_id("game") {
                use wasm_bindgen::JsCast;
                if let Some(el) = canvas.dyn_ref::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }
        }
    }
    let Some(root) = doc.document_element() else {
        return;
    };
    if root.has_attribute("data-testbed") {
        let buttons: Vec<_> = buttons
            .iter()
            .filter(|(_, n, _, v)| v.get() && n.size().min_element() > 0.)
            .map(|(a, n, t, _)| {
                serde_json::json!({
                    "action": format!("{:?}", a.0),
                    "x": t.translation.x / window.scale_factor(),
                    "y": t.translation.y / window.scale_factor(),
                    "width": n.size().x / window.scale_factor(),
                    "height": n.size().y / window.scale_factor(),
                })
            })
            .collect();
        let panels: Vec<_> = panels
            .iter()
            .map(|(p, n, t)| {
                serde_json::json!({
                    "name":p.0,"min":(t.translation-n.size()*0.5).to_array(),
                    "max":(t.translation+n.size()*0.5).to_array(),
                    "overflow":n.content_size().y > n.size().y + 2.,
                })
            })
            .collect();
        let state = serde_json::json!({"ready":*ready, "case":game.0,"modal":format!("{:?}",game.0.modal),"buttons":buttons,
            "panels":panels,"overview":director.overview,"camera_height":director.height,
            "cat_scale":cat.iter().next().map(|t|t.scale.to_array())});
        let _ = root.set_attribute("data-findcat", &state.to_string());
    }
}
