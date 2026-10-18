use crate::*;
use bevy::text::{FontSmoothing, LineHeight};

const INK: Color = Color::srgb(0.09, 0.17, 0.15);
const SECONDARY: Color = Color::srgb(0.28, 0.34, 0.30);
const PAPER: Color = Color::srgb(0.99, 0.975, 0.94);
const SOFT: Color = Color::srgb(0.92, 0.925, 0.87);
const GREEN: Color = Color::srgb(0.15, 0.30, 0.25);
const ACCENT: Color = Color::srgb(0.51, 0.27, 0.11);
const LINE: Color = Color::srgb(0.77, 0.80, 0.72);

#[derive(Component)]
pub struct UiRoot;
#[derive(Component)]
pub struct Click(pub Action);
#[derive(Component)]
pub struct ButtonBase(pub bool);
#[derive(Component)]
pub struct Marker(pub usize);
#[derive(Component)]
pub struct MarkerTooltip(pub usize);
#[derive(Component)]
pub struct NearText;
#[derive(Component)]
pub struct NearButton;
#[derive(Component)]
pub struct UiPanel(pub &'static str);

fn column(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(gap),
        ..default()
    }
}
fn label(
    p: &mut ChildSpawnerCommands,
    m: &Media,
    s: impl Into<String>,
    size: f32,
    color: Color,
    bold: bool,
) {
    p.spawn((
        Text::new(s),
        TextFont {
            font: if bold { m.bold.clone() } else { m.font.clone() },
            font_size: size,
            line_height: LineHeight::RelativeToFont(1.4),
            font_smoothing: FontSmoothing::AntiAliased,
            ..default()
        },
        TextColor(color),
    ));
}
fn button(
    p: &mut ChildSpawnerCommands,
    m: &Media,
    s: impl Into<String>,
    action: Action,
    primary: bool,
) {
    p.spawn((
        Button,
        Click(action),
        ButtonBase(primary),
        Node {
            min_height: px(48),
            padding: UiRect::axes(px(18), px(11)),
            border: UiRect::all(px(1)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_shrink: 0.,
            ..default()
        },
        BorderColor::all(if primary { GREEN } else { LINE }),
        BorderRadius::all(px(12)),
        BackgroundColor(if primary { GREEN } else { SOFT }),
    ))
    .with_children(|p| label(p, m, s, 17., if primary { PAPER } else { INK }, true));
}
fn rule(p: &mut ChildSpawnerCommands) {
    p.spawn((
        Node {
            width: percent(100),
            height: px(1),
            flex_shrink: 0.,
            ..default()
        },
        BackgroundColor(LINE),
    ));
}
fn panel(n: Node, name: &'static str) -> impl Bundle {
    (
        n,
        UiPanel(name),
        BackgroundColor(PAPER),
        BorderRadius::all(px(20)),
    )
}
fn photo(p: &mut ChildSpawnerCommands, m: &Media, size: f32) {
    p.spawn((
        Node {
            width: px(size),
            height: px(size),
            overflow: Overflow::clip(),
            flex_shrink: 0.,
            ..default()
        },
        BorderRadius::all(px(12)),
    ))
    .with_children(|p| {
        p.spawn((
            ImageNode::new(m.photo.clone()),
            Node {
                width: px(size),
                height: px(size),
                ..default()
            },
        ));
    });
}
fn progress(p: &mut ChildSpawnerCommands, completed: usize) {
    p.spawn(Node {
        column_gap: px(6),
        width: percent(100),
        ..default()
    })
    .with_children(|p| {
        for i in 0..3 {
            p.spawn((
                Node {
                    flex_grow: 1.,
                    height: px(5),
                    ..default()
                },
                BackgroundColor(if completed > i { GREEN } else { LINE }),
                BorderRadius::MAX,
            ));
        }
    });
}
pub fn buttons(
    mut q: Query<
        (
            &Interaction,
            &Click,
            Option<&ButtonBase>,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
    mut actions: ResMut<Actions>,
) {
    for (i, action, base, mut bg) in &mut q {
        let filled = base.is_some_and(|b| b.0);
        *bg = BackgroundColor(match i {
            Interaction::Pressed => {
                actions.0.push_back(action.0.clone());
                Color::srgb(0.37, 0.27, 0.13)
            }
            Interaction::Hovered => {
                if filled {
                    Color::srgb(0.22, 0.39, 0.30)
                } else {
                    Color::srgb(0.81, 0.86, 0.75)
                }
            }
            Interaction::None => {
                if filled {
                    GREEN
                } else {
                    SOFT
                }
            }
        });
    }
}
pub fn rebuild(
    mut commands: Commands,
    game: Res<Session>,
    media: Res<Media>,
    files: Res<Files>,
    director: Res<presentation::Director>,
    old: Query<Entity, With<UiRoot>>,
    mut last: Local<Option<(u64, u32, u32, bool, bool)>>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
) {
    let c = &game.0;
    let key = (
        c.revision,
        (window.width() / ui_scale.0).round() as u32,
        (window.height() / ui_scale.0).round() as u32,
        director.age > 3.,
        director.celebrate > 0.,
    );
    if *last == Some(key) {
        return;
    }
    *last = Some(key);
    for e in &old {
        commands.entity(e).despawn();
    }
    let w = window.width() / ui_scale.0;
    let h = window.height() / ui_scale.0;
    commands
        .spawn((
            UiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ))
        .with_children(|root| {
            match &c.phase {
                Phase::Title => title(root, &media, files.load.is_some(), h),
                Phase::Story(n) => story(root, &media, *n, w),
                Phase::Search => field_hud(root, &media, c, w, director.celebrate > 0.),
                Phase::Reunion => {
                    if director.age > 3. {
                        ending(root, &media, c, h)
                    } else {
                        caption(root, &media, "율아, 기다려 줘서 고마워. 우리 집에 가자.");
                    }
                }
            }
            if c.modal != Modal::None {
                match c.modal {
                    Modal::Clue(id) => evidence(root, &media, id, h),
                    Modal::Rescue => rescue(root, &media, c, h),
                    _ => {
                        root.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: percent(100),
                                height: percent(100),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                padding: UiRect::all(px(24)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.045, 0.10, 0.08, 0.65)),
                            ZIndex(30),
                        ))
                        .with_children(|p| {
                            let width: f32 = if matches!(c.modal, Modal::Journal | Modal::Deduction)
                            {
                                920.
                            } else {
                                650.
                            };
                            p.spawn(panel(
                                Node {
                                    width: px(width.min(w - 48.)),
                                    max_height: px(h - 48.),
                                    padding: UiRect::all(px(28)),
                                    row_gap: px(16),
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                },
                                "modal",
                            ))
                            .with_children(|p| match c.modal {
                                Modal::Journal => journal(p, &media, c),
                                Modal::Deduction if c.chapter < 3 => deduction(p, &media, c, h),
                                Modal::Help => help(p, &media),
                                Modal::Pause => pause(p, &media),
                                _ => button(p, &media, "돌아가기", Action::Close, true),
                            });
                        });
                    }
                }
            }
        });
}
fn title(root: &mut ChildSpawnerCommands, m: &Media, saved: bool, h: f32) {
    root.spawn(panel(
        Node {
            position_type: PositionType::Absolute,
            left: px(48),
            top: px((h * 0.18).max(110.)),
            width: px(438),
            padding: UiRect::all(px(32)),
            row_gap: px(21),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        "title",
    ))
    .with_children(|p| {
        label(p, m, "고양이 탐정 이야기", 17., ACCENT, true);
        label(p, m, "오늘도,\n너에게", 52., INK, true);
        label(
            p,
            m,
            "율의 작은 발자국을 따라,\n네 마음에 한 걸음 가까이.",
            21.,
            INK,
            false,
        );
        rule(p);
        button(p, m, "새로운 이야기 시작     →", Action::Start, true);
        if saved {
            button(p, m, "이어서 찾기", Action::Continue, false);
        }
        button(p, m, "조작 방법", Action::Help, false);
    });
}
fn story(root: &mut ChildSpawnerCommands, m: &Media, n: usize, w: f32) {
    root.spawn(panel(
        Node {
            position_type: PositionType::Absolute,
            left: px(32),
            top: px(26),
            padding: UiRect::axes(px(19), px(12)),
            ..default()
        },
        "chapter",
    ))
    .with_children(|p| label(p, m, format!("우리들의 오후   {}/5", n + 1), 16., INK, true));
    if n == 0 {
        root.spawn(Node {
            position_type: PositionType::Absolute,
            right: px(32),
            top: px(26),
            ..default()
        })
        .with_children(|p| button(p, m, "장난감 흔들기  [Space]", Action::Play, true));
    }
    root.spawn(panel(
        Node {
            position_type: PositionType::Absolute,
            left: px(32),
            right: px(32),
            bottom: px(28),
            padding: UiRect::all(px(25)),
            column_gap: px(28),
            align_items: AlignItems::Center,
            ..default()
        },
        "story",
    ))
    .with_children(|p| {
        p.spawn(Node {
            width: px(w - 400.),
            flex_grow: 1.,
            flex_direction: FlexDirection::Column,
            row_gap: px(9),
            ..default()
        })
        .with_children(|p| {
            label(p, m, STORY[n].0, 27., INK, true);
            label(p, m, STORY[n].1, 19., INK, false);
        });
        p.spawn(Node {
            width: px(220),
            flex_shrink: 0.,
            ..default()
        })
        .with_children(|p| {
            button(
                p,
                m,
                if n == 4 {
                    "율을 찾으러 가기\n[Enter]"
                } else {
                    "다음 이야기  →\n[Enter]"
                },
                Action::Next,
                true,
            );
        });
    });
}
fn caption(root: &mut ChildSpawnerCommands, m: &Media, s: &str) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(38),
            left: px(40),
            right: px(40),
            padding: UiRect::all(px(23)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(GREEN),
        BorderRadius::all(px(14)),
    ))
    .with_children(|p| label(p, m, s, 23., PAPER, true));
}
fn field_hud(root: &mut ChildSpawnerCommands, m: &Media, c: &Case, w: f32, celebrate: bool) {
    root.spawn(panel(
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            top: px(24),
            width: px(350),
            padding: UiRect::all(px(19)),
            row_gap: px(9),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        "objective",
    ))
    .with_children(|p| {
        label(
            p,
            m,
            format!(
                "{} / 3   {}",
                c.chapter.min(2) + 1,
                ["첫 발자국", "숨은 마음", "다시, 함께"][c.chapter.min(2)]
            ),
            16.,
            ACCENT,
            true,
        );
        label(
            p,
            m,
            [
                "집 주변의 흔적을 모아\n첫 수색 범위를 추리하세요.",
                "분수와 온실의 흔적을 연결해\n은신처를 추리하세요.",
                "온실에서 율을 살펴보고,\n익숙한 담요를 찾아주세요.",
                "온실로 돌아가\n율을 안심시켜 주세요.",
            ][c.chapter],
            18.,
            INK,
            true,
        );
        progress(p, c.chapter);
    });
    root.spawn(panel(
        Node {
            position_type: PositionType::Absolute,
            right: px(24),
            top: px(24),
            padding: UiRect::all(px(14)),
            column_gap: px(12),
            align_items: AlignItems::Center,
            ..default()
        },
        "case",
    ))
    .with_children(|p| {
        photo(p, m, 54.);
        p.spawn(column(3.)).with_children(|p| {
            label(p, m, format!("단서  {} / 8", c.count()), 18., INK, true);
            label(p, m, format!("{CAT_NAME}을 찾는 중"), 14., SECONDARY, false);
        });
        button(p, m, "메뉴  [Esc]", Action::Pause, false);
    });
    if celebrate {
        root.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(190),
                left: percent(30),
                right: percent(30),
                padding: UiRect::all(px(17)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(GREEN),
            BorderRadius::all(px(14)),
            ZIndex(8),
        ))
        .with_children(|p| {
            label(
                p,
                m,
                "추리 성공! 한 걸음 더 가까워졌어요.",
                20.,
                PAPER,
                true,
            )
        });
    }
    if c.modal != Modal::None {
        return;
    }
    for (id, _) in CLUES.iter().enumerate().filter(|(id, _)| c.available(*id)) {
        root.spawn((
            Marker(id),
            Button,
            Click(Action::Inspect(id)),
            ButtonBase(true),
            Node {
                position_type: PositionType::Absolute,
                width: px(42),
                height: px(42),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(3)),
                ..default()
            },
            BorderColor::all(PAPER),
            BorderRadius::MAX,
            BackgroundColor(GREEN),
            ZIndex(3),
        ))
        .with_children(|p| {
            label(p, m, format!("{}", id + 1), 17., PAPER, true);
            p.spawn((
                MarkerTooltip(id),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(48),
                    top: px(-1),
                    padding: UiRect::axes(px(12), px(8)),
                    min_width: px(140),
                    ..default()
                },
                BackgroundColor(PAPER),
                BorderRadius::all(px(10)),
                Visibility::Hidden,
            ))
            .with_children(|p| label(p, m, CLUES[id].place, 16., INK, true));
        });
    }
    root.spawn(panel(
        Node {
            position_type: PositionType::Absolute,
            bottom: px(24),
            left: px((w - (w - 48.).min(1140.)) * 0.5),
            width: px((w - 48.).min(1140.)),
            padding: UiRect::all(px(18)),
            row_gap: px(12),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        "field-dock",
    ))
    .with_children(|p| {
        p.spawn(Node {
            column_gap: px(12),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|p| {
            label(p, m, "모리", 17., ACCENT, true);
            let limit = if w < 1200. { 67 } else { 92 };
            let msg = if c.message.chars().count() > limit {
                format!("{}…", c.message.chars().take(limit).collect::<String>())
            } else {
                c.message.clone()
            };
            label(
                p,
                m,
                msg.strip_prefix("모리 · ").unwrap_or(&msg),
                17.,
                INK,
                false,
            );
        });
        rule(p);
        p.spawn(Node {
            column_gap: px(10),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|p| {
            button(
                p,
                m,
                format!("단서 수첩  {} / 8  [J]", c.count()),
                Action::Journal,
                false,
            );
            if c.chapter < 3 {
                button(
                    p,
                    m,
                    if c.ready() {
                        "단서 연결하기  [R]"
                    } else {
                        "추리하기  [R]"
                    },
                    Action::Deduce,
                    c.ready(),
                );
            } else {
                button(p, m, "아이에게 돌아가기", Action::Inspect(5), true);
            }
            button(p, m, "모리의 도움  [Q]", Action::Hint, false);
            p.spawn(Node {
                flex_grow: 1.,
                ..default()
            });
            p.spawn((
                NearButton,
                Button,
                Click(Action::Inspect(0)),
                ButtonBase(true),
                Node {
                    min_height: px(48),
                    min_width: px(150),
                    padding: UiRect::axes(px(16), px(10)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(GREEN),
                BorderRadius::all(px(12)),
            ))
            .with_children(|p| {
                p.spawn((
                    NearText,
                    Text::new("가까이 가서 조사 [E]"),
                    TextFont {
                        font: m.bold.clone(),
                        font_size: 16.,
                        font_smoothing: FontSmoothing::AntiAliased,
                        ..default()
                    },
                    TextColor(PAPER),
                ));
            });
        });
    });
}
fn evidence(root: &mut ChildSpawnerCommands, m: &Media, id: usize, h: f32) {
    let clue = &CLUES[id];
    let compact = h < 830.;
    root.spawn((
        panel(
            Node {
                position_type: PositionType::Absolute,
                right: px(28),
                top: px(142),
                width: px(452),
                max_height: px(h - 170.),
                padding: UiRect::all(px(if compact { 20. } else { 27. })),
                row_gap: px(if compact { 12. } else { 17. }),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            "evidence",
        ),
        ZIndex(20),
    ))
    .with_children(|p| {
        label(
            p,
            m,
            format!("단서 {:02}  ·  {}", id + 1, clue.place),
            16.,
            ACCENT,
            true,
        );
        label(p, m, clue.title, if compact { 25. } else { 27. }, INK, true);
        rule(p);
        label(
            p,
            m,
            clue.detail,
            if compact { 17. } else { 18. },
            INK,
            false,
        );
        p.spawn((
            Node {
                padding: UiRect::all(px(if compact { 13. } else { 18. })),
                row_gap: px(10),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(SOFT),
            BorderRadius::all(px(12)),
        ))
        .with_children(|p| {
            label(p, m, "새로 알게 된 고양이의 마음", 16., GREEN, true);
            label(
                p,
                m,
                clue.lesson,
                if compact { 16. } else { 17. },
                INK,
                false,
            );
        });
        button(p, m, "기록하고 계속 찾기  [Enter]", Action::Close, true);
    });
}
fn journal(p: &mut ChildSpawnerCommands, m: &Media, c: &Case) {
    p.spawn(Node {
        column_gap: px(18),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|p| {
        photo(p, m, 70.);
        p.spawn(column(4.)).with_children(|p| {
            label(p, m, "탐정 수첩", 29., INK, true);
            label(
                p,
                m,
                "크림빛 무늬 · 하얀 양말 · 동그란 호박색 눈",
                17.,
                SECONDARY,
                false,
            );
        });
    });
    label(
        p,
        m,
        "기록을 누르면 장소와 고양이 습성을 다시 살펴볼 수 있어요.",
        17.,
        INK,
        false,
    );
    p.spawn(Node {
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::flex(2, 1.),
        column_gap: px(12),
        row_gap: px(10),
        ..default()
    })
    .with_children(|p| {
        for (id, clue) in CLUES.iter().enumerate() {
            if c.found[id] {
                button(
                    p,
                    m,
                    format!("{:02}   {}\n{}", id + 1, clue.title, clue.place),
                    Action::Inspect(id),
                    false,
                );
            } else {
                p.spawn((
                    Node {
                        padding: UiRect::all(px(18)),
                        min_height: px(72),
                        ..default()
                    },
                    BackgroundColor(SOFT),
                    BorderRadius::all(px(12)),
                ))
                .with_children(|p| {
                    label(
                        p,
                        m,
                        format!("{:02}   아직 발견하지 못했어요", id + 1),
                        17.,
                        SECONDARY,
                        false,
                    )
                });
            }
        }
    });
    button(p, m, "수첩 닫기  [J / Esc]", Action::Close, true);
}
fn deduction(p: &mut ChildSpawnerCommands, m: &Media, c: &Case, h: f32) {
    label(
        p,
        m,
        format!("추리 {} / 3", c.chapter + 1),
        16.,
        ACCENT,
        true,
    );
    label(p, m, QUESTIONS[c.chapter], 28., INK, true);
    p.spawn(Node {
        column_gap: px(24),
        ..default()
    })
    .with_children(|row| {
        row.spawn(Node {
            width: percent(48),
            row_gap: px(if h < 830. { 9. } else { 12. }),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|p| {
            label(
                p,
                m,
                format!("1. 단서 두 개 선택  ({} / 2)", c.selected.len()),
                18.,
                GREEN,
                true,
            );
            p.spawn(Node {
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(
                    if c.count() > 4 { 2 } else { 1 },
                    1.,
                ),
                column_gap: px(8),
                row_gap: px(9),
                ..default()
            })
            .with_children(|p| {
                for (id, clue) in CLUES.iter().enumerate().filter(|(id, _)| c.found[*id]) {
                    let compact_name = [
                        "문틈의\n크림빛 털",
                        "화분 뒤\n작은 발자국",
                        "빵집 주인의\n목격담",
                        "벤치 아래\n빈 상자",
                        "물가에서\n돌아선 흔적",
                        "잎사귀 뒤\n호박색 눈",
                        "햇살 냄새 나는\n담요",
                        "바람 종이\n들려준 비밀",
                    ][id];
                    button(
                        p,
                        m,
                        format!(
                            "{}  {}",
                            if c.selected.contains(&id) {
                                "●"
                            } else {
                                "○"
                            },
                            if c.count() > 4 {
                                compact_name
                            } else {
                                clue.title
                            }
                        ),
                        Action::Select(id),
                        c.selected.contains(&id),
                    );
                }
            });
            if c.count() == 0 {
                label(
                    p,
                    m,
                    "빛나는 번호를 눌러\n먼저 단서를 찾아주세요.",
                    19.,
                    INK,
                    false,
                );
            }
        });
        row.spawn(Node {
            width: percent(49),
            row_gap: px(13),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|p| {
            label(p, m, "2. 단서에 맞는 생각 선택", 18., GREEN, true);
            for i in 0..3 {
                button(
                    p,
                    m,
                    format!("{}   {}", ["A", "B", "C"][i], THEORIES[c.chapter][i]),
                    Action::Theory(i),
                    c.selected.len() == 2,
                );
            }
            p.spawn((
                Node {
                    padding: UiRect::all(px(17)),
                    ..default()
                },
                BackgroundColor(SOFT),
                BorderRadius::all(px(12)),
            ))
            .with_children(|p| label(p, m, &c.message, 17., INK, false));
        });
    });
    button(p, m, "탐색으로 돌아가기  [Esc]", Action::Close, false);
}
fn rescue(root: &mut ChildSpawnerCommands, m: &Media, c: &Case, h: f32) {
    let compact = h < 830.;
    root.spawn((
        panel(
            Node {
                position_type: PositionType::Absolute,
                right: px(28),
                top: px(142),
                width: px(470),
                max_height: px(h - 170.),
                padding: UiRect::all(px(if compact { 20. } else { 26. })),
                row_gap: px(if compact { 11. } else { 16. }),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            "rescue",
        ),
        ZIndex(20),
    ))
    .with_children(|p| {
        label(
            p,
            m,
            format!("마음의 거리   {} / 3", c.trust),
            17.,
            ACCENT,
            true,
        );
        progress(p, c.trust);
        label(
            p,
            m,
            [
                "율아, 여기서 기다릴게.",
                "집 냄새, 기억하고 있지?",
                "율이 다가올 때까지.",
            ][c.trust.min(2)],
            if compact { 24. } else { 27. },
            INK,
            true,
        );
        label(
            p,
            m,
            &c.message,
            if compact { 17. } else { 18. },
            INK,
            false,
        );
        rule(p);
        let choices = [
            [
                "큰 소리로 “율아!” 하고 부른다",
                "거리를 두고 낮은 자세로 앉는다",
                "잎사귀를 헤치고 손을 뻗는다",
            ],
            [
                "담요를 넣은 이동장을 열어 둔다",
                "품에 안으려고 다가간다",
                "숨어 있는 입구를 막는다",
            ],
            [
                "이동장 안으로 밀어 넣는다",
                "다시 큰 소리로 부른다",
                "조용히 기다려 스스로 다가오게 한다",
            ],
        ];
        for i in 0..3 {
            button(p, m, choices[c.trust.min(2)][i], Action::Rescue(i), false);
        }
        button(p, m, "잠깐 숨 고르기", Action::Close, false);
    });
}
fn help(p: &mut ChildSpawnerCommands, m: &Media) {
    label(p, m, "작은 고양이 탐정 안내서", 28., INK, true);
    label(
        p,
        m,
        "1. 장소 조사  →  2. 단서 연결  →  3. 조심스러운 재회",
        18.,
        GREEN,
        true,
    );
    label(
        p,
        m,
        "동네의 번호를 클릭하면 걸어가서 조사해요.\n직접 걸어가 E를 눌러도 돼요.\n발견한 흔적은 단서 수첩에 기록돼요.",
        18.,
        INK,
        false,
    );
    p.spawn((Node {padding: UiRect::all(px(18)),..default()},BackgroundColor(SOFT),BorderRadius::all(px(12))))
        .with_children(|p|label(p,m,"WASD / 방향키    걷기\nE    가까운 곳 조사\nJ    단서 수첩     R    추리하기\nQ    모리의 안내\nTab    동네 전체 보기     휠    확대 / 축소\nM    소리 켜기 / 끄기     Esc    메뉴 / 닫기",18.,INK,false));
    label(
        p,
        m,
        "시간 제한 없이 천천히 살펴봐요.\n잘못 추리해도 다시 시도할 수 있어요.\n진행은 자동으로 저장돼요.",
        17.,
        SECONDARY,
        false,
    );
    button(p, m, "알겠어요", Action::Close, true);
}
fn pause(p: &mut ChildSpawnerCommands, m: &Media) {
    label(p, m, "잠깐 쉬어 가요", 30., INK, true);
    label(
        p,
        m,
        "찾아가는 길은 서두르지 않아도 괜찮아요.\n진행 상황은 자동으로 저장했어요.",
        19.,
        INK,
        false,
    );
    button(p, m, "계속 찾기  [Esc]", Action::Close, true);
    button(p, m, "조작 방법", Action::Help, false);
    button(p, m, "소리 켜기 / 끄기  [M]", Action::Sound, false);
    button(p, m, "이야기 처음부터", Action::Restart, false);
}
fn ending(root: &mut ChildSpawnerCommands, m: &Media, c: &Case, h: f32) {
    root.spawn(panel(
        Node {
            position_type: PositionType::Absolute,
            left: px(40),
            top: px((h * 0.17).max(95.)),
            width: px(440),
            padding: UiRect::all(px(30)),
            row_gap: px(20),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        "ending",
    ))
    .with_children(|p| {
        label(p, m, "함께 돌아가는 길", 17., ACCENT, true);
        label(p, m, "찾았다.\n율아, 집에 가자.", 38., INK, true);
        label(
            p,
            m,
            "한 발, 또 한 발. 율이 스스로 다가왔다.\n작은 온기가 품 안을 채웠다.",
            19.,
            INK,
            false,
        );
        rule(p);
        label(
            p,
            m,
            format!("마음의 연결  3 / 3     단서  {} / 8", c.count()),
            18.,
            GREEN,
            true,
        );
        label(
            p,
            m,
            if c.count() == 8 {
                "다정한 관찰자 · 모든 흔적을 발견했어요."
            } else {
                "든든한 보호자 · 아이의 마음을 이해했어요."
            },
            17.,
            ACCENT,
            true,
        );
        label(
            p,
            m,
            "사랑은 네가 안심할 만큼\n기다리는 일이기도 해.",
            19.,
            INK,
            false,
        );
        button(p, m, "우리의 오후 다시 만나기", Action::Restart, true);
    });
}
pub fn markers(
    camera: Single<(&Camera, &GlobalTransform), With<WorldCamera>>,
    player: Single<&Transform, With<Detective>>,
    mut nodes: Query<(&Marker, &mut Node, &mut Visibility, &Interaction), Without<MarkerTooltip>>,
    mut tips: Query<(&MarkerTooltip, &mut Visibility), Without<Marker>>,
    game: Res<Session>,
    director: Res<presentation::Director>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
) {
    let near = nearest(&player, &game.0);
    let mut used: Vec<Vec2> = vec![];
    let mut named = vec![];
    for (marker, mut node, mut visibility, interaction) in &mut nodes {
        let clue = &CLUES[marker.0];
        *visibility = Visibility::Hidden;
        let show = !game.0.found[marker.0]
            || near == Some(marker.0)
            || director.overview
            || marker.0 == 5 && game.0.chapter == 3;
        if game.0.modal != Modal::None || !show {
            continue;
        }
        if let Ok(mut pos) = camera.0.world_to_viewport(
            camera.1,
            Vec3::new(clue.position[0], 0.45, clue.position[1]),
        ) {
            // Camera projection is in window points; Node pixels include UiScale.
            pos /= ui_scale.0;
            pos.y -= 34.;
            if pos.x < 30.
                || pos.x > window.width() / ui_scale.0 - 35.
                || pos.y < 178.
                || pos.y > window.height() / ui_scale.0 - 210.
            {
                continue;
            }
            if used.iter().any(|p| p.distance(pos) < 50.) {
                pos.y -= 52.;
            }
            node.left = px(pos.x - 21.);
            node.top = px(pos.y - 21.);
            used.push(pos);
            *visibility = Visibility::Visible;
            if *interaction != Interaction::None || near == Some(marker.0) {
                named.push(marker.0);
            }
        }
    }
    for (tip, mut visibility) in &mut tips {
        *visibility = if named.contains(&tip.0) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
pub fn proximity(
    player: Single<&Transform, With<Detective>>,
    game: Res<Session>,
    mut texts: Query<&mut Text, With<NearText>>,
    mut buttons: Query<(&mut Click, &mut Visibility), With<NearButton>>,
) {
    let near = nearest(&player, &game.0);
    for (mut click, mut visible) in &mut buttons {
        *visible = if near.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if let Some(id) = near {
            click.0 = Action::Inspect(id);
        }
    }
    for mut text in &mut texts {
        if let Some(id) = near {
            text.0 = format!(
                "{} {}  [E]",
                CLUES[id].place,
                if id == 5 && game.0.chapter == 3 {
                    "마음 전하기"
                } else {
                    "조사"
                }
            );
        }
    }
}
#[cfg(test)]
mod tests {
    fn lum(c: [f32; 3]) -> f32 {
        let v = c.map(|x| {
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        });
        v[0] * 0.2126 + v[1] * 0.7152 + v[2] * 0.0722
    }
    #[test]
    fn text_colors_meet_readability_contrast() {
        for (fg, bg) in [
            ([0.09, 0.17, 0.15], [0.99, 0.975, 0.94]),
            ([0.28, 0.34, 0.30], [0.99, 0.975, 0.94]),
            ([0.99, 0.975, 0.94], [0.15, 0.30, 0.25]),
            ([0.51, 0.27, 0.11], [0.99, 0.975, 0.94]),
        ] {
            let a = lum(fg);
            let b = lum(bg);
            assert!((a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5);
        }
    }
}
