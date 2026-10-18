# 오늘도, 너에게 · FindCat

집을 나간 작은 고양이 **율**을 찾아, 고양이의 마음을 배우는 따뜻한 3D 추리 게임입니다. Rust / Bevy 0.17.3으로 만든 Windows / WebGL2 게임입니다. 한 동네에서 시작과 결말이 있는 짧은 싱글 플레이 이야기입니다.

**[브라우저에서 플레이](https://edoli.github.io/FindCat/)** · [자동 배포 상태](https://github.com/edoli/FindCat/actions/workflows/pages.yml)

## 웹 실행과 배포

지원하는 모바일 브라우저에서는 첫 터치 후 전체화면을 요청합니다. 오른쪽 아래 아이콘으로 전체화면을 끄거나 다시 켤 수 있으며, 수동으로 종료한 뒤에는 자동으로 재진입하지 않습니다. 전체화면을 지원하지 않거나 요청이 거절되면 일반 화면으로 계속 플레이합니다. 데스크톱에는 자동 전환이나 전체화면 아이콘을 표시하지 않습니다.

본문은 Pretendard Regular, 제목과 버튼은 SemiBold를 사용합니다. 글자 가장자리의 커버리지는 밝은 배경의 어두운 글자와 어두운 버튼의 밝은 글자를 구분해 보정합니다. 선형 색상으로 합성할 때 본문 획이 지나치게 가늘어지거나 밝은 버튼 글자가 번져 보이는 현상을 줄입니다. 보정은 글리프별로 한 번만 적용하므로 화면을 다시 열어도 누적되지 않습니다.

`uv run --project python python python/validate_typography.py --url https://edoli.github.io/FindCat/`는 DPR 1 / 2에서 실제 게임 글자와 같은 글꼴의 브라우저 렌더링을 비교하고, UI를 반복해서 다시 그려도 글자 픽셀이 변하지 않는지 확인합니다. 비교 이미지는 `artifacts/typography`에 생성합니다.

WebGL2를 지원하는 최신 PC 브라우저에서 실행하세요. 첫 입력 후 소리가 켜지며, 진행 상황은 해당 브라우저의 로컬 저장소에 자동 저장됩니다. 렌더링 해상도는 브라우저 창 크기 × 기기의 픽셀 배율(DPR)에 맞춥니다. 창 크기와 모니터 배율이 바뀌면 3D 화면과 글자도 해당 해상도로 다시 그립니다. UI 크기는 Bevy 안에서 조절하므로 완성된 캔버스를 CSS로 확대하지 않습니다. 글자는 grayscale antialiasing, 3D는 4× MSAA를 사용합니다.

```powershell
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked
trunk serve --release
```

`http://127.0.0.1:8080`에서 플레이할 수 있습니다. 정적 웹 배포본은 `trunk build --release --locked --public-url /FindCat/`로 `dist/web`에 생성합니다. 이어서 `uv run --project python python python/web_licenses.py`로 의존성 고지문을 포함합니다. `.github/workflows/pages.yml`은 `main` push와 수동 실행 시 게임 규칙 테스트 → WASM 빌드 → GitHub Pages 배포를 수행합니다. Pages의 Source는 **GitHub Actions**입니다.

웹 검증은 `uv run --project python python python/validate_web.py --url https://edoli.github.io/FindCat/`로 실행합니다. 해상도와 글자 검증은 `uv run --project python python python/validate_resolution.py --url https://edoli.github.io/FindCat/`로 실행합니다. 창 크기 변경, DPR 1 / 1.25 / 1.5 / 2, 실제 WebGL drawing buffer와 카메라 크기 일치, 글자 가장자리의 중간 계조, UI 범위와 클릭 위치를 확인합니다. Playwright Chromium이 필요하며, `uv run --project python playwright install chromium`으로 설치합니다. OS 마우스와 키보드를 사용하지 않고 실제 브라우저 안에서 플레이합니다. `?testbed`는 검사에 필요한 상태와 버튼 좌표를 읽기 전용 DOM 속성으로 내보냅니다.

원본 `reference/cat` 사진은 저장소에 포함하지 않습니다. 배포에 필요한 모델과 가공된 고양이 사진은 `assets`에 포함되어 있습니다.

사진 속 동그란 호박색 눈, 크림빛 갈색 무늬, 하얀 가슴과 양말, 분홍 코를 가진 고양이가 주인공입니다. `reference/cat`의 사진을 참고해 직접 만든 3D 모델과 실제 사진 한 장을 수첩에 사용했습니다.

## 실행

배포본은 ZIP을 **모두 풀고** `FindCat.exe`를 실행하세요. `assets` 폴더가 실행 파일 옆에 있어야 합니다. Blender, Python, Rust 설치와 인터넷 연결은 플레이에 필요하지 않습니다.

- Windows 10 / 11, x64
- Vulkan 또는 DirectX 12를 지원하는 그래픽 장치와 최신 드라이버
- 최소 창 크기 1100 × 760, 기본 1440 × 900
- 일부 PC에서는 Microsoft Visual C++ 2015–2022 x64 런타임이 필요할 수 있습니다.

소스에서 실행:

```powershell
cargo run --locked
```

첫 빌드는 Bevy와 그래픽 라이브러리를 컴파일하므로 시간이 걸립니다. 생성된 에셋은 이미 `assets`에 있어 재생성할 필요가 없습니다.

## 플레이

이야기, 단서 관찰, 모리의 추리 피드백과 재회 대사는 한 글자씩 나타납니다. 표시 중 대사창을 클릭하거나 Enter·Space를 누르면 현재 대사만 즉시 완성됩니다. 그 입력으로 다음 이야기나 선택지가 함께 실행되지는 않습니다. 대사를 모두 표시한 뒤 율의 놀이·탈출, 모리의 등장 같은 장면 동작이 시작되며, 해당 연출이 끝나면 다음 이야기를 선택할 수 있습니다. 대사창은 완성된 문장 기준으로 공간을 확보해 타이핑 중에도 줄바꿈과 높이를 유지합니다.

첫 장면에서는 장난감 흔들기, 공 굴려 주기, 쓰다듬기를 순서대로 함께합니다. 각 행동은 율의 반응이 끝난 뒤 다음 행동으로 이어지며, 세 번의 교감(최소 약 12.5초)을 마쳐야 ‘다음 이야기’ 버튼이 나타납니다. Enter나 버튼 연타로 이 시간을 건너뛸 수 없습니다. 놀이 도중에는 일시 정지도 가능합니다.

열린 문으로 율이 뛰어나가며 서서히 사라집니다. 율이 사라진 뒤 슬픔의 장면을 지나, 고양이 요정 **모리**가 페이드 인으로 처음 등장합니다. 눈물을 닦고 함께 탐정 수첩을 펼칩니다.

1. 장소의 번호를 클릭해 걸어가 조사합니다. 직접 걸으며 조사할 수도 있습니다.
2. 동네의 흔적 여덟 개를 읽고 고양이의 행동을 배웁니다.
3. 추리 화면의 **수집한 단서 전체 목록**에서 직접 두 근거를 고릅니다. 목록을 누르면 가운데 상세 패널에서 발견 장소·전체 관찰 기록·고양이 습성을 확인할 수 있고, 별도의 선택 버튼으로 추리에 담습니다. 수집한 단서는 페이지 구분 없이 모두 표시됩니다. 상세 패널에는 관찰 기록과 고양이 습성이 함께 나오며, 다른 단서를 읽어도 두 선택은 유지됩니다. 결론을 제출하기 전에는 정답 단서나 연결 해설을 보여 주지 않습니다.
4. 세 번의 추리로 수색 범위, 은신처, 다가갈 방법을 알아냅니다.
5. 겁먹은 아이에게 안전한 거리와 익숙한 냄새를 주고, 스스로 다가오길 기다립니다.
6. 재회 후 율과 장난감을 쫓고 담요 위에서 쉬는 오후를 보냅니다. 대사와 약 30초의 장면 연출이 끝나면 메인 화면으로 자동 복귀하며, 완료 기록에서 행복한 오후를 다시 볼 수 있습니다.

추리 성공 화면은 두 관찰이 결론으로 이어지는 이유와 다음 조사할 곳을 설명합니다. 시간 제한과 게임 오버는 없습니다. 틀리면 모리가 이유를 설명해 주며 다시 선택할 수 있습니다. 첫 추리 후 얻는 **발자국 렌즈**와 모리의 빛이 탐색을 돕습니다. 모든 단서를 모으면 ‘다정한 관찰자’ 기록을 받습니다.

| 조작 | 기능 |
|---|---|
| 장소 번호 클릭 | 장애물을 피해 이동한 뒤 조사 |
| 바닥 클릭 | 해당 지점으로 이동 |
| WASD / 방향키 | 화면 방향에 맞춰 직접 이동 |
| E | 가까운 흔적 조사 / 고양이에게 마음 전하기 |
| J | 수첩 열기 / 닫기 |
| R | 단서 연결과 추리 |
| Q | 모리의 안내 |
| Tab | 동네 전체 보기 / 탐정 따라가기 |
| 마우스 휠 | 탐색 중 확대 / 축소 |
| M | 음악과 효과음 켜기 / 끄기 |
| Enter / Space | 이야기 진행 (첫 기억의 Space는 장난감 놀이) |
| Esc | 일시 정지 / 창 닫기 |
| F1 | 도움말 |
| F12 | 현재 작업 폴더의 `artifacts/screenshot.png` 저장 |

선택지는 마우스로 클릭합니다. 진행은 `%LOCALAPPDATA%\FindCat\save.json`에 자동 저장됩니다. 타이틀 화면의 ‘이어서 찾기’로 돌아옵니다. 손상되거나 유효하지 않은 저장 파일은 불러오지 않습니다. ‘처음부터’는 새 이야기를 시작하며, 새 수색 진행이 저장되면 기존 기록을 대체합니다.

## 화면과 연출

화면 왼쪽 위에는 현재 목표, 오른쪽 위에는 모은 단서, 아래에는 수첩·추리·도움 버튼을 배치했습니다. 지도 번호에 마우스를 올리거나 가까이 가면 장소 이름이 나타납니다. 글자는 불투명한 바탕 위에 표시하고, 본문은 17–19px, 주요 제목과 버튼은 굵은 한글 폰트를 사용합니다.

카메라는 놀이·탈출·슬픔·요정의 등장·탐색 시작에 맞춰 위치와 거리를 부드럽게 바꿉니다. 조사할 때는 해당 장소를 가까이 보여주며 단서 카드를 오른쪽에 표시합니다. 재회 후에는 탐정과 고양이가 함께 집 앞으로 걸어옵니다. 고양이의 눈 깜빡임·고개·꼬리, 탐정의 걸음과 감정 동작, 새·나비·물방울도 움직입니다.

동네에는 빵과 과일, 자전거, 화단, 빨래, 담쟁이, 읽던 책, 동네 고양이, 빵집 주인, 온실의 선반과 늘어진 식물을 추가했습니다. 정적인 소품은 재질별로 합쳐 렌더링 부하를 줄였습니다.

## 고양이에 대한 이야기

실종 고양이 행동의 토대는 [Missing Animal Response Network의 행동 설명](https://www.missinganimalresponse.com/lost-cat-behavior/)과 [실종 고양이 수색 연구](https://www.missinganimalresponse.com/missing-cat-study/)입니다. 낯선 야외에 나온 집고양이가 가까운 은신처에서 소리 없이 숨을 수 있고, 보호자의 목소리에 응답하지 않을 수 있다는 내용을 추리에 반영했습니다. 자료를 그대로 인용한 것이 아니라 게임에 맞게 새로 쓴 이야기입니다.

게임의 특정 발자국, 목격담과 정답은 허구입니다. 고양이의 성격과 상황은 다르며, 담요를 두거나 기다린다고 모든 고양이가 같은 방식으로 행동하는 것은 아닙니다. 실제 실종 상황에서는 주변 수색과 이웃·보호소 연락 등 여러 방법이 함께 필요합니다. 게임이 전달하려는 마음은 **쫓거나 억지로 잡기보다 고양이가 두려워한다는 것을 이해하기**입니다.

## 에셋 파이프라인

`python`은 uv 프로젝트입니다. 모델은 Blender Python API로 생성하며, Blender MCP나 화면 조작 없이 재현할 수 있습니다. 난수 시드는 고정되어 있습니다.

```powershell
uv sync --project python
uv run --project python python python/prepare_media.py
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python python/generate_assets.py
```

- `generate_assets.py`: 고양이, 탐정, 모리, 동네 GLB 4개. 둥근 형태와 파스텔 재질, 빵집·온실·분수·정원·가구·발자국을 직접 생성합니다.
- `prepare_media.py`: 사진 리사이즈, OFL 한글 폰트 다운로드, 독자적인 음악과 효과음 WAV 합성. 폰트 최초 다운로드에만 인터넷이 필요합니다.
- `src/game.rs`: 화면과 독립적인 이야기·증거·추리·안심시키기·저장 상태 로직.
- `src/navigation.rs`: 충돌 판정과 격자 A* 경로 탐색.
- `src/main.rs`: Bevy 렌더링, 실제 입력, 이동, 애니메이션, 오디오와 저장.
- `src/ui.rs`: 한국어 수첩, 단서 카드, 추리 보드, 화면에 투영되는 조사 표식.
- `src/presentation.rs`: 장면별 카메라, 캐릭터 동작, 생활 효과, 음악 볼륨 전환.
- `src/testbed.rs`: 실제 게임 시스템을 이용하는 무인 종단 테스트.

## 검증

```powershell
cargo fmt --all --check
cargo test --locked
uv run --project python python python/validate.py
```

화면 없이 이야기 로직만 빠르게 확인하려면 `cargo test --lib --no-default-features --locked`를 사용할 수 있습니다.

검증기는 별도 게임 프로세스를 실행합니다. 실제 마우스나 키보드를 건드리지 않고 Bevy 입력 리소스와 게임 액션으로 이야기부터 결말까지 진행합니다. 여덟 장소의 실제 경로 이동, 단서 조사, 추리 세 번, 잘못된 선택 복구, 신뢰 세 단계, 저장 파일 재로딩을 확인합니다. GPU로 그린 화면을 PNG로 저장하고 비어 있지 않은지 검사합니다.

`artifacts/testbed-report.json`은 결과, `artifacts/testbed.log`는 실행 로그, `artifacts/01-title.png`부터 `07-reunion.png`는 화면입니다. 테스트 저장은 `artifacts/testbed`에 격리되어 개인 진행 상황을 건드리지 않습니다.

UI의 본문·버튼 색상 대비도 검사합니다. 전체 자동 플레이 중 카메라 위치·거리 변화를 기록하고, UI 패널의 화면 이탈과 세로 넘침을 확인합니다. 1100 × 760 창에서 단서 카드와 모든 단서가 모인 추리 화면, 수첩과 도움말도 캡처합니다. `14-small-evidence.png`, `15-small-deduction.png`, `16-journal.png`, `17-help.png`에서 작은 창의 배치를 볼 수 있습니다.

## 배포 빌드

```powershell
cargo build --release --locked
uv run --project python python python/package.py
uv run --project python python python/validate.py --exe dist/FindCat-1.0.0-windows-x64/FindCat.exe
```

`dist/FindCat-1.0.0-windows-x64.zip`에 실행 파일, 모든 에셋, 안내서, 라이선스 목록, 파일별 SHA-256을 담습니다. 배포용 ZIP은 itch.io 등에서 Windows 다운로드로 올릴 수 있는 포터블 형식입니다. 스토어 업로드, 코드 서명, 설치 프로그램은 포함하지 않습니다.

## 권리

코드와 직접 생성한 모델·음악은 MIT 라이선스입니다. 제공된 고양이 사진의 권리는 원소유자에게 있습니다. 사진은 MIT 재허락 대상에 포함하지 않습니다. 폰트와 외부 라이브러리의 고지는 `THIRD_PARTY.md`와 배포본 `licenses`에 있습니다.
