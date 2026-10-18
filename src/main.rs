#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#[cfg(target_arch = "wasm32")]
mod browser;
mod navigation;
mod presentation;
#[cfg(not(target_arch = "wasm32"))]
mod testbed;
mod ui;

#[cfg(not(target_arch = "wasm32"))]
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::{
    audio::Volume,
    camera::{ScalingMode, Viewport},
    prelude::*,
    window::PrimaryWindow,
};
use findcat::game::*;
use std::{collections::VecDeque, path::PathBuf};

// The cat's model is 1.34 units high; the detective is 1.45 * 0.72 units.
// Keep a believable pet-to-person ratio in every scene, including the escape.
const CAT_SCALE: f32 = 0.28;

#[derive(Resource, Default)]
pub struct Session(pub Case);
#[derive(Resource, Default)]
pub struct Actions(pub VecDeque<Action>);
#[derive(Resource, Default)]
pub struct Travel {
    pub path: VecDeque<Vec2>,
    pub inspect: Option<usize>,
    pub last_near: Option<usize>,
}
#[derive(Resource)]
pub struct Files {
    pub save: PathBuf,
    pub testbed: bool,
    pub load: Option<Case>,
}
#[derive(Component)]
pub struct Detective;
#[derive(Component)]
pub struct Cat;
#[derive(Component)]
pub struct Fairy;
#[derive(Component)]
pub struct WorldCamera;
#[derive(Component)]
pub struct Music;
#[derive(Resource, Default)]
pub struct ToyMoment(pub f32);
#[derive(Component)]
pub struct Beacon(pub usize);
#[derive(Resource)]
pub struct Media {
    pub font: Handle<Font>,
    pub bold: Handle<Font>,
    pub photo: Handle<Image>,
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let (asset_dir, testbed, save, load) = native_files();
    #[cfg(target_arch = "wasm32")]
    let (asset_dir, testbed, save, load) = (
        "assets".to_owned(),
        false,
        PathBuf::from("findcat-save-v1"),
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("findcat-save-v1").ok().flatten())
            .and_then(|s| serde_json::from_str::<Case>(&s).ok())
            .filter(Case::validate),
    );
    run(asset_dir, testbed, save, load);
}

#[cfg(not(target_arch = "wasm32"))]
fn native_files() -> (String, bool, PathBuf, Option<Case>) {
    // Assets resolve next to the executable in a portable package, or the repository during development.
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let asset_dir = exe
        .as_ref()
        .map(|p| p.join("assets"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));
    let testbed = std::env::args().any(|a| a == "--testbed");
    let data_dir = if testbed {
        PathBuf::from("artifacts/testbed")
    } else {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("FindCat")
    };
    let _ = std::fs::create_dir_all(&data_dir);
    let save = data_dir.join("save.json");
    let load = std::fs::read_to_string(&save)
        .ok()
        .and_then(|s| serde_json::from_str::<Case>(&s).ok())
        .filter(Case::validate);
    (
        asset_dir.to_string_lossy().into_owned(),
        testbed,
        save,
        load,
    )
}

fn run(asset_dir: String, testbed: bool, save: PathBuf, load: Option<Case>) {
    let mut app = App::new();
    app.insert_resource(Files {
        save,
        testbed,
        load,
    })
    .insert_resource(ClearColor(Color::srgb(0.84, 0.87, 0.80)))
    .insert_resource(AmbientLight {
        color: Color::srgb(1., 0.91, 0.78),
        brightness: 300.,
        ..default()
    })
    .insert_resource(GlobalVolume::new(Volume::Linear(0.5)))
    .init_resource::<Session>()
    .init_resource::<Actions>()
    .init_resource::<Travel>()
    .init_resource::<ToyMoment>()
    .init_resource::<presentation::Director>()
    .add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: asset_dir,
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("{} · FindCat", TITLE),
                    resolution: (1440, 900).into(),
                    resizable: true,
                    canvas: Some("#game".into()),
                    fit_canvas_to_parent: cfg!(target_arch = "wasm32"),
                    prevent_default_event_handling: true,
                    resize_constraints: bevy::window::WindowResizeConstraints {
                        min_width: if cfg!(target_arch = "wasm32") {
                            1.
                        } else {
                            1100.
                        },
                        min_height: if cfg!(target_arch = "wasm32") {
                            1.
                        } else {
                            760.
                        },
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            }),
    )
    // Minimal Bevy builds explicitly register the components carried by our GLB scenes.
    .register_type::<Name>()
    .register_type::<Transform>()
    .register_type::<GlobalTransform>()
    .register_type::<bevy::transform::components::TransformTreeChanged>()
    .register_type::<Visibility>()
    .register_type::<InheritedVisibility>()
    .register_type::<ViewVisibility>()
    .register_type::<ChildOf>()
    .register_type::<Children>()
    .register_type::<Mesh3d>()
    .register_type::<MeshMaterial3d<StandardMaterial>>()
    .register_type::<bevy::camera::primitives::Aabb>()
    .register_type::<bevy::gltf::GltfMeshName>()
    .register_type::<bevy::gltf::GltfMaterialName>()
    .register_type::<bevy::gltf::GltfExtras>()
    .register_type::<bevy::gltf::GltfSceneExtras>()
    .register_type::<bevy::gltf::GltfMeshExtras>()
    .register_type::<bevy::gltf::GltfMaterialExtras>()
    .add_systems(Startup, setup)
    .add_systems(Startup, presentation::populate)
    .add_systems(
        Update,
        (
            input,
            ui::buttons,
            actions,
            movement,
            animate,
            presentation::direct,
            presentation::rig_parts,
            presentation::animate_parts,
            presentation::ambience,
            presentation::music,
            ui::rebuild,
            ui::proximity,
        )
            .chain(),
    );
    // Clamp the viewport after all window changes, before cameras are prepared for rendering.
    // This also covers programmatic resizes in the isolated testbed.
    app.add_systems(
        PostUpdate,
        (
            viewport.before(bevy::camera::CameraUpdateSystems),
            ui::markers
                .after(bevy::camera::CameraUpdateSystems)
                .after(bevy::transform::TransformSystems::Propagate),
        ),
    );
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, screenshots);
    #[cfg(not(target_arch = "wasm32"))]
    if testbed {
        app.init_resource::<testbed::Testbed>()
            .add_systems(Update, testbed::drive.after(ui::proximity));
    }
    #[cfg(target_arch = "wasm32")]
    app.add_systems(
        PreUpdate,
        browser::resize.before(bevy::ui::UiSystems::Focus),
    );
    #[cfg(target_arch = "wasm32")]
    app.add_systems(
        PostUpdate,
        browser::status.after(bevy::ui::UiSystems::Layout),
    );
    app.run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Media {
        font: assets.load("fonts/NanumGothic.ttf"),
        bold: assets.load("fonts/NanumGothic-Bold.ttf"),
        photo: assets.load("images/cat_portrait.png"),
    });
    commands.spawn((
        SceneRoot(assets.load(GltfAssetLabel::Scene(0).from_asset("models/neighborhood.glb"))),
        Transform::default(),
    ));
    commands.spawn((
        SceneRoot(assets.load(GltfAssetLabel::Scene(0).from_asset("models/detective.glb"))),
        Transform::from_xyz(-3., 0.15, -0.7).with_scale(Vec3::splat(0.72)),
        Detective,
    ));
    commands.spawn((
        SceneRoot(assets.load(GltfAssetLabel::Scene(0).from_asset("models/cat.glb"))),
        Transform::from_xyz(-4.9, 0.15, -1.5).with_scale(Vec3::splat(CAT_SCALE)),
        Cat,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.07))),
        MeshMaterial3d(mats.add(Color::srgb(0.95, 0.62, 0.28))),
        Transform::from_xyz(-3.8, 0.23, 0.0),
    ));
    commands.spawn((
        SceneRoot(assets.load(GltfAssetLabel::Scene(0).from_asset("models/fairy.glb"))),
        Transform::from_xyz(-2., 1.7, -0.3),
        Fairy,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 8500.,
            color: Color::srgb(1., 0.94, 0.84),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-6., 12., 8.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 17.8,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(12., 17.5, 23.).looking_at(Vec3::new(0., 0., 0.3), Vec3::Y),
        WorldCamera,
        Msaa::Sample4,
    ));
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: bevy::camera::ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
        Msaa::Sample4,
        bevy::ui_render::UiAntiAlias::On,
    ));
    let ring = meshes.add(Torus::new(0.23, 0.26));
    let material = mats.add(StandardMaterial {
        base_color: Color::srgb(1., 0.79, 0.32),
        emissive: LinearRgba::rgb(0.45, 0.19, 0.035),
        ..default()
    });
    for (id, c) in CLUES.iter().enumerate() {
        commands.spawn((
            Mesh3d(ring.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(c.position[0], 0.20, c.position[1]),
            Beacon(id),
        ));
    }
    commands.spawn((
        AudioPlayer::new(assets.load("audio/evening.wav")),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.26)),
        Music,
    ));
}

fn viewport(
    window: Single<&Window, With<PrimaryWindow>>,
    mut camera: Single<&mut Camera, With<WorldCamera>>,
) {
    let width = window.physical_width();
    let height = window.physical_height();
    let next = UVec2::new(width, height);
    if camera
        .viewport
        .as_ref()
        .is_none_or(|v| v.physical_size != next)
    {
        camera.viewport = Some(Viewport {
            physical_position: UVec2::ZERO,
            physical_size: next,
            ..default()
        });
    }
}

fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<WorldCamera>>,
    mut game: ResMut<Session>,
    mut acts: ResMut<Actions>,
    mut travel: ResMut<Travel>,
    player: Single<&Transform, With<Detective>>,
    ui_scale: Res<UiScale>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        acts.0.push_back(Action::Sound);
    }
    if keys.just_pressed(KeyCode::Escape) {
        acts.0.push_back(Action::Pause);
    }
    if keys.just_pressed(KeyCode::F1) {
        acts.0.push_back(Action::Help);
    }
    match game.0.phase {
        Phase::Title if game.0.modal == Modal::None => {
            if keys.just_pressed(KeyCode::Enter) {
                acts.0.push_back(Action::Start);
            }
        }
        Phase::Story(n) if game.0.modal == Modal::None => {
            if n == 0 && keys.just_pressed(KeyCode::Space) {
                acts.0.push_back(Action::Play);
            }
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                if n != 0 || keys.just_pressed(KeyCode::Enter) {
                    acts.0.push_back(Action::Next);
                }
            }
        }
        Phase::Search => {
            if keys.just_pressed(KeyCode::KeyJ) {
                acts.0.push_back(Action::Journal);
            }
            if keys.just_pressed(KeyCode::KeyQ) {
                acts.0.push_back(Action::Hint);
            }
            if keys.just_pressed(KeyCode::KeyR) && game.0.modal == Modal::None {
                acts.0.push_back(Action::Deduce);
            }
            if keys.just_pressed(KeyCode::KeyE) && game.0.modal == Modal::None {
                if let Some(i) = nearest(&player, &game.0) {
                    if i == 5 && game.0.chapter == 3 {
                        game.0.modal = Modal::Rescue;
                        game.0.revision += 1;
                    } else {
                        acts.0.push_back(Action::Inspect(i));
                    }
                }
            }
            if keys.just_pressed(KeyCode::Enter) && matches!(game.0.modal, Modal::Clue(_)) {
                acts.0.push_back(Action::Close);
            }
            if game.0.modal == Modal::None && mouse.just_pressed(MouseButton::Left) {
                if let Some(pos) = window.cursor_position().filter(|p| {
                    p.x < window.width()
                        && p.y > 155. * ui_scale.0
                        && p.y < window.height() - 155. * ui_scale.0
                }) {
                    if let Ok(ray) = camera.0.viewport_to_world(camera.1, pos) {
                        if let Some(distance) = ray
                            .intersect_plane(Vec3::new(0., 0.15, 0.), InfinitePlane3d::new(Vec3::Y))
                        {
                            let at = ray.get_point(distance);
                            travel.path = navigation::route(
                                Vec2::new(player.translation.x, player.translation.z),
                                Vec2::new(at.x, at.z),
                            )
                            .into();
                            travel.inspect = None;
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn screenshots(keys: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::F12) {
        let _ = std::fs::create_dir_all("artifacts");
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("artifacts/screenshot.png"));
    }
}

pub fn nearest(player: &Transform, game: &Case) -> Option<usize> {
    let pos = Vec2::new(player.translation.x, player.translation.z);
    CLUES
        .iter()
        .enumerate()
        .filter(|(i, _)| game.available(*i))
        .map(|(i, c)| (i, pos.distance(Vec2::from_array(c.position))))
        .filter(|(_, d)| *d < 1.65)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

fn actions(
    mut commands: Commands,
    mut acts: ResMut<Actions>,
    mut game: ResMut<Session>,
    mut files: ResMut<Files>,
    mut travel: ResMut<Travel>,
    mut player: Single<&mut Transform, With<Detective>>,
    assets: Res<AssetServer>,
    mut music: Query<&mut AudioSink, With<Music>>,
    time: Res<Time>,
    mut toy: ResMut<ToyMoment>,
) {
    while let Some(action) = acts.0.pop_front() {
        let playing = matches!(action, Action::Play) && game.0.phase == Phase::Story(0);
        if playing {
            toy.0 = time.elapsed_secs() + 0.9;
        }
        if matches!(action, Action::Start | Action::Restart) {
            travel.path.clear();
            travel.inspect = None;
            player.translation = Vec3::new(-3., 0.15, -0.7);
        }
        if matches!(action, Action::Continue) {
            if let Some(saved) = files.load.clone() {
                game.0 = saved;
                game.0.revision += 1;
                game.0.message = "모리 · 다시 왔구나. 수첩 속 흔적부터 이어 가자.".into();
                for mut sink in &mut music {
                    sink.set_volume(Volume::Linear(if game.0.muted { 0. } else { 0.26 }));
                }
            }
            continue;
        }
        if let Action::Inspect(id) = action {
            if !game.0.available(id) || game.0.phase != Phase::Search {
                continue;
            }
            let target = Vec2::from_array(CLUES[id].position);
            let reading_notebook =
                game.0.found[id] && matches!(game.0.modal, Modal::Journal | Modal::Clue(_));
            if !reading_notebook
                && Vec2::new(player.translation.x, player.translation.z).distance(target) > 1.65
            {
                travel.path = navigation::route(
                    Vec2::new(player.translation.x, player.translation.z),
                    target,
                )
                .into();
                travel.inspect = Some(id);
                game.0.message = format!("{}(으)로 걸어가는 중…", CLUES[id].place);
                game.0.revision += 1;
                continue;
            }
            if id == 5 && game.0.chapter == 3 && !reading_notebook {
                game.0.modal = Modal::Rescue;
                game.0.revision += 1;
                continue;
            }
        }
        let chapter = game.0.chapter;
        let count = game.0.count();
        let phase = game.0.phase.clone();
        game.0.apply(action);
        let sound = if playing {
            "meow"
        } else if phase == Phase::Story(0) && game.0.phase == Phase::Story(1) {
            "meow"
        } else if phase == Phase::Story(2) && game.0.phase == Phase::Story(3) {
            "discover"
        } else if game.0.phase == Phase::Reunion && phase != Phase::Reunion {
            "reunion"
        } else if game.0.chapter > chapter {
            "deduce"
        } else if game.0.count() > count {
            "discover"
        } else {
            "click"
        };
        if !game.0.muted {
            commands.spawn((
                AudioPlayer::new(assets.load(format!("audio/{sound}.wav"))),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.35)),
            ));
        }
        for mut sink in &mut music {
            sink.set_volume(Volume::Linear(if game.0.muted { 0. } else { 0.26 }));
        }
        if matches!(game.0.phase, Phase::Search | Phase::Reunion) {
            if let Err(e) = save_case(&files.save, &game.0) {
                warn!("Could not save progress: {e}");
                game.0.message =
                    "저장하지 못했어요. 저장 폴더의 여유 공간과 권한을 확인해 주세요.".into();
            } else {
                files.load = Some(game.0.clone());
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_case(path: &std::path::Path, c: &Case) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(c)?)?;
    std::fs::rename(tmp, path)
}

#[cfg(target_arch = "wasm32")]
pub fn save_case(_: &std::path::Path, c: &Case) -> std::io::Result<()> {
    let storage = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .ok_or_else(|| std::io::Error::other("Browser storage unavailable"))?;
    storage
        .set_item("findcat-save-v1", &serde_json::to_string(c)?)
        .map_err(|_| std::io::Error::other("Browser storage write failed"))
}

fn movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<Session>,
    director: Res<presentation::Director>,
    mut player: Single<&mut Transform, With<Detective>>,
    mut travel: ResMut<Travel>,
    mut acts: ResMut<Actions>,
) {
    if game.0.phase != Phase::Search || game.0.modal != Modal::None {
        return;
    }
    let mut dir = Vec2::ZERO;
    let forward = Vec2::new(-director.yaw.sin(), -director.yaw.cos());
    let right = Vec2::new(director.yaw.cos(), -director.yaw.sin());
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir += forward;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir -= forward;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir -= right;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir += right;
    }
    let mut pos = Vec2::new(player.translation.x, player.translation.z);
    let dt = time.delta_secs().min(0.05);
    if dir.length_squared() > 0.01 {
        travel.path.clear();
        travel.inspect = None;
        dir = dir.normalize();
    } else if let Some(next) = travel.path.front().copied() {
        let delta = next - pos;
        if delta.length() < 0.07 {
            travel.path.pop_front();
        } else {
            dir = delta.normalize();
        }
    }
    if dir.length_squared() > 0.01 {
        let speed = 2.65;
        let step = dir * speed * dt;
        let new = pos + step;
        if navigation::walkable(new) {
            pos = new;
        } else {
            if navigation::walkable(pos + Vec2::new(step.x, 0.)) {
                pos.x += step.x;
            }
            if navigation::walkable(pos + Vec2::new(0., step.y)) {
                pos.y += step.y;
            }
        }
        player.translation = Vec3::new(
            pos.x,
            0.15 + (time.elapsed_secs() * 13.).sin().abs() * 0.035,
            pos.y,
        );
        let rotation = Quat::from_rotation_y(dir.x.atan2(dir.y));
        player.rotation = player.rotation.slerp(rotation, (dt * 12.).min(1.));
    } else {
        player.translation.y = 0.15;
    }
    if let Some(id) = travel.inspect {
        if pos.distance(Vec2::from_array(CLUES[id].position)) < 1.5 {
            travel.inspect = None;
            travel.path.clear();
            acts.0.push_back(Action::Inspect(id));
        } else if travel.path.is_empty() {
            travel.inspect = None;
        }
    }
}

fn animate(
    time: Res<Time>,
    toy: Res<ToyMoment>,
    game: Res<Session>,
    director: Res<presentation::Director>,
    mut cats: Query<
        (&mut Transform, &mut Visibility),
        (With<Cat>, Without<Detective>, Without<Fairy>),
    >,
    mut fairies: Query<&mut Transform, (With<Fairy>, Without<Cat>, Without<Detective>)>,
    player: Single<&Transform, With<Detective>>,
    mut beacons: Query<
        (&Beacon, &mut Transform, &mut Visibility),
        (Without<Cat>, Without<Fairy>, Without<Detective>),
    >,
) {
    let t = time.elapsed_secs();
    for (mut c, mut vis) in &mut cats {
        *vis = Visibility::Visible;
        c.scale = Vec3::splat(CAT_SCALE);
        match game.0.phase {
            Phase::Title | Phase::Story(0) => {
                c.translation = Vec3::new(-4.35, 0.15, -0.7);
                if toy.0 > t {
                    c.translation.y += ((toy.0 - t) / 0.9 * std::f32::consts::PI).sin() * 0.25;
                }
                c.rotation = Quat::from_rotation_y(0.12 + (t * 0.5).sin() * 0.07);
                c.scale *= 1. + (t * 2.).sin() * 0.009;
            }
            Phase::Story(1) => {
                let s = (director.age / 1.3).clamp(0., 1.);
                c.translation = Vec3::new(
                    -4.35 + 2.1 * s,
                    0.15 + (s * 18.).sin().abs() * 0.09,
                    -0.7 - 1.1 * s,
                );
                c.rotation = Quat::from_rotation_y(1.6);
                if director.age > 1.8 {
                    *vis = Visibility::Hidden;
                }
            }
            Phase::Story(_) => *vis = Visibility::Hidden,
            Phase::Search => {
                if game.0.chapter < 2 {
                    *vis = Visibility::Hidden;
                } else {
                    c.translation = Vec3::new(4.7, 0.17, 3.25);
                    c.rotation = Quat::from_rotation_y(0.10);
                    c.scale.y *= 0.88 + (t * 2.).sin() * 0.008;
                    if game.0.trust > 0 {
                        c.translation.z += game.0.trust as f32 * 0.22;
                        c.scale.y = CAT_SCALE;
                    }
                }
            }
            Phase::Reunion => {
                let target = player.translation + Vec3::new(0.5, 0., 0.4);
                c.translation = c.translation.lerp(target, 0.025);
                c.rotation = Quat::from_rotation_y(-0.3 + (t * 0.7).sin() * 0.25);
                c.scale *= 1. + (t * 3.).sin() * 0.013;
            }
        }
    }
    for mut f in &mut fairies {
        let mut at = player.translation + Vec3::new(0.65, 1.65, -0.35);
        if game.0.hints > 0 && game.0.chapter < 3 && game.0.modal == Modal::None {
            if let Some(id) = PAIRS[game.0.chapter].iter().find(|id| !game.0.found[**id]) {
                at = Vec3::new(CLUES[*id].position[0], 1.7, CLUES[*id].position[1]);
            }
        }
        at.y += (t * 2.2).sin() * 0.11;
        f.translation = f.translation.lerp(at, 0.045);
        f.rotation = Quat::from_rotation_y(0.3 + (t * 0.7).sin() * 0.25);
    }
    for (b, mut tr, mut v) in &mut beacons {
        *v = if game.0.phase == Phase::Search
            && game.0.available(b.0)
            && (!game.0.found[b.0] || director.overview || b.0 == 5 && game.0.chapter >= 2)
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let s = if game.0.found[b.0] {
            0.7
        } else {
            1. + (t * 2. + b.0 as f32).sin() * 0.12
        };
        tr.scale = Vec3::splat(s);
    }
}
