//! Browser readiness and opt-in, read-only telemetry for hands-free release checks.
use crate::*;
use bevy::text::FontSmoothing;
use bevy::world_serialization::{WorldInstance, WorldInstanceSpawner};

pub fn mobile() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .is_some_and(|root| root.has_attribute("data-mobile"))
}

/// Keep the GPU backbuffer, camera targets and glyph atlas at the actual display density.
/// CSS only controls the surface's size; it never scales a previously rendered frame.
pub fn resize(
    mut window: Single<(Entity, &mut Window), With<PrimaryWindow>>,
    mut ui_scale: ResMut<UiScale>,
    mut resized: MessageWriter<bevy::window::WindowResized>,
    mut scaled: MessageWriter<bevy::window::WindowScaleFactorChanged>,
) {
    let Some(browser) = web_sys::window() else {
        return;
    };
    let width = browser
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(1440.)
        .max(1.) as f32;
    let height = browser
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(900.)
        .max(1.) as f32;
    let dpr = browser.device_pixel_ratio().max(0.1) as f32;
    let size =
        UVec2::new((width * dpr).round() as u32, (height * dpr).round() as u32).max(UVec2::ONE);
    let (entity, window) = &mut *window;
    let scale_changed = (window.scale_factor() - dpr).abs() > 0.001;
    if window.physical_size() != size || scale_changed {
        window.resolution.set_scale_factor_override(Some(dpr));
        window.resolution.set_physical_resolution(size.x, size.y);
        resized.write(bevy::window::WindowResized {
            window: *entity,
            width,
            height,
        });
        if scale_changed {
            scaled.write(bevy::window::WindowScaleFactorChanged {
                window: *entity,
                scale_factor: dpr as f64,
            });
        }
    }
    // Scale the layout before text rasterization, so fonts get native-resolution atlases.
    let next = (width / 1440.).min(height / 900.);
    if (ui_scale.0 - next).abs() > 0.0001 {
        ui_scale.0 = next;
    }
}

pub fn status(
    game: Res<Session>,
    media: Option<Res<Media>>,
    assets: Res<AssetServer>,
    scenes: Query<&WorldInstance>,
    spawner: Res<WorldInstanceSpawner>,
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
    ui_scale: Res<UiScale>,
    camera: Single<(&Camera, &Msaa), With<WorldCamera>>,
    fonts: Query<(&TextFont, &Text, &ComputedNode, &UiGlobalTransform)>,
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
                if !boot.has_attribute("data-failed") {
                    let _ = boot.set_attribute("style", "display:none");
                }
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
        let texts: Vec<_> = fonts
            .iter()
            .map(|(f, t, n, at)| {
                serde_json::json!({
                    "text":t.0,"font_size":match f.font_size { bevy::text::FontSize::Px(size) => size, _ => 0. },
                    "min":(at.translation-n.size()*0.5).to_array(),
                    "max":(at.translation+n.size()*0.5).to_array(),
                })
            })
            .collect();
        let state = serde_json::json!({"ready":*ready, "case":game.0,"modal":format!("{:?}",game.0.modal),"buttons":buttons,
            "texts":texts,
            "panels":panels,"overview":director.overview,"camera_height":director.height,
            "render":{"logical":window.size().to_array(),"physical":window.physical_size().to_array(),
                "dpr":window.scale_factor(),"ui_scale":ui_scale.0,
                "viewport":camera.0.physical_viewport_size().map(|s|s.to_array()),
                "msaa":camera.1.samples(),
                "text_antialiased":fonts.iter().all(|(f,_,_,_)|f.font_smoothing == FontSmoothing::AntiAliased)},
            "cat_scale":cat.iter().next().map(|t|t.scale.to_array())});
        let _ = root.set_attribute("data-findcat", &state.to_string());
    }
}
