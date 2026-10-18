use crate::dialogue::Dialogue;
use serde::{Deserialize, Serialize};

pub const TITLE: &str = "오늘도, 너에게";
pub const SUBTITLE: &str = "A LITTLE CAT DETECTIVE STORY";
pub const CAT_NAME: &str = "율";
pub const REUNION_LINES: [&str; 3] = [
    "율아, 기다려 줘서 고마워. 우리 집에 가자.",
    "“율아, 잡아 봐!” 다시 장난감을 쫓는 작은 앞발. 우리의 오후가 돌아왔다.",
    "익숙한 담요 위, 내 곁에 누운 율. 오늘은 네가 먼저 눈을 감을 때까지 함께 있을게.",
];
pub const BOND_DURATIONS: [f32; 3] = [4.0, 4.0, 4.5];
pub const BOND_ACTIONS: [&str; 3] = [
    "장난감 살살 흔들기",
    "율에게 공 굴려 주기",
    "가까이 앉아 쓰다듬기",
];
pub const BOND_REACTIONS: [&str; 3] = [
    "장난감 끝을 따라 고개를 갸웃하던 율. 작은 앞발로 톡 잡고는 나를 올려다본다.\n“잘 잡네, 율아. 이번엔 공으로 놀까?”",
    "데굴데굴, 공을 따라 폴짝! 공을 붙잡은 율이 내 쪽으로 돌아온다.\n“재미있었어? 이제 잠깐 쉬자. 네가 먼저 다가올 때까지 기다릴게.”",
    "율이 먼저 다가와 손에 이마를 부빈다. 가만히 쓰다듬자 작은 골골송이 들린다.\n“나도 좋아, 율아. 너랑 보내는 이 시간이.”",
];

#[derive(Clone, Debug, Default)]
pub struct Bonding {
    pub completed: usize,
    pub active: bool,
    pub elapsed: f32,
}
impl Bonding {
    pub fn stage(&self) -> usize {
        self.completed.min(2)
    }
    pub fn progress(&self) -> f32 {
        (self.elapsed / BOND_DURATIONS[self.stage()]).clamp(0., 1.)
    }
    pub fn ready(&self) -> bool {
        self.completed >= 3
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Clue {
    pub title: &'static str,
    pub place: &'static str,
    pub position: [f32; 2],
    pub detail: &'static str,
    pub lesson: &'static str,
}

pub const CLUES: [Clue; 8] = [
    Clue {
        title: "문틈의 크림빛 털",
        place: "우리 집 문 앞",
        position: [-5.0, -1.8],
        detail: "문 아래에 크림빛 털과 하얀 털이 함께 걸려 있다. 동그란 호박색 눈, 하얀 양말… 율의 털이 맞아. 멀리 나간 흔적은 아직 없다.",
        lesson: "낯선 곳으로 나온 집고양이는 출구 가까운 은신처에 숨는 경우가 많아요. 먼저 집 주변부터 꼼꼼하게 살펴봐요.",
    },
    Clue {
        title: "화분 뒤 작은 발자국",
        place: "꽃길",
        position: [-2.3, -2.8],
        detail: "흙 위에 아주 작은 발자국. 넓은 길을 가로지르지 않고, 화분과 낮은 담장 안쪽으로 이어진다. 몸을 숨길 수 있는 길을 고른 것 같아.",
        lesson: "불안한 고양이는 탁 트인 곳보다 몸을 가릴 수 있는 가장자리와 틈을 이용할 수 있어요.",
    },
    Clue {
        title: "빵집 주인의 목격담",
        place: "달빛 빵집",
        position: [3.4, -2.8],
        detail: "“크림색 꼬리가 잠깐 보였어요. 제가 다가가니 쏙 숨더라고요. 멀리 뛰어가지는 않았어요.” 빵집 주인이 따뜻한 물 한 잔을 건넨다.",
        lesson: "평소 다정한 고양이도 낯선 야외에서는 보호자에게 바로 다가오지 않을 수 있어요. 사랑이 식은 게 아니라 두려운 거예요.",
    },
    Clue {
        title: "벤치 아래 빈 상자",
        place: "작은 쉼터",
        position: [-4.2, 2.1],
        detail: "아늑한 상자지만 안에는 회색 털만 남아 있다. 율의 크림빛 털과 달라. 이곳은 동네 고양이의 쉼터인 것 같다. 모든 흔적이 율의 것은 아니구나.",
        lesson: "고양이는 좁고 가려진 공간에서 안정감을 느껴요. 다만 털색과 발자국 등 여러 단서를 함께 비교해야 해요.",
    },
    Clue {
        title: "물가에서 돌아선 흔적",
        place: "물빛 분수",
        position: [-0.2, 1.0],
        detail: "젖은 바닥 앞에서 발자국이 방향을 바꿨다. 온실 쪽 마른 돌길로 이어지는 크림빛 털 한 올. 물가가 목적지는 아니었던 것 같다.",
        lesson: "고양이를 찾을 때 소리 하나에만 의존하지 말고, 이동 흔적과 주변 환경을 함께 살펴봐요.",
    },
    Clue {
        title: "잎사귀 뒤 호박색 눈",
        place: "오래된 온실",
        position: [4.7, 2.2],
        detail: "잎사귀 아래에서 동그란 두 눈이 반짝인다. 하얀 가슴, 크림빛 이마, 분홍 코. 율아, 찾았다! 하지만 율은 몸을 낮추고 귀를 뒤로 젖힌 채 움직이지 않는다.",
        lesson: "몸을 낮추고 숨는 모습은 두려움의 신호일 수 있어요. 억지로 끌어내거나 뒤쫓지 말고 안전한 거리를 주세요.",
    },
    Clue {
        title: "햇살 냄새 나는 담요",
        place: "집 앞 빨랫줄",
        position: [-5.4, 0.2],
        detail: "율이 매일 꾹꾹이를 하던 담요다. 조심스럽게 접어 가방에 넣는다. 익숙한 집 냄새가 나. 집으로 돌아오는 길도, 율이 스스로 선택할 수 있게 해 주고 싶어.",
        lesson: "익숙한 담요와 이동장은 안심할 공간을 만드는 데 도움이 될 수 있어요. 먹이를 둔다면 주변 동물과 현장을 함께 살펴봐요.",
    },
    Clue {
        title: "바람 종이 들려준 비밀",
        place: "온실 옆 풍경",
        position: [2.1, 4.0],
        detail: "바람이 불자 종이 딸랑 울린다. 바로 옆 잎사귀도 바스락. 온실 아래는 비를 피할 수 있고, 낮고 어두운 틈이 있다. 숨기에 딱 좋은 곳이야.",
        lesson: "겁먹은 고양이는 가까이에 있어도 울거나 부름에 응답하지 않을 수 있어요. 조용하고 가려진 은신처를 직접 살펴봐요.",
    },
];

pub const STORY: [(&str, &str); 5] = [
    (
        "율과 놀던 오후",
        "“율아, 잡아 봐!” 햇살 한 조각, 장난감 하나.\n네가 앞발을 뻗을 때마다 평범한 하루가 반짝였다.",
    ),
    (
        "아주 잠깐 열린 문",
        "택배를 받으려 문을 연 그 순간, 복도에서 쿵!\n깜짝 놀란 율이 작은 틈 사이로 뛰쳐나갔다.",
    ),
    (
        "율이 없는 집",
        "“율아…” 조심스레 불러도 대답이 없었다.\n미안해, 율아. 눈물이 자꾸 났다. 그런데…",
    ),
    (
        "작은 빛이 말을 걸었다",
        "“나는 고양이 요정 모리. 눈물을 닦아 봐.\n율의 마음을 알면, 발자국도 읽을 수 있어.”",
    ),
    (
        "오늘부터, 고양이 탐정",
        "울고만 있을 순 없지. 수첩과 담요를 챙겼다.\n율아, 네가 좋아하는 세상을 배우며 너에게 갈게.",
    ),
];

pub const QUESTIONS: [&str; 3] = [
    "처음 수색할 곳은 어디일까?",
    "어떤 은신처를 먼저 살필까?",
    "찾아낸 율에게 어떻게 다가갈까?",
];
pub const THEORIES: [[&str; 3]; 3] = [
    [
        "멀리 떠났을 거야. 큰길부터 달려가자",
        "출구 가까이, 몸을 가릴 수 있는 곳",
        "높은 지붕에서 이름을 크게 부르자",
    ],
    [
        "트인 분수 한가운데",
        "시끄러운 빵집 입구",
        "마르고 어두운 온실 아래 틈",
    ],
    [
        "거리를 두고 익숙한 냄새와 기다림을",
        "반가우니까 바로 달려가 안아 주자",
        "숨지 못하도록 입구를 모두 막자",
    ],
];
pub const CORRECT: [usize; 3] = [1, 2, 0];
pub const PAIRS: [[usize; 2]; 3] = [[0, 1], [4, 7], [5, 6]];

#[derive(Clone, Debug, Default)]
pub struct EvidenceView {
    pub preview: Option<usize>,
}
pub const CONNECTIONS: [&str; 3] = [
    "문 앞에 율의 털이 남아 있고, 발자국은 화분과 담장 안쪽으로 이어져요. 멀리 갔다고 단정하기보다 출구 가까운 가려진 곳부터 살펴볼 근거가 돼요.",
    "발자국과 크림빛 털은 분수에서 온실 쪽으로 이어져요. 그 방향에 비를 피할 수 있는 낮고 어두운 틈도 있어요. 이동 방향과 은신처의 조건이 온실에서 만나요.",
    "몸을 낮추고 귀를 뒤로 젖힌 율은 아직 겁먹은 모습이에요. 율이 쓰던 담요의 익숙한 냄새로 안심할 공간을 만들고, 스스로 다가올 시간을 주면 돼요.",
];
pub const NEXT_STEPS: [&str; 3] = [
    "분수의 이동 흔적과 온실 옆 환경을 살펴보세요. 모리의 빛 [Q]가 아직 못 찾은 흔적을 안내해요.",
    "온실 아래를 조사해 율의 모습을 관찰하고, 집 앞 빨랫줄의 담요를 챙겨 주세요.",
    "온실로 돌아가 [E]를 누르세요. 낮은 자세, 익숙한 담요, 조용한 기다림으로 율에게 마음을 전해요.",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Phase {
    Title,
    Story(usize),
    Search,
    Reunion,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Modal {
    #[default]
    None,
    Clue(usize),
    Journal,
    Deduction,
    Conclusion(usize),
    Rescue,
    Help,
    Pause,
}
#[derive(Clone, Debug)]
pub enum Action {
    Play,
    Start,
    Continue,
    Next,
    Inspect(usize),
    Close,
    Journal,
    Deduce,
    Select(usize),
    ViewClue(usize),
    Theory(usize),
    Rescue(usize),
    Hint,
    Help,
    Pause,
    Restart,
    MainMenu,
    FinishDialogue,
    Sound,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Case {
    pub phase: Phase,
    pub found: [bool; 8],
    pub chapter: usize,
    pub trust: usize,
    pub mistakes: usize,
    pub hints: usize,
    pub muted: bool,
    #[serde(skip)]
    pub bonding: Bonding,
    #[serde(skip)]
    pub dialogue: Dialogue,
    #[serde(skip)]
    pub scene_elapsed: f32,
    #[serde(skip)]
    pub modal: Modal,
    #[serde(skip)]
    pub selected: Vec<usize>,
    #[serde(skip)]
    pub evidence: EvidenceView,
    #[serde(skip)]
    pub message: String,
    #[serde(skip)]
    pub revision: u64,
}

impl Default for Case {
    fn default() -> Self {
        Self {
            phase: Phase::Title,
            found: [false; 8],
            chapter: 0,
            trust: 0,
            mistakes: 0,
            hints: 0,
            muted: false,
            bonding: Bonding::default(),
            dialogue: Dialogue::default(),
            scene_elapsed: 0.,
            modal: Modal::None,
            selected: vec![],
            evidence: EvidenceView::default(),
            message: "모리 · 작은 흔적에도 마음이 담겨 있어. 천천히 살펴보자.".into(),
            revision: 0,
        }
    }
}

impl Case {
    pub fn reunion_beat(&self) -> usize {
        if self.scene_elapsed < 6. {
            0
        } else if self.scene_elapsed < 15. {
            1
        } else if self.scene_elapsed < 22. {
            2
        } else {
            3
        }
    }
    pub fn dialogue_text(&self) -> Option<&str> {
        match self.modal {
            Modal::Clue(id) => Some(CLUES[id].detail),
            Modal::Rescue => Some(&self.message),
            Modal::Conclusion(chapter) => Some(CONNECTIONS[chapter]),
            Modal::Deduction if !self.message.is_empty() => Some(&self.message),
            Modal::None => match self.phase {
                Phase::Story(0) => Some(&self.message),
                Phase::Story(n) => Some(STORY[n].1),
                Phase::Reunion => REUNION_LINES.get(self.reunion_beat()).copied(),
                _ => None,
            },
            _ => None,
        }
    }
    pub fn sync_dialogue(&mut self) {
        // Opening a menu pauses a line; closing it resumes that same line.
        if matches!(self.modal, Modal::Pause | Modal::Help) {
            return;
        }
        let text = self.dialogue_text().unwrap_or("").to_owned();
        self.dialogue.set_text(&text);
    }
    pub fn typing(&self) -> bool {
        self.dialogue_text().is_some() && self.dialogue.typing()
    }
    pub fn story_ready(&self) -> bool {
        !self.typing()
            && match self.phase {
                Phase::Story(0) => self.bonding.ready(),
                Phase::Story(1) => self.scene_elapsed >= 1.8,
                Phase::Story(2) => self.scene_elapsed >= 1.0,
                Phase::Story(3) => self.scene_elapsed >= 1.8,
                Phase::Story(4) => self.scene_elapsed >= 1.2,
                _ => false,
            }
    }
    pub fn tick(&mut self, seconds: f32) {
        self.sync_dialogue();
        if matches!(self.modal, Modal::Pause | Modal::Help) {
            return;
        }
        let typing = self.typing();
        self.dialogue.tick(seconds);
        if typing {
            if !self.typing() {
                self.revision = self.revision.wrapping_add(1);
            }
            // A completed line releases scene actions on the following frame.
            return;
        }
        let ready = self.story_ready();
        if self.modal == Modal::None && matches!(self.phase, Phase::Story(_) | Phase::Reunion) {
            self.scene_elapsed += seconds;
        }
        self.tick_bonding(seconds);
        self.sync_dialogue();
        if ready != self.story_ready() {
            self.revision = self.revision.wrapping_add(1);
        }
    }
    pub fn can_play(&self) -> bool {
        self.phase == Phase::Story(0)
            && self.modal == Modal::None
            && !self.bonding.active
            && !self.bonding.ready()
            && !self.typing()
    }
    pub fn tick_bonding(&mut self, seconds: f32) {
        if self.phase != Phase::Story(0)
            || self.modal != Modal::None
            || !self.bonding.active
            || self.typing()
        {
            return;
        }
        self.bonding.elapsed += seconds.max(0.);
        if self.bonding.elapsed >= BOND_DURATIONS[self.bonding.stage()] {
            self.bonding.active = false;
            self.message = BOND_REACTIONS[self.bonding.stage()].into();
            self.bonding.completed += 1;
            self.bonding.elapsed = 0.;
            self.revision = self.revision.wrapping_add(1);
            self.sync_dialogue();
        }
    }
    pub fn count(&self) -> usize {
        self.found.iter().filter(|v| **v).count()
    }
    pub fn available(&self, id: usize) -> bool {
        id < 8 && (id != 5 || self.chapter >= 2)
    }
    pub fn ready(&self) -> bool {
        self.chapter < 3 && PAIRS[self.chapter].iter().all(|i| self.found[*i])
    }
    pub fn objective(&self) -> &str {
        match (&self.phase, self.chapter) {
            (Phase::Reunion, _) => "함께 돌아갈 집이 있다는 것",
            (_, 0) => "집 주변의 흔적을 모아 첫 수색 범위를 추리하세요.",
            (_, 1) => "분수와 온실 주변의 흔적을 연결해 은신처를 추리하세요.",
            (_, 2) => "온실에서 율을 관찰하고, 익숙한 담요를 찾아주세요.",
            _ => "온실로 돌아가 율이 안심할 수 있도록 도와주세요.",
        }
    }
    pub fn apply(&mut self, action: Action) {
        self.sync_dialogue();
        if self.typing()
            && !matches!(
                action,
                Action::FinishDialogue | Action::Help | Action::Pause | Action::Sound
            )
        {
            self.dialogue.finish();
            self.revision = self.revision.wrapping_add(1);
            return;
        }
        let phase = self.phase.clone();
        match action {
            Action::FinishDialogue => self.dialogue.finish(),
            Action::Play if self.can_play() => {
                self.bonding.active = true;
                self.bonding.elapsed = 0.;
                self.message = [
                    "“율아, 여기 봐.” 장난감을 천천히 흔들자 호박색 눈이 반짝인다.\n서두르지 않고, 율이 앞발을 뻗을 때까지 함께 놀아 준다.",
                    "작은 공을 살며시 굴렸다. 율이 몸을 낮추고 엉덩이를 씰룩인다.\n“준비… 됐어? 잡아 봐!”",
                    "놀이를 마치고 손을 낮게 내밀었다. 율이 먼저 다가온다.\n부드럽게 이마를 쓰다듬으며, 함께 숨을 고른다.",
                ][self.bonding.stage()].into();
            }
            Action::Start if self.phase == Phase::Title => {
                self.phase = Phase::Story(0);
                self.modal = Modal::None;
                self.bonding = Bonding::default();
                self.message = "“율아, 같이 놀까?” 먼저 장난감을 천천히 흔들어 보자.\n[Space] 또는 오른쪽 위 놀이 버튼으로 율에게 말을 걸어 주세요.".into();
            }
            Action::Next => {
                if let Phase::Story(n) = self.phase {
                    // Guard the story itself, so Enter and queued actions cannot skip bonding.
                    if !self.story_ready() {
                        return;
                    }
                    if n < 4 {
                        self.phase = Phase::Story(n + 1)
                    } else {
                        self.phase = Phase::Search;
                        self.message="모리 · 빛나는 번호를 누르면 그곳으로 걸어가 조사해. 수첩에 단서를 모아 보자!".into();
                    }
                }
            }
            Action::Inspect(id) if self.phase == Phase::Search && self.available(id) => {
                self.found[id] = true;
                self.modal = Modal::Clue(id);
                self.message = format!("모리 · {}", CLUES[id].lesson);
            }
            Action::Close => {
                self.modal = Modal::None;
                self.selected.clear();
            }
            Action::Journal if self.phase == Phase::Search => {
                self.modal = if self.modal == Modal::Journal {
                    Modal::None
                } else {
                    Modal::Journal
                }
            }
            Action::Deduce if self.phase == Phase::Search && self.chapter < 3 => {
                self.modal = Modal::Deduction;
                self.selected.clear();
                self.message.clear();
                self.evidence = EvidenceView {
                    preview: self.found.iter().position(|found| *found),
                    ..Default::default()
                };
            }
            Action::ViewClue(id)
                if self.modal == Modal::Deduction && id < CLUES.len() && self.found[id] =>
            {
                self.evidence.preview = Some(id);
                self.message.clear();
            }
            Action::Select(id) if self.modal == Modal::Deduction && id < 8 && self.found[id] => {
                self.message.clear();
                if self.selected.contains(&id) {
                    self.selected.retain(|i| *i != id)
                } else if self.selected.len() < 2 {
                    self.selected.push(id)
                } else {
                    self.message =
                        "모리 · 이미 두 단서를 골랐어. 바꾸려면 선택한 단서 중 하나를 먼저 빼 줘."
                            .into();
                }
            }
            Action::Theory(answer) if self.modal == Modal::Deduction && self.chapter < 3 => {
                let pair = PAIRS[self.chapter];
                if self.selected.len() != 2 {
                    self.message = "모리 · 먼저 이 추리를 뒷받침할 단서 두 개를 골라 줘.".into();
                } else if !pair.iter().all(|i| self.selected.contains(i)) {
                    self.mistakes += 1;
                    self.message = if self.selected.contains(&3) {
                        "모리 · 상자에 남은 털색이 율의 털색과 같은지 다시 비교해 봐. 고양이의 흔적이라고 모두 율의 것은 아니야."
                    } else { match self.chapter {
                        0 => "모리 · 두 단서가 탈출 직후의 위치와 이동 방식을 함께 뒷받침하는지 살펴봐. 목격된 장소만으로 수색 범위를 단정하기는 어려워.",
                        1 => "모리 · 어느 방향으로 이동했는지와 그곳이 숨기에 알맞은지를 함께 설명할 근거가 필요해. 관찰 기록과 주변 환경을 비교해 봐.",
                        _ => "모리 · 지금 율이 보내는 신호와 안심할 수 있는 방법을 연결해 봐. 예전의 이동 흔적만으로 지금의 마음을 알 수는 없어.",
                    }}.into();
                } else if answer != CORRECT[self.chapter] {
                    self.mistakes += 1;
                    self.message = format!(
                        "모리 · {}",
                        match self.chapter {
                            0 =>
                                "큰길이나 지붕으로 향한 흔적은 없어요. 문 앞의 털과 화분 안쪽 발자국이 함께 가리키는 수색 범위를 골라 봐요.",
                            1 =>
                                "분수에서는 돌아섰고, 빵집 입구로 이어진 흔적도 없어요. 발자국의 방향과 몸을 가릴 공간이 모두 맞는 곳은 어디일까요?",
                            _ =>
                                "지금 달려가거나 출구를 막으면 겁먹은 율이 더 불안해질 수 있어요. 율의 자세와 담요의 익숙한 냄새를 함께 생각해 봐요.",
                        }
                    );
                } else {
                    let solved = self.chapter;
                    self.chapter += 1;
                    self.modal = Modal::Conclusion(solved);
                    self.selected.clear();
                    self.message = NEXT_STEPS[solved].into();
                }
            }
            Action::Rescue(choice) if self.phase == Phase::Search && self.chapter == 3 => {
                self.modal = Modal::Rescue;
                let correct = [1, 0, 2][self.trust.min(2)];
                if choice == correct {
                    self.trust += 1;
                    self.message = match self.trust {
                        1 => {
                            "“율아, 괜찮아.” 몸을 낮추고 거리를 두자, 납작했던 귀가 조금 올라왔다."
                        }
                        2 => "담요를 놓은 열린 이동장. 익숙한 냄새에 율의 작은 코가 씰룩인다.",
                        _ => {
                            "한 발, 또 한 발. 율이 스스로 내 곁으로 걸어왔다. 율아, 우리 집에 가자."
                        }
                    }
                    .into();
                    if self.trust >= 3 {
                        self.phase = Phase::Reunion;
                        self.modal = Modal::None;
                    }
                } else {
                    self.mistakes += 1;
                    self.message="모리 · 율이 아직 조금 무서운가 봐. 서두르지 않아도 괜찮아. 거리를 두고, 익숙한 공간을 만들고, 조용히 기다려 보자.".into();
                }
            }
            Action::Hint if self.phase == Phase::Search => {
                self.hints += 1;
                self.message = if self.chapter < 3 {
                    let missing = PAIRS[self.chapter].iter().find(|i| !self.found[**i]);
                    if let Some(id) = missing {
                        format!(
                            "모리 · {} 쪽에 마음에 걸리는 흔적이 있어. 빛나는 기둥을 따라가 봐.",
                            CLUES[*id].place
                        )
                    } else {
                        "모리 · 필요한 단서가 모였어! [추리하기]에서 단서 두 개와 알맞은 생각을 연결해 보자.".into()
                    }
                } else {
                    "모리 · 온실 앞에서 [E]. 낮은 자세 → 담요를 둔 이동장 → 조용한 기다림. 율이 선택하게 해 줘.".into()
                };
            }
            Action::Help => {
                self.modal = if self.modal == Modal::Help {
                    Modal::None
                } else {
                    Modal::Help
                }
            }
            Action::Pause => {
                self.modal = if self.modal == Modal::None {
                    Modal::Pause
                } else {
                    Modal::None
                }
            }
            Action::Restart => {
                let muted = self.muted;
                *self = Self::default();
                self.muted = muted;
                self.phase = Phase::Story(0);
                self.message =
                    "“율아, 같이 놀까?” [Space] 또는 놀이 버튼으로 먼저 함께 놀아 주세요.".into();
            }
            Action::MainMenu if self.phase == Phase::Reunion => {
                let muted = self.muted;
                let revision = self.revision;
                *self = Self::default();
                self.muted = muted;
                self.revision = revision;
            }
            Action::Sound => self.muted = !self.muted,
            _ => {}
        }
        if self.phase != phase {
            self.scene_elapsed = 0.;
        }
        self.sync_dialogue();
        self.revision = self.revision.wrapping_add(1);
    }
    pub fn validate(&self) -> bool {
        self.chapter <= 3
            && self.trust <= 3
            && (self.trust == 0 || self.chapter == 3)
            && (0..self.chapter).all(|c| PAIRS[c].iter().all(|i| self.found[*i]))
            && (self.phase != Phase::Reunion || self.trust == 3)
            && (!matches!(self.phase,Phase::Story(n) if n>4))
            && (self.trust != 3 || self.phase == Phase::Reunion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn search() -> Case {
        Case {
            phase: Phase::Search,
            ..Default::default()
        }
    }
    fn act(c: &mut Case, action: Action) {
        c.apply(Action::FinishDialogue);
        c.apply(action);
    }
    #[test]
    fn complete_case_with_wrong_turns_and_recovery() {
        let mut c = search();
        assert!(!c.available(5));
        act(&mut c, Action::Inspect(5));
        assert!(!c.found[5]);
        for ch in 0..3 {
            for id in PAIRS[ch] {
                act(&mut c, Action::Inspect(id));
                act(&mut c, Action::Close);
            }
            assert!(c.ready());
            act(&mut c, Action::Deduce);
            act(&mut c, Action::Theory(CORRECT[ch]));
            assert_eq!(c.chapter, ch);
            for id in PAIRS[ch] {
                act(&mut c, Action::Select(id));
            }
            act(&mut c, Action::Theory((CORRECT[ch] + 1) % 3));
            assert_eq!(c.chapter, ch);
            act(&mut c, Action::Theory(CORRECT[ch]));
            assert_eq!(c.chapter, ch + 1);
            assert_eq!(c.modal, Modal::Conclusion(ch));
            act(&mut c, Action::Close);
            assert!(c.validate());
        }
        act(&mut c, Action::Rescue(0));
        assert_eq!(c.trust, 0);
        for a in [1, 0, 2] {
            act(&mut c, Action::Rescue(a));
            assert!(c.validate());
        }
        assert_eq!(c.phase, Phase::Reunion);
        act(&mut c, Action::MainMenu);
        assert_eq!(c.phase, Phase::Title);
        assert!(c.validate());
        act(&mut c, Action::Start);
        assert_eq!(c.chapter, 0);
        assert_eq!(c.count(), 0);
    }
    #[test]
    fn save_round_trip_and_corruption_guard() {
        let mut c = search();
        act(&mut c, Action::Inspect(0));
        let loaded: Case = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert!(loaded.validate());
        assert_eq!(loaded.modal, Modal::None);
        assert!(loaded.found[0]);
        c.chapter = 3;
        assert!(!c.validate());
    }
    #[test]
    fn wrong_evidence_cannot_unlock_chapter() {
        let mut c = search();
        for i in [0, 1, 2, 3, 4] {
            act(&mut c, Action::Inspect(i));
        }
        act(&mut c, Action::Deduce);
        for i in [2, 3] {
            act(&mut c, Action::Select(i));
        }
        act(&mut c, Action::ViewClue(0));
        assert_eq!(c.modal, Modal::Deduction);
        assert_eq!(c.selected, vec![2, 3]);
        assert_eq!(c.evidence.preview, Some(0));
        // Reading another candidate and trying a third selection must not replace either choice.
        act(&mut c, Action::Select(4));
        assert_eq!(c.selected, vec![2, 3]);
        act(&mut c, Action::Theory(1));
        assert_eq!(c.chapter, 0);
        assert_eq!(c.mistakes, 1);
    }
    #[test]
    fn every_story_beat_and_restart_are_reachable() {
        let mut c = Case::default();
        c.apply(Action::Start);
        c.apply(Action::Next);
        assert_eq!(c.phase, Phase::Story(0));
        for stage in 0..3 {
            c.apply(Action::Play);
            // Repeated presses cannot complete a reaction or skip the first afternoon.
            c.apply(Action::Play);
            c.apply(Action::Next);
            assert_eq!(c.bonding.completed, stage);
            assert_eq!(c.phase, Phase::Story(0));
            c.tick_bonding(BOND_DURATIONS[stage]);
            assert_eq!(c.bonding.completed, stage + 1);
            c.apply(Action::FinishDialogue);
        }
        assert!(c.bonding.ready());
        for i in 0..5 {
            assert_eq!(c.phase, Phase::Story(i));
            c.apply(Action::FinishDialogue);
            c.tick(2.0);
            c.apply(Action::Next);
        }
        assert_eq!(c.phase, Phase::Search);
        c.apply(Action::Restart);
        assert_eq!(c.phase, Phase::Story(0));
        assert!(!c.bonding.ready());
        assert_eq!(c.bonding.completed, 0);
    }
}
