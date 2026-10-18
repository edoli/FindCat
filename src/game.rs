use serde::{Deserialize, Serialize};

pub const TITLE: &str = "오늘도, 너에게";
pub const SUBTITLE: &str = "A LITTLE CAT DETECTIVE STORY";
pub const CAT_NAME: &str = "율";

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
    Theory(usize),
    Rescue(usize),
    Hint,
    Help,
    Pause,
    Restart,
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
    pub modal: Modal,
    #[serde(skip)]
    pub selected: Vec<usize>,
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
            modal: Modal::None,
            selected: vec![],
            message: "모리 · 작은 흔적에도 마음이 담겨 있어. 천천히 살펴보자.".into(),
            revision: 0,
        }
    }
}

impl Case {
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
        match action {
            Action::Play if self.phase == Phase::Story(0) => {
                self.message = "장난감에 율의 앞발이 쏙. 작은 발끝이 내 하루를 간질인다.".into();
            }
            Action::Start if self.phase == Phase::Title => {
                self.phase = Phase::Story(0);
                self.modal = Modal::None;
            }
            Action::Next => {
                if let Phase::Story(n) = self.phase {
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
                self.message =
                    "수첩에서 서로 연결되는 단서 두 개를 고르고, 추리를 선택하세요.".into();
            }
            Action::Select(id) if self.modal == Modal::Deduction && id < 8 && self.found[id] => {
                if self.selected.contains(&id) {
                    self.selected.retain(|i| *i != id)
                } else if self.selected.len() < 2 {
                    self.selected.push(id)
                }
            }
            Action::Theory(answer) if self.modal == Modal::Deduction && self.chapter < 3 => {
                let pair = PAIRS[self.chapter];
                if self.selected.len() != 2 {
                    self.message = "모리 · 먼저 이 추리를 뒷받침할 단서 두 개를 골라 줘.".into();
                } else if !pair.iter().all(|i| self.selected.contains(i)) {
                    self.mistakes += 1;
                    self.message=match self.chapter {0=>"모리 · 문틈의 털과 화분 뒤 발자국을 함께 보면 율이 어디로 갔는지 알 수 있어.",1=>"모리 · 물가에서 돌아선 흔적과 바람 종 근처의 환경을 연결해 봐.",_=>"모리 · 숨어 있는 율의 모습과 집 냄새가 나는 물건을 연결해 봐."}.into();
                } else if answer != CORRECT[self.chapter] {
                    self.mistakes += 1;
                    self.message="모리 · 고양이의 눈높이에서 다시 생각해 봐. 무서울 때는 조용히 몸을 숨기고, 안전하다고 느껴야 다가올 수 있어.".into();
                } else {
                    self.chapter += 1;
                    self.modal = Modal::None;
                    self.selected.clear();
                    self.message=match self.chapter {1=>"추리 성공! 발자국 렌즈를 얻었어요. [Q]로 모리의 빛을 부르면 아직 못 찾은 흔적을 안내해요.",2=>"추리 성공! 온실 아래를 살펴보세요. 뛰어가 안기보다 먼저 율의 마음을 읽어 주세요.",_=>"추리 성공! 준비가 됐어요. 온실 곁에서 [E]를 눌러 율에게 천천히 마음을 전하세요."}.into();
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
            }
            Action::Sound => self.muted = !self.muted,
            _ => {}
        }
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
    #[test]
    fn complete_case_with_wrong_turns_and_recovery() {
        let mut c = search();
        assert!(!c.available(5));
        c.apply(Action::Inspect(5));
        assert!(!c.found[5]);
        for ch in 0..3 {
            for id in PAIRS[ch] {
                c.apply(Action::Inspect(id));
                c.apply(Action::Close);
            }
            assert!(c.ready());
            c.apply(Action::Deduce);
            c.apply(Action::Theory(CORRECT[ch]));
            assert_eq!(c.chapter, ch);
            for id in PAIRS[ch] {
                c.apply(Action::Select(id));
            }
            c.apply(Action::Theory((CORRECT[ch] + 1) % 3));
            assert_eq!(c.chapter, ch);
            c.apply(Action::Theory(CORRECT[ch]));
            assert_eq!(c.chapter, ch + 1);
            assert!(c.validate());
        }
        c.apply(Action::Rescue(0));
        assert_eq!(c.trust, 0);
        for a in [1, 0, 2] {
            c.apply(Action::Rescue(a));
            assert!(c.validate());
        }
        assert_eq!(c.phase, Phase::Reunion);
    }
    #[test]
    fn save_round_trip_and_corruption_guard() {
        let mut c = search();
        c.apply(Action::Inspect(0));
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
        for i in [2, 3] {
            c.apply(Action::Inspect(i));
        }
        c.apply(Action::Deduce);
        for i in [2, 3] {
            c.apply(Action::Select(i));
        }
        c.apply(Action::Theory(1));
        assert_eq!(c.chapter, 0);
        assert_eq!(c.mistakes, 1);
    }
    #[test]
    fn every_story_beat_and_restart_are_reachable() {
        let mut c = Case::default();
        c.apply(Action::Start);
        for i in 0..5 {
            assert_eq!(c.phase, Phase::Story(i));
            c.apply(Action::Next);
        }
        assert_eq!(c.phase, Phase::Search);
        c.apply(Action::Restart);
        assert_eq!(c.phase, Phase::Story(0));
    }
}
