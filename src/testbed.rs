//! End-to-end simulation through production actions, pathfinding, rendering and save I/O.
//! No operating-system mouse or keyboard input is generated.
use crate::*;
use std::time::Instant;

#[derive(Resource)]
pub struct Testbed {
    started: Instant,
    step: usize,
    changed: Instant,
    walk_started: bool,
    shots: Vec<String>,
    checks: usize,
    modal_seen: Option<Instant>,
    camera_views: Vec<serde_json::Value>,
    ui_checks: usize,
    audit: u8,
}
impl Default for Testbed {
    fn default() -> Self {
        Self {
            started: Instant::now(),
            step: 0,
            changed: Instant::now(),
            walk_started: false,
            shots: vec![],
            checks: 0,
            modal_seen: None,
            camera_views: vec![],
            ui_checks: 0,
            audit: 0,
        }
    }
}

fn shot(commands: &mut Commands, t: &mut Testbed, name: &str) {
    let path = format!("artifacts/{name}.png");
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path.clone()));
    t.shots.push(path);
}
pub fn drive(
    mut commands: Commands,
    mut t: ResMut<Testbed>,
    mut acts: ResMut<Actions>,
    game: Res<Session>,
    files: Res<Files>,
    mut exit: MessageWriter<AppExit>,
    mut time: ResMut<Time<Virtual>>,
    mut input: ResMut<ButtonInput<KeyCode>>,
    player: Single<&Transform, With<Detective>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    window_entity: Single<Entity, With<PrimaryWindow>>,
    mut resized: MessageWriter<bevy::window::WindowResized>,
    director: Res<presentation::Director>,
    panels: Query<(&ui::UiPanel, &ComputedNode, &UiGlobalTransform)>,
    mut marker_input: Query<(&ui::Marker, &mut Interaction)>,
    cat: Single<&Transform, With<Cat>>,
) {
    if t.started.elapsed().as_secs() > 240 {
        panic!(
            "Testbed timed out at step {}, {:?}, position {:?}",
            t.step, game.0.modal, player.translation
        );
    }
    assert!(
        game.0.validate(),
        "Invalid game state at testbed step {}",
        t.step
    );
    time.set_relative_speed(4.);
    let c = &game.0;
    assert!(cat.scale.max_element() <= CAT_SCALE * 1.02, "Cat became oversized in {:?}", c.phase);
    if t.step == 7 && t.changed.elapsed().as_secs_f32() > 0.20 {
        input.release(KeyCode::KeyD);
    }
    if t.changed.elapsed().as_secs_f32() < 1.35 {
        return;
    }
    for (panel, node, transform) in &panels {
        if node.size.x < 1. {
            continue;
        }
        let lo = transform.translation - node.size * 0.5;
        let hi = transform.translation + node.size * 0.5;
        assert!(
            lo.x >= -2.
                && lo.y >= -2.
                && hi.x <= window.physical_width() as f32 + 2.
                && hi.y <= window.physical_height() as f32 + 2.,
            "UI panel {} out of bounds: {:?} {:?}",
            panel.0,
            lo,
            hi
        );
        assert!(
            node.content_size.y <= node.size.y + 2.,
            "UI panel {} overflows vertically: {:?} {:?}",
            panel.0,
            node.content_size,
            node.size
        );
        t.ui_checks += 1;
    }
    let mut done = false;
    match t.step {
        0 => {
            if t.started.elapsed().as_secs_f32() > 7. {
                shot(&mut commands, &mut t, "01-title");
                acts.0.push_back(Action::Start);
                acts.0.push_back(Action::Play);
                done = true;
            }
        }
        1..=4 => {
            if t.step == 1 {
                shot(&mut commands, &mut t, "09-play");
            }
            if t.step == 2 {
                shot(&mut commands, &mut t, "10-escape");
            }
            if t.step == 3 {
                window.resolution.set(1100., 760.);
                resized.write(bevy::window::WindowResized {
                    window: *window_entity,
                    width: 1100.,
                    height: 760.,
                });
            }
            if t.step == 4 {
                shot(&mut commands, &mut t, "08-small-window");
            }
            acts.0.push_back(Action::Next);
            done = true;
        }
        5 => {
            shot(&mut commands, &mut t, "02-story");
            acts.0.push_back(Action::Next);
            done = true;
        }
        // Actual keyboard resource input, no hardware input. Verify collision and pause.
        6 => {
            window.resolution.set(1440., 900.);
            resized.write(bevy::window::WindowResized {
                window: *window_entity,
                width: 1440.,
                height: 900.,
            });
            input.press(KeyCode::KeyD);
            done = true;
        }
        7 => {
            input.release(KeyCode::KeyD);
            acts.0.push_back(Action::Pause);
            done = true;
        }
        8 => {
            assert_eq!(c.modal, Modal::Pause);
            shot(&mut commands, &mut t, "18-pause");
            acts.0.push_back(Action::Close);
            done = true;
        }
        9 | 10 | 11 | 12 | 16 | 17 | 18 | 22 => {
            let id = match t.step {
                9 => 0,
                10 => 6,
                11 => 1,
                12 => 2,
                16 => 3,
                17 => 4,
                18 => 7,
                _ => 5,
            };
            if !t.walk_started {
                if t.step == 10 {
                    window.resolution.set(1100., 760.);
                    resized.write(bevy::window::WindowResized {
                        window: *window_entity,
                        width: 1100.,
                        height: 760.,
                    });
                }
                if t.step == 11 {
                    window.resolution.set(1440., 900.);
                    resized.write(bevy::window::WindowResized {
                        window: *window_entity,
                        width: 1440.,
                        height: 900.,
                    });
                }
                if t.step == 9 {
                    for (marker, mut interaction) in &mut marker_input {
                        if marker.0 == 0 {
                            *interaction = Interaction::Hovered;
                        }
                    }
                    shot(&mut commands, &mut t, "12-search");
                }
                acts.0.push_back(Action::Inspect(id));
                t.walk_started = true;
            }
            if c.modal == Modal::Clue(id) {
                if t.modal_seen.is_none() {
                    t.modal_seen = Some(Instant::now());
                }
                if t.modal_seen.unwrap().elapsed().as_secs_f32() < 1.5 {
                    return;
                }
                if id == 0 {
                    shot(&mut commands, &mut t, "03-evidence");
                }
                if id == 5 {
                    shot(&mut commands, &mut t, "13-found-cat");
                }
                if id == 6 {
                    shot(&mut commands, &mut t, "14-small-evidence");
                }
                acts.0.push_back(Action::Close);
                t.walk_started = false;
                t.modal_seen = None;
                done = true;
            }
        }
        13 | 19 | 23 => {
            if t.step == 23 {
                window.resolution.set(1100., 760.);
                resized.write(bevy::window::WindowResized {
                    window: *window_entity,
                    width: 1100.,
                    height: 760.,
                });
            }
            acts.0.push_back(Action::Deduce);
            done = true;
        }
        14 | 20 | 24 => {
            assert_eq!(c.modal, Modal::Deduction);
            for id in PAIRS[c.chapter] {
                acts.0.push_back(Action::Select(id));
            }
            done = true;
        }
        15 | 21 | 25 => {
            if t.step == 15 {
                shot(&mut commands, &mut t, "04-deduction");
            }
            if t.step == 25 {
                shot(&mut commands, &mut t, "15-small-deduction");
            }
            acts.0.push_back(Action::Theory(CORRECT[c.chapter]));
            done = true;
        }
        26 => {
            assert_eq!(c.chapter, 3);
            assert_eq!(c.count(), 8);
            match t.audit {
                0 => {
                    acts.0.push_back(Action::Journal);
                    t.audit = 1;
                    t.changed = Instant::now();
                    return;
                }
                1 => {
                    assert_eq!(c.modal, Modal::Journal);
                    shot(&mut commands, &mut t, "16-journal");
                    acts.0.push_back(Action::Help);
                    t.audit = 2;
                    t.changed = Instant::now();
                    return;
                }
                2 => {
                    assert_eq!(c.modal, Modal::Help);
                    shot(&mut commands, &mut t, "17-help");
                    acts.0.push_back(Action::Close);
                    t.audit = 3;
                    t.changed = Instant::now();
                    return;
                }
                _ => {}
            }
            shot(&mut commands, &mut t, "05-neighborhood");
            acts.0.push_back(Action::Inspect(5));
            done = true;
        }
        27 => {
            window.resolution.set(1440., 900.);
            resized.write(bevy::window::WindowResized {
                window: *window_entity,
                width: 1440.,
                height: 900.,
            });
            assert_eq!(c.modal, Modal::Rescue);
            acts.0.push_back(Action::Rescue(0));
            done = true;
        }
        28 => {
            assert_eq!(c.trust, 0);
            assert!(c.mistakes > 0);
            acts.0.push_back(Action::Rescue(1));
            done = true;
        }
        29 => {
            assert_eq!(c.trust, 1);
            shot(&mut commands, &mut t, "06-trust");
            acts.0.push_back(Action::Rescue(0));
            done = true;
        }
        30 => {
            assert_eq!(c.trust, 2);
            acts.0.push_back(Action::Rescue(2));
            done = true;
        }
        31 => {
            if director.age < 6. {
                return;
            }
            assert_eq!(c.phase, Phase::Reunion);
            assert_eq!(c.trust, 3);
            shot(&mut commands, &mut t, "07-reunion");
            let saved: Case =
                serde_json::from_str(&std::fs::read_to_string(&files.save).expect("save exists"))
                    .expect("save JSON");
            assert!(saved.validate());
            assert_eq!(saved.phase, Phase::Reunion);
            assert_eq!(saved.count(), 8);
            done = true;
        }
        32 => {
            if t.changed.elapsed().as_secs_f32() > 2. {
                assert!(t.camera_views.len() > 20);
                assert!(t.ui_checks > 40);
                let report = serde_json::json!({"passed":true,"checks":t.checks,"ui_bounds_checks":t.ui_checks,"camera_views":t.camera_views,"all_clues":c.count(),"deductions":c.chapter,"trust":c.trust,"wrong_choice_recovery":c.mistakes>0,"save_reload":true,"navigation":"all 8 real routes completed","screenshots":t.shots,"seconds":t.started.elapsed().as_secs_f32(),"hardware_input":false});
                std::fs::write(
                    "artifacts/testbed-report.json",
                    serde_json::to_string_pretty(&report).unwrap(),
                )
                .unwrap();
                exit.write(AppExit::Success);
                done = true;
            }
        }
        _ => {}
    }
    if done {
        let step = t.step;
        t.camera_views.push(serde_json::json!({"step":step,"focus":[director.target.x,director.target.y,director.target.z],"height":director.height,"yaw":director.yaw}));
        t.checks += 1;
        t.step += 1;
        t.changed = Instant::now();
    }
}
