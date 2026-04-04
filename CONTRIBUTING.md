# Contributing to CommitCat

CommitCat is a cross-platform desktop pet built with **Tauri 2** (Rust backend) and **React 19** (frontend). Contributions of any kind are welcome.

---

## Table of Contents

- [Development Setup](#development-setup)
- [Build & Run](#build--run)
- [Project Structure](#project-structure)
- [Server Development](#server-development)
- [Running Tests](#running-tests)
- [How to Contribute](#how-to-contribute)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Commit Convention](#commit-convention)

---

## Development Setup

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| **Node.js** | 18+ | [nodejs.org](https://nodejs.org/) |
| **Rust** | stable | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Tauri CLI** | 2.x | `cargo install tauri-cli --version "^2"` |

macOS의 경우 추가로 Xcode Command Line Tools가 필요합니다:

```bash
xcode-select --install
```

---

## Build & Run

### 데스크탑 앱 실행

```bash
git clone https://github.com/eunseo9311/commit-cat.git
cd commit-cat
npm install
npm run tauri dev
```

Vite dev server(프론트엔드)와 Tauri backend가 함께 시작됩니다. 고양이 창이 데스크탑에 표시됩니다.

### 프로덕션 빌드

```bash
npm run tauri build
```

### 빠른 체크 (빌드 없이 오류만 확인)

```bash
# 코어 크레이트
cargo check -p commit-cat-core

# 데스크탑 앱
cargo check -p commit-cat
```

### VSCode Extension

```bash
cd vscode-extension
npm install
# VSCode에서 F5 → Extension Development Host 실행
```

### JetBrains Plugin

```bash
cd jetbrains-plugin
./gradlew buildPlugin
# 빌드 결과: build/distributions/commitcat-jetbrains-*.zip
```

---

## Project Structure

```
commit-cat/
├── crates/
│   └── commit-cat-core/          # 플랫폼 독립 코어 로직
│       ├── src/models/            # 데이터 구조 (AppData, CatState 등)
│       ├── src/state_machine.rs   # 고양이 행동 상태 전이
│       ├── src/xp.rs              # XP 계산, 레벨링, 스트릭
│       └── src/platform.rs        # 트레이트 정의 (EventEmitter, Storage, IdeDetector)
│
├── server/                        # 클라우드 API 서버 (Rust + Axum)
│   └── src/                       # HTTP 핸들러, 인증, DB 로직
│
├── src-tauri/                     # Tauri 데스크탑 앱 (Rust backend)
│   ├── src/commands/              # IPC 커맨드 핸들러
│   ├── src/services/              # 백그라운드 서비스 (git, github, docker 등)
│   └── src/platform/              # OS별 구현 (macOS, Windows, Linux)
│
├── src/                           # React 프론트엔드
│   ├── components/                # UI 컴포넌트 (Cat, Settings, Summary 등)
│   ├── stores/                    # Zustand 상태 관리
│   └── styles/                    # CSS
│
├── vscode-extension/              # VSCode Extension (TypeScript)
│   └── src/extension.ts
│
├── jetbrains-plugin/              # JetBrains Plugin (Kotlin)
│   └── src/main/kotlin/
│
└── assets/                        # 픽셀 아트 스프라이트 및 아이콘
```

### 아키텍처 원칙

- **코어 로직은 `commit-cat-core`에** — 데스크탑, 서버, 향후 WASM 타겟에서 공유
- **플랫폼별 코드는 `src-tauri/src/platform/`에** — OS마다 파일 하나
- **새 플랫폼은 `commit-cat-core::platform`의 트레이트를 구현** — 기존 파일 수정 불필요

---

## Server Development

클라우드 싱크 기능을 담당하는 API 서버입니다.

```bash
# 서버 실행 (기본 포트: 3000)
cargo run -p commit-cat-server

# 환경변수 설정 예시
DATABASE_URL=sqlite://./data.db cargo run -p commit-cat-server
```

서버는 `server/` 디렉토리에 있으며, 프로덕션 환경은 Fly.io(`commitcat-api.fly.dev`)에서 운영됩니다.

---

## Running Tests

```bash
# 전체 테스트 실행
cargo test

# 특정 크레이트만
cargo test -p commit-cat-core

# 서버 테스트만
cargo test -p commit-cat-server

# 특정 테스트 이름으로 필터링
cargo test xp_calculation
```

---

## How to Contribute

### Bug Reports

[issue](https://github.com/eunseo9311/commit-cat/issues)를 열고 아래 내용을 포함해주세요:

- OS 및 버전 (macOS 15.x, Windows 11, Ubuntu 24.04 등)
- CommitCat 버전
- 재현 단계
- 기대 동작 vs 실제 동작
- 스크린샷 (해당되는 경우)

### Feature Suggestions

`enhancement` 라벨로 issue를 열고 다음을 설명해주세요:

- 어떤 문제를 해결하는가
- 어떻게 동작했으면 하는가
- 고려한 대안이 있다면

### Code Contributions

1. 저장소를 **Fork**
2. `main`에서 브랜치 생성:
   ```bash
   git checkout -b feat/your-feature
   ```
3. 변경사항 작성
4. `cargo check` 통과 및 앱 동작 확인
5. Push 후 Pull Request 오픈

### Design & Assets

픽셀 아트 기여를 환영합니다. 새 고양이 스킨, 아이템 스프라이트, 애니메이션을 제안하려면 먼저 issue를 열어 방향을 논의해주세요.

---

## Pull Request Guidelines

### 제출 전 체크리스트

- [ ] `cargo check -p commit-cat-core` 통과
- [ ] `cargo check -p commit-cat` 통과
- [ ] `cargo test` 통과
- [ ] `npm run tauri dev`로 앱 정상 동작 확인
- [ ] 관련 없는 변경사항 미포함

### PR 제목 형식

```
feat: add item equip system
fix: cat not hiding on fullscreen (macOS)
refactor: extract git polling into core crate
docs: add Linux troubleshooting guide
chore: update dependencies
```

### PR 설명

- **무엇을** 바꿨는지, **왜** 바꿨는지 설명
- 관련 issue 링크 (예: `Closes #42`)
- UI 변경사항은 스크린샷 포함

### 리뷰 프로세스

- PRs는 메인테이너가 리뷰합니다
- 크고 복잡한 PR보다 작고 집중된 PR을 선호합니다
- 작업 중인 경우 **Draft**로 표시해주세요

---

## Commit Convention

```
type: short description

# Types:
# feat     — 새 기능
# fix      — 버그 수정
# refactor — 동작 변경 없는 코드 구조 개선
# docs     — 문서
# chore    — 유지보수 (의존성, 설정, CI)
# style    — 포맷팅, CSS
# test     — 테스트 추가 또는 수정
```

커밋 메시지는 **무엇을** 했는지보다 **왜** 했는지를 설명하세요.

```
# Good
feat: add streak milestone notifications at 3/7/30 days

# Not ideal
update code
```

---

## Good First Issues

[`good first issue`](https://github.com/eunseo9311/commit-cat/labels/good%20first%20issue) 라벨이 붙은 issue를 찾아보세요.

기여하기 좋은 영역:

- **Linux 개선** — 풀스크린 감지, IDE 프로세스 감지 개선
- **새 IDE 지원** — 아직 지원하지 않는 에디터 추가
- **문서** — 오타 수정, 가이드 추가, README 개선
- **접근성** — 스크린 리더, 키보드 내비게이션 개선

---

## Questions?

- GitHub [Discussion](https://github.com/eunseo9311/commit-cat/discussions) 오픈
- 기존 [Issues](https://github.com/eunseo9311/commit-cat/issues) 검색

Thank you for contributing to CommitCat.
