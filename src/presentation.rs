//! Camera direction, ambient life, and small character performances.
use crate::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};

#[derive(Resource)]
pub struct Director {
    pub age: f32,
    pub overview: bool,
    pub celebrate: f32,
    pub target: Vec3,
    pub height: f32,
    pub yaw: f32,
    pub shown_trust: usize,
    zoom: f32,
    phase: Phase,
    chapter: usize,
    found: [bool; 8],
    bond_sound: Option<usize>,
}
impl Default for Director {
    fn default() -> Self {
        Self {
            age: 0.,
            overview: false,
            celebrate: 0.,
            target: Vec3::new(-5.35, 0.6, -0.5),
            height: 4.2,
            yaw: 0.18,
            shown_trust: 0,
            zoom: 0.,
            phase: Phase::Title,
            chapter: 0,
            found: [false; 8],
            bond_sound: None,
        }
    }
}
#[derive(Resource)]
pub struct Effects {
    mesh: Handle<Mesh>,
    gold: Handle<StandardMaterial>,
    pink: Handle<StandardMaterial>,
}
#[derive(Component)]
pub struct Drift {
    kind: u8,
    seed: f32,
    origin: Vec3,
}
#[derive(Component)]
pub struct Spark {
    velocity: Vec3,
    life: f32,
    age: f32,
}
#[derive(Component)]
pub struct LivingPart {
    base: Transform,
    kind: u8,
}

pub const ENDING_DURATION: f32 = 30.;

pub fn intro_cat_position(bond: &Bonding) -> Vec3 {
    let p = if bond.active { bond.progress() } else { 0. };
    match bond.stage() {
        0 => Vec3::new(
            -4.35 + (p * std::f32::consts::TAU).sin() * 0.2,
            0.15 + (p * std::f32::consts::PI).sin().powi(8) * 0.16,
            -0.7 + (p * std::f32::consts::PI).sin() * 0.25,
        ),
        1 => Vec3::new(
            -4.35 + smooth((p - 0.15) / 0.35) * 0.85 - smooth((p - 0.75) / 0.25) * 0.10,
            0.15 + ((p - 0.30) / 0.25)
                .clamp(0., 1.)
                .mul_add(std::f32::consts::PI, 0.)
                .sin()
                * 0.20,
            -0.7 + smooth(p / 0.7) * 0.3,
        ),
        _ => Vec3::new(-3.6 + (p * std::f32::consts::PI).sin() * 0.06, 0.15, -0.4),
    }
}

pub fn ending_beat(age: f32) -> u8 {
    if age < 6. {
        0
    } else if age < 15. {
        1
    } else if age < 22. {
        2
    } else {
        3
    }
}

fn smooth(value: f32) -> f32 {
    let x = value.clamp(0., 1.);
    x * x * (3. - 2. * x)
}

/// Fade the actual GLB materials; visibility also keeps absent characters from casting shadows.
pub fn character_opacity(phase: &Phase, age: f32, fairy: bool) -> f32 {
    if fairy {
        match phase {
            Phase::Title | Phase::Story(0..=2) => 0.,
            Phase::Story(3) => smooth((age - 0.25) / 1.5),
            Phase::Reunion => 1. - smooth((age - 5.) / 1.5),
            _ => 1.,
        }
    } else {
        match phase {
            Phase::Story(1) => 1. - smooth((age - 0.65) / 0.85),
            Phase::Story(2..) => 0.,
            _ => 1.,
        }
    }
}

#[derive(Component)]
pub struct FadingMaterial {
    fairy: bool,
    color: Color,
    emissive: LinearRgba,
    mode: AlphaMode,
}

pub fn fade_characters(
    mut commands: Commands,
    game: Res<Session>,
    director: Res<Director>,
    parents: Query<&ChildOf>,
    roots: Query<Has<Fairy>, Or<(With<Cat>, With<Fairy>)>>,
    mut meshes: Query<(
        Entity,
        &mut MeshMaterial3d<StandardMaterial>,
        Option<&FadingMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut handle, cached) in &mut meshes {
        if let Some(fade) = cached {
            let alpha = character_opacity(&game.0.phase, director.age, fade.fairy);
            let color = fade.color.with_alpha(fade.color.alpha() * alpha);
            let emissive = fade.emissive * alpha;
            let mode = if alpha < 1. {
                AlphaMode::Blend
            } else {
                fade.mode
            };
            // Avoid uploading all character materials every frame after a fade has finished.
            let changed = materials.get(&handle.0).is_some_and(|material| {
                material.base_color != color
                    || material.emissive != emissive
                    || material.alpha_mode != mode
            });
            if changed {
                if let Some(mut material) = materials.get_mut(&handle.0) {
                    material.base_color = color;
                    material.emissive = emissive;
                    material.alpha_mode = mode;
                }
            }
        } else {
            let Some(fairy) = parents
                .iter_ancestors(entity)
                .find_map(|e| roots.get(e).ok())
            else {
                continue;
            };
            let Some(mut material) = materials.get(&handle.0).cloned() else {
                continue;
            };
            let fade = FadingMaterial {
                fairy,
                color: material.base_color,
                emissive: material.emissive,
                mode: material.alpha_mode,
            };
            let alpha = character_opacity(&game.0.phase, director.age, fairy);
            material.base_color = material.base_color.with_alpha(fade.color.alpha() * alpha);
            material.emissive *= alpha;
            if alpha < 1. {
                material.alpha_mode = AlphaMode::Blend;
            }
            handle.0 = materials.add(material);
            commands.entity(entity).insert(fade);
        }
    }
}

#[derive(Component)]
pub struct CozyProp;

pub fn epilogue(
    game: Res<Session>,
    director: Res<Director>,
    mut actions: ResMut<Actions>,
    mut toys: Query<(&mut Transform, &mut Visibility), With<Toy>>,
    mut props: Query<&mut Visibility, (With<CozyProp>, Without<Toy>)>,
) {
    let home = game.0.phase == Phase::Reunion && director.age >= 6.;
    for mut vis in &mut props {
        *vis = if home {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (mut tr, mut vis) in &mut toys {
        *vis = if matches!(game.0.phase, Phase::Title | Phase::Story(0))
            || home && director.age < 15.
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        tr.translation = if game.0.phase == Phase::Story(0) {
            let bond = &game.0.bonding;
            let p = if bond.active { bond.progress() } else { 0. };
            match bond.stage() {
                0 => Vec3::new(
                    -3.95 + (p * 12.).sin() * 0.22,
                    0.3 + (p * 9.).sin().abs() * 0.10,
                    -0.3,
                ),
                _ => Vec3::new(-4.2 + smooth(p / 0.55) * 0.85, 0.23, -0.3),
            }
        } else if home {
            Vec3::new(
                -4.0 + ((director.age - 6.) * 1.8 + 0.6).sin() * 0.5,
                0.23,
                -0.1,
            )
        } else {
            Vec3::new(-3.8, 0.23, 0.)
        };
        if game.0.phase == Phase::Story(0) && game.0.bonding.stage() == 2 {
            *vis = Visibility::Hidden;
        }
    }
    if game.0.phase == Phase::Reunion
        && game.0.modal == Modal::None
        && director.age >= ENDING_DURATION
    {
        actions.0.push_back(Action::MainMenu);
    }
}

pub fn populate(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // A little familiar corner for the quiet afternoon after the rescue.
    for (size, at, color) in [
        (
            Vec3::new(1.5, 0.035, 0.95),
            Vec3::new(-3.9, 0.19, -0.2),
            Color::srgb(0.76, 0.60, 0.43),
        ),
        (
            Vec3::new(0.40, 0.12, 0.30),
            Vec3::new(-4.45, 0.25, -0.38),
            Color::srgb(0.90, 0.78, 0.58),
        ),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_translation(at),
            CozyProp,
            Visibility::Hidden,
        ));
    }
    let dot = meshes.add(Sphere::new(0.025));
    let gold = materials.add(StandardMaterial {
        base_color: Color::srgb(1., 0.73, 0.24),
        emissive: LinearRgba::rgb(0.55, 0.28, 0.04),
        ..default()
    });
    let pink = materials.add(StandardMaterial {
        base_color: Color::srgb(0.94, 0.47, 0.45),
        ..default()
    });
    commands.insert_resource(Effects {
        mesh: dot.clone(),
        gold: gold.clone(),
        pink: pink.clone(),
    });
    for i in 0..22 {
        let seed = i as f32 * 1.67;
        let at = Vec3::new(
            seed.sin() * 6.,
            0.5 + (seed * 0.8).sin().abs(),
            (seed * 1.7).cos() * 4.5,
        );
        commands.spawn((
            Mesh3d(dot.clone()),
            MeshMaterial3d(gold.clone()),
            Transform::from_translation(at),
            Drift {
                kind: 0,
                seed,
                origin: at,
            },
        ));
    }
    let wing = meshes.add(Sphere::new(1.));
    for i in 0..6 {
        let seed = i as f32 * 1.7;
        let at = Vec3::new(-2. + seed.sin() * 3., 0.9, 2.7 + seed.cos() * 1.4);
        commands
            .spawn((
                Transform::from_translation(at),
                Visibility::Inherited,
                Drift {
                    kind: 1,
                    seed,
                    origin: at,
                },
            ))
            .with_children(|p| {
                for side in [-1., 1.] {
                    p.spawn((
                        Mesh3d(wing.clone()),
                        MeshMaterial3d(if i % 2 == 0 {
                            pink.clone()
                        } else {
                            gold.clone()
                        }),
                        Transform::from_xyz(side * 0.055, 0., 0.)
                            .with_scale(Vec3::new(0.065, 0.016, 0.085)),
                    ));
                }
            });
    }
    for i in 0..7 {
        commands.spawn((
            Mesh3d(dot.clone()),
            MeshMaterial3d(materials.add(Color::srgb(0.53, 0.82, 0.85))),
            Transform::default(),
            Drift {
                kind: 2,
                seed: i as f32 * 0.9,
                origin: Vec3::new(-0.2, 1.2, 1.),
            },
        ));
    }
    let bird_mesh = meshes.add(Cuboid::new(0.14, 0.022, 0.045));
    let bird_mat = materials.add(Color::srgb(0.33, 0.40, 0.35));
    for i in 0..3 {
        let at = Vec3::new(0., 4.7, -3.5);
        commands
            .spawn((
                Transform::from_translation(at),
                Visibility::Inherited,
                Drift {
                    kind: 3,
                    seed: i as f32 * 1.9,
                    origin: at,
                },
            ))
            .with_children(|p| {
                for sign in [-1., 1.] {
                    p.spawn((
                        Mesh3d(bird_mesh.clone()),
                        MeshMaterial3d(bird_mat.clone()),
                        Transform::from_xyz(sign * 0.065, 0., 0.)
                            .with_rotation(Quat::from_rotation_z(sign * 0.45)),
                    ));
                }
            });
    }
    for i in 0..8 {
        commands.spawn((
            Mesh3d(dot.clone()),
            MeshMaterial3d(gold.clone()),
            Transform::default(),
            Visibility::Hidden,
            Drift {
                kind: 4,
                seed: i as f32,
                origin: Vec3::ZERO,
            },
        ));
    }
    commands.spawn((
        Mesh3d(meshes.add(Torus::new(0.27, 0.30))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.51, 0.43),
            emissive: LinearRgba::rgb(0.02, 0.12, 0.08),
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden,
        Drift {
            kind: 5,
            seed: 0.,
            origin: Vec3::ZERO,
        },
    ));
    let tear = materials.add(Color::srgb(0.36, 0.68, 0.85));
    for seed in [-1., 1.] {
        commands.spawn((
            Mesh3d(dot.clone()),
            MeshMaterial3d(tear.clone()),
            Transform::default(),
            Visibility::Hidden,
            Drift {
                kind: 6,
                seed,
                origin: Vec3::ZERO,
            },
        ));
    }
}

pub fn direct(
    time: Res<Time<Real>>,
    game: Res<Session>,
    travel: Res<Travel>,
    mut d: ResMut<Director>,
    player: Single<&Transform, With<Detective>>,
    mut camera: Single<(&mut Transform, &mut Projection), (With<WorldCamera>, Without<Detective>)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    mut commands: Commands,
    effects: Res<Effects>,
    assets: Res<AssetServer>,
) {
    let dt = time.delta_secs().min(0.05);
    let entered = d.phase != game.0.phase;
    let scene_started = d.age <= 0.
        && game.0.scene_elapsed > 0.
        && matches!(game.0.phase, Phase::Story(_) | Phase::Reunion);
    if entered {
        d.phase = game.0.phase.clone();
        d.age = 0.;
        d.overview = false;
        d.zoom = 0.;
        d.bond_sound = None;
    }
    if matches!(game.0.phase, Phase::Story(_) | Phase::Reunion) {
        d.age = game.0.scene_elapsed;
    } else if game.0.modal != Modal::Pause
        && (game.0.phase != Phase::Reunion || game.0.modal == Modal::None)
    {
        d.age += dt;
    }
    d.celebrate = (d.celebrate - dt).max(0.);
    if game.0.chapter > d.chapter {
        d.celebrate = 2.6;
    }
    d.chapter = game.0.chapter;
    for id in 0..8 {
        if game.0.found[id] && !d.found[id] {
            let c = &CLUES[id];
            burst(
                &mut commands,
                &effects,
                Vec3::new(c.position[0], 0.6, c.position[1]),
                false,
            );
        }
    }
    if !game.0.typing() {
        d.shown_trust = game.0.trust;
    }
    if game.0.phase == Phase::Story(0)
        && game.0.bonding.active
        && game.0.bonding.elapsed > 0.
        && d.bond_sound != Some(game.0.bonding.completed)
    {
        d.bond_sound = Some(game.0.bonding.completed);
        if !game.0.muted {
            commands.spawn((
                AudioPlayer::new(assets.load("audio/meow.wav")),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.35)),
            ));
        }
    }
    if scene_started && !game.0.muted {
        let sound = match game.0.phase {
            Phase::Story(1) => Some("meow"),
            Phase::Story(3) => Some("discover"),
            Phase::Reunion => Some("reunion"),
            _ => None,
        };
        if let Some(sound) = sound {
            commands.spawn((
                AudioPlayer::new(assets.load(format!("audio/{sound}.wav"))),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.35)),
            ));
        }
    }
    if scene_started && game.0.phase == Phase::Reunion {
        burst(
            &mut commands,
            &effects,
            player.translation + Vec3::Y * 1.1,
            true,
        );
    }
    d.found = game.0.found;
    if game.0.phase == Phase::Search && game.0.modal == Modal::None {
        if keys.just_pressed(KeyCode::Tab) {
            d.overview = !d.overview;
            d.zoom = 0.;
        }
        for e in wheel.read() {
            let amount = if e.unit == MouseScrollUnit::Line {
                e.y * 0.8
            } else {
                e.y * 0.012
            };
            d.zoom = (d.zoom - amount).clamp(-3.5, 5.);
        }
    } else {
        wheel.clear();
    }
    let (mut target, mut height, mut yaw) = match game.0.phase {
        Phase::Title => (
            Vec3::new(-5.35, 0.65, -0.5),
            4.2,
            0.18 + (time.elapsed_secs() * 0.13).sin() * 0.055,
        ),
        Phase::Story(0) => {
            let cuddling = game.0.bonding.stage() == 2;
            (
                Vec3::new(if cuddling { -3.5 } else { -3.9 }, 0.52, -0.55),
                if cuddling { 3.2 } else { 3.8 },
                0.20 + (time.elapsed_secs() * 0.2).sin() * 0.035,
            )
        }
        Phase::Story(1) => {
            let s = (d.age / 1.4).min(1.);
            (
                Vec3::new(-3.6 + 0.8 * s, 0.55, -0.4 - 1.2 * s),
                5.4,
                0.28 + 0.2 * s,
            )
        }
        Phase::Story(2) => (Vec3::new(-3.1, 0.7, -0.7), 5.7, 0.64),
        Phase::Story(3) => (Vec3::new(-2.5, 1.1, -0.65), 5.7, 0.25),
        Phase::Story(_) => (Vec3::new(-0.8, 0.4, 0.), 11. + d.age.min(3.) * 1.4, 0.44),
        Phase::Search => {
            if d.overview {
                (Vec3::new(0., 0.3, 0.2), 18.5, 0.47)
            } else {
                let mut target = Vec3::new(
                    player.translation.x * 0.17,
                    0.3,
                    player.translation.z * 0.12,
                );
                let mut height = 15.7;
                if let Some(id) = travel.inspect {
                    target = target.lerp(
                        Vec3::new(CLUES[id].position[0], 0.4, CLUES[id].position[1]),
                        0.32,
                    );
                    height = 13.8;
                }
                (target, height + d.zoom, 0.47)
            }
        }
        Phase::Reunion => {
            let yaw: f32 = 0.18 + smooth((d.age - 6.) / 16.) * 0.25;
            let right = Vec3::new(yaw.cos(), 0., -yaw.sin());
            (
                player.translation + Vec3::new(0.4, 0.65, 0.3) - right * 1.05,
                if d.age < 6. {
                    6.0
                } else if d.age < 22. {
                    3.6
                } else {
                    4.6
                },
                yaw,
            )
        }
    };
    match game.0.modal {
        Modal::Clue(id) => {
            yaw = [0.18, 0.45, 0.15, 0.5, 0.47, -0.13, 0.25, -0.10][id];
            let right = Vec3::new(yaw.cos(), 0., -yaw.sin());
            let p = CLUES[id].position;
            target = Vec3::new(p[0], 0.65, p[1]) + right * 1.8;
            height = if id == 5 { 4.6 } else { 7.8 };
        }
        Modal::Rescue => {
            yaw = -0.13;
            target = Vec3::new(4.7, 0.40, 3.45) + Vec3::new(yaw.cos(), 0., -yaw.sin()) * 1.1;
            height = 4.4;
        }
        Modal::Deduction | Modal::Conclusion(_) | Modal::Journal => {
            target = Vec3::new(0., 0.4, 0.2);
            height = 18.;
            yaw = 0.47;
        }
        _ => {}
    }
    let ease = 1. - (-dt * 3.0).exp();
    d.target = d.target.lerp(target, ease);
    d.height += (height - d.height) * ease;
    d.yaw += (yaw - d.yaw) * ease;
    let offset = Vec3::new(d.yaw.sin(), 0.71, d.yaw.cos()).normalize() * 30.;
    *camera.0 = Transform::from_translation(d.target + offset).looking_at(d.target, Vec3::Y);
    if let Projection::Orthographic(ref mut p) = *camera.1 {
        p.scaling_mode = ScalingMode::FixedVertical {
            viewport_height: d.height,
        };
    }
}
fn burst(commands: &mut Commands, e: &Effects, at: Vec3, happy: bool) {
    for i in 0..if happy { 28 } else { 12 } {
        let a = i as f32 * 2.39996;
        commands.spawn((
            Mesh3d(e.mesh.clone()),
            MeshMaterial3d(if happy && i % 2 == 0 {
                e.pink.clone()
            } else {
                e.gold.clone()
            }),
            Transform::from_translation(at).with_scale(Vec3::splat(if happy { 1.9 } else { 1.2 })),
            Spark {
                velocity: Vec3::new(a.sin() * 0.9, 0.7 + (a * 2.).cos().abs(), a.cos() * 0.9),
                life: if happy { 2.8 } else { 1.2 },
                age: 0.,
            },
        ));
    }
}

pub fn music(
    time: Res<Time<Real>>,
    game: Res<Session>,
    mut music: Query<&mut AudioSink, With<Music>>,
) {
    let target = if game.0.muted {
        0.
    } else {
        match game.0.phase {
            Phase::Story(1) => 0.15,
            Phase::Story(2) => 0.10,
            Phase::Story(3) => 0.20,
            Phase::Reunion => 0.30,
            _ => 0.26,
        }
    };
    let a = 1. - (-time.delta_secs() * 2.).exp();
    for mut sink in &mut music {
        let current = sink.volume().to_linear();
        sink.set_volume(Volume::Linear(current + (target - current) * a));
    }
}

pub fn rig_parts(mut commands: Commands, parts: Query<(Entity, &Name, &Transform), Added<Name>>) {
    for (e, name, base) in &parts {
        let n = name.as_str();
        let kind = if n.starts_with("CatEyeRig") {
            0
        } else if n.starts_with("CatHeadRig") {
            1
        } else if n.starts_with("CatTailRig") {
            2
        } else if n.starts_with("LegRigL") {
            3
        } else if n.starts_with("LegRigR") {
            4
        } else if n.starts_with("ArmRigL") {
            5
        } else if n.starts_with("ArmRigR") {
            6
        } else if n.starts_with("DetectiveHeadRig") {
            7
        } else {
            continue;
        };
        commands.entity(e).insert(LivingPart { base: *base, kind });
    }
}
pub fn animate_parts(
    time: Res<Time<Real>>,
    game: Res<Session>,
    director: Res<Director>,
    travel: Res<Travel>,
    keys: Res<ButtonInput<KeyCode>>,
    mut parts: Query<(&LivingPart, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    let walking = game.0.phase == Phase::Search
        && game.0.modal == Modal::None
        && (!travel.path.is_empty()
            || [
                KeyCode::KeyW,
                KeyCode::KeyA,
                KeyCode::KeyS,
                KeyCode::KeyD,
                KeyCode::ArrowUp,
                KeyCode::ArrowDown,
                KeyCode::ArrowLeft,
                KeyCode::ArrowRight,
            ]
            .iter()
            .any(|k| keys.pressed(*k)))
        || game.0.phase == Phase::Reunion && director.age < 6.;
    let stride = if walking { (t * 11.).sin() * 0.36 } else { 0. };
    for (part, mut tr) in &mut parts {
        *tr = part.base;
        match part.kind {
            0 => {
                let phase = t % 5.4;
                let blink = if phase > 4.9 && phase < 5.15 {
                    1. - ((phase - 4.9) / 0.25 * std::f32::consts::PI).sin() * 0.94
                } else {
                    1.
                };
                tr.scale.y *= blink;
            }
            1 => {
                let cuddling = game.0.phase == Phase::Story(0) && game.0.bonding.stage() == 2;
                tr.rotate_z(
                    (t * if cuddling { 2.4 } else { 0.7 }).sin()
                        * if cuddling { 0.10 } else { 0.035 },
                );
            }
            2 => tr.rotate_y(
                (t * 1.6).sin()
                    * if game.0.phase == Phase::Story(0) {
                        0.35
                    } else {
                        0.20
                    },
            ),
            3 => tr.rotate_x(stride),
            4 => tr.rotate_x(-stride),
            5 | 6 => {
                if game.0.phase == Phase::Story(0) {
                    let petting = game.0.bonding.stage() == 2
                        && (game.0.bonding.active || game.0.bonding.ready())
                        && !game.0.typing();
                    if part.kind == 6 {
                        tr.rotate_x(if petting { -1.0 } else { -0.55 });
                        tr.rotate_z(if petting {
                            0.4 + (t * 2.4).sin() * 0.12
                        } else {
                            (game.0.bonding.elapsed * 3.).sin() * 0.22
                        });
                    }
                } else if game.0.phase == Phase::Story(2) && director.age > 0. {
                    tr.rotate_x(-1.9);
                    tr.rotate_z(if part.kind == 5 { 0.28 } else { -0.28 });
                } else if game.0.phase == Phase::Story(4) && director.age > 0. && part.kind == 6 {
                    tr.rotate_z(0.7 + (t * 2.).sin() * 0.12);
                } else if game.0.phase == Phase::Reunion && director.age >= 6. && part.kind == 6 {
                    tr.rotate_x(-0.65);
                    tr.rotate_z(0.35 + (t * 1.8).sin() * 0.10);
                } else {
                    tr.rotate_x(if part.kind == 6 { stride } else { -stride });
                }
            }
            7 => {
                if game.0.phase == Phase::Story(2) && director.age > 0. {
                    tr.rotate_x(0.20)
                }
            }
            _ => {}
        }
    }
}
pub fn ambience(
    time: Res<Time<Real>>,
    game: Res<Session>,
    travel: Res<Travel>,
    director: Res<Director>,
    mut commands: Commands,
    mut player: Single<&mut Transform, With<Detective>>,
    mut drift: Query<(&Drift, &mut Transform, &mut Visibility), Without<Detective>>,
    mut sparks: Query<(Entity, &mut Spark, &mut Transform), (Without<Drift>, Without<Detective>)>,
) {
    let t = time.elapsed_secs();
    let dt = time.delta_secs().min(0.05);
    if game.0.phase == Phase::Story(0) {
        let petting = game.0.bonding.stage() == 2;
        player.translation.y +=
            ((if petting { 0.03 } else { 0.15 }) - player.translation.y) * (1. - (-dt * 3.).exp());
        player.rotation = player.rotation.slerp(
            Quat::from_rotation_y(if petting { -1.0 } else { -0.6 }),
            1. - (-dt * 3.).exp(),
        );
    } else if matches!(game.0.phase, Phase::Story(_)) {
        player.translation.y += (0.15 - player.translation.y) * (1. - (-dt * 3.).exp());
    }
    if game.0.phase == Phase::Reunion && !game.0.typing() {
        let at = Vec3::new(-3.4, if director.age >= 15. { 0.03 } else { 0.15 }, -0.6);
        player.translation = player.translation.lerp(at, 1. - (-dt * 0.9).exp());
        let facing = if player.translation.distance(at) > 0.15 {
            -1.9
        } else {
            0.15
        };
        player.rotation = player
            .rotation
            .slerp(Quat::from_rotation_y(facing), 1. - (-dt * 2.).exp());
    }
    for (d, mut tr, mut vis) in &mut drift {
        *vis = Visibility::Visible;
        match d.kind {
            0 => {
                tr.translation = d.origin
                    + Vec3::new(
                        (t * 0.3 + d.seed).sin() * 0.15,
                        (t * 0.7 + d.seed).sin() * 0.17,
                        0.,
                    );
                tr.scale = Vec3::splat(0.6 + (t + d.seed).sin().abs() * 0.7);
            }
            1 => {
                tr.translation = d.origin
                    + Vec3::new(
                        (t * 0.8 + d.seed).sin() * 0.5,
                        (t * 1.7 + d.seed).sin() * 0.18,
                        (t * 0.6 + d.seed).cos() * 0.35,
                    );
                tr.rotation = Quat::from_rotation_y(t * 0.7 + d.seed)
                    * Quat::from_rotation_z((t * 24. + d.seed).sin() * 0.3);
            }
            2 => {
                let s = (t * 0.9 + d.seed) % 1.;
                let a = d.seed * 2.4;
                tr.translation = d.origin
                    + Vec3::new(a.cos() * s * 0.48, 0.35 - s * s * 1.05, a.sin() * s * 0.48);
            }
            3 => {
                tr.translation = d.origin
                    + Vec3::new(
                        (t * 0.10 + d.seed).sin() * 5.,
                        (t * 1.3 + d.seed).sin() * 0.1,
                        (t * 0.10 + d.seed).cos() * 1.3,
                    );
                tr.rotation = Quat::from_rotation_y(-t * 0.10 - d.seed)
                    * Quat::from_rotation_x((t * 5. + d.seed).sin() * 0.2);
            }
            4 => {
                if game.0.phase == Phase::Search
                    && !travel.path.is_empty()
                    && game.0.modal == Modal::None
                {
                    let index = ((travel.path.len() - 1) as f32 * (d.seed + 1.) / 8.) as usize;
                    if let Some(p) = travel.path.get(index) {
                        tr.translation = Vec3::new(p.x, 0.22, p.y);
                        tr.scale = Vec3::splat(1.3);
                    }
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            5 => {
                tr.translation = Vec3::new(player.translation.x, 0.19, player.translation.z);
                if game.0.phase != Phase::Search {
                    *vis = Visibility::Hidden;
                }
            }
            6 => {
                if game.0.phase == Phase::Story(2) && director.age > 0. {
                    let u = (t * 1.5 + d.seed * 0.2).fract().abs();
                    tr.translation =
                        player.translation + Vec3::new(d.seed * 0.065, 0.81 - u * 0.17, 0.17);
                    tr.scale = Vec3::new(0.65, 1.15, 0.65);
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            _ => {}
        }
    }
    for (e, mut spark, mut tr) in &mut sparks {
        spark.age += dt;
        if spark.age > spark.life {
            commands.entity(e).despawn();
            continue;
        }
        spark.velocity.y -= dt * 0.65;
        tr.translation += spark.velocity * dt;
        tr.scale = Vec3::splat((1. - spark.age / spark.life) * 1.5);
    }
    let _ = director;
}
