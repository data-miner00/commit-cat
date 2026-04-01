import { useEffect, useRef } from "react";
import { listen, emit } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { sendNotification, isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import { Cat } from "./components/cat/Cat";
import { useCatStore } from "./stores/catStore";
import type { CatProfile, CatProfilesResponse } from "./types/cat";

// 백엔드에서 오는 상태 데이터
interface ActivityStatus {
  isIdeRunning: boolean;
  activeIde: string | null;
  idleSeconds: number;
}

interface XpResult {
  level: number;
  currentExp: number;
  expToNext: number;
  leveledUp: boolean;
}

interface XpStatus {
  level: number;
  currentExp: number;
  expToNext: number;
}

/** 설정 확인 후 알림 전송 */
async function notify(title: string, body: string) {
  try {
    const settings = await invoke<{ notificationsEnabled?: boolean }>("get_settings");
    if (settings.notificationsEnabled === false) return;
    let granted = await isPermissionGranted();
    if (!granted) {
      const perm = await requestPermission();
      granted = perm === "granted";
    }
    if (granted) sendNotification({ title, body });
  } catch (_) {}
}

function App() {
  const {
    setState, setActiveIde, setIdleSeconds, addCodingMinute, setLevel, triggerLevelUp,
    setEmotion, clearEmotion, incrementCommitStreak, resetCommitStreak,
    incrementBuildFails, resetBuildFails, syncSubCats, applyActiveProfile,
  } = useCatStore();

  // celebrating/interaction 같은 임시 상태의 자동 복귀 타이머
  const tempStateTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // 감정 자동 해제 타이머
  const emotionTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  // 커밋 스트릭 리셋 타이머 (5분 내 연속 커밋)
  const commitStreakTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // 코딩 시간 카운터 (1분마다)
  const codingTimer = useRef<ReturnType<typeof setInterval> | null>(null);

  // XP 코딩 시간 카운터 (60분마다 +5 XP)
  const xpCodingMinutes = useRef(0);

  // 야간 코딩 XP (세션당 1회)
  const lateNightXpGiven = useRef(false);

  // 감정 트리거 헬퍼 (sleeping 상태면 무시)
  const triggerEmotion = (emotion: "surprised" | "excited" | "proud" | "angry", duration: number) => {
    const current = useCatStore.getState().state;
    if (current === "sleeping") return;
    if (emotionTimer.current) clearTimeout(emotionTimer.current);
    setEmotion(emotion);
    emit("sub-cat:emotion", { emotion, duration }).catch(() => {});
    emotionTimer.current = setTimeout(() => {
      clearEmotion();
      emit("sub-cat:emotion", { emotion: null, duration: 0 }).catch(() => {});
    }, duration);
  };

  const startCodingTimer = () => {
    if (!codingTimer.current) {
      codingTimer.current = setInterval(() => {
        addCodingMinute();
        invoke("add_coding_minute").catch(() => {});

        xpCodingMinutes.current += 1;
        if (xpCodingMinutes.current >= 60) {
          xpCodingMinutes.current = 0;
          invoke<XpResult>("add_xp", { amount: 5, source: "coding_hour" }).then((res) => {
            setLevel(res.level, res.currentExp, res.expToNext);
          }).catch(() => {});
        }
      }, 60_000);
    }
  };

  const stopCodingTimer = () => {
    if (codingTimer.current) {
      clearInterval(codingTimer.current);
      codingTimer.current = null;
    }
  };

  // 앱 초기화: XP 상태 동기화
  useEffect(() => {
    (async () => {
      try {
        const [status, settings, profileResponse] = await Promise.all([
          invoke<XpStatus>("get_xp_status"),
          invoke<{ maxCompanions?: number }>("get_settings"),
          invoke<CatProfilesResponse>("get_cat_profiles"),
        ]);
        setLevel(status.level, status.currentExp, status.expToNext);
        const activeProfile = profileResponse.profiles.find(
          profile => profile.id === profileResponse.activeProfileId,
        );
        if (activeProfile) {
          applyActiveProfile(activeProfile);
        }
        // 초기 로드 후 서브 고양이 동기화 (maxCompanions > 0일 때만)
        const companions = settings.maxCompanions ?? 2;
        if (companions > 0) {
          setTimeout(() => useCatStore.getState().syncSubCats(companions), 0);
        }
      } catch (e) {
        console.error("Failed to load XP status:", e);
      }
    })();
  }, [applyActiveProfile, setLevel]);

  useEffect(() => {
    const unlisten = Promise.all([
      // ── IDE 감지됨 → coding ──
      listen<string>("activity:ide-detected", (event) => {
        const ideName = event.payload;
        setActiveIde(ideName);
        setState("coding");
        startCodingTimer();
      }),

      // ── 주기적 상태 (10초마다) — 앱 시작 시 이미 IDE 켜져있는 경우 보완 ──
      listen<ActivityStatus>("activity:status", (event) => {
        const { isIdeRunning, activeIde } = event.payload;
        if (isIdeRunning) {
          setActiveIde(activeIde);
          const current = useCatStore.getState().state;
          if (current === "idle" || current === "sleeping") {
            setState("coding");
          }
          startCodingTimer();
        }
      }),

      // ── IDE 꺼짐 → idle ──
      listen("activity:ide-closed", () => {
        setActiveIde(null);
        setState("idle");
        stopCodingTimer();
      }),

      // ── 유휴 → idle ──
      listen<number>("activity:idle", () => {
        setState("idle");
      }),

      // ── 장시간 유휴 → sleeping ──
      listen<number>("activity:sleeping", () => {
        setState("sleeping");
        stopCodingTimer();
      }),

      // ── 밤 코딩 → tired + XP ──
      listen<number>("activity:late-night-coding", () => {
        // coding 상태에서만 tired로
        const current = useCatStore.getState().state;
        if (current === "coding") {
          setState("tired");
        }

        // 야간 코딩 XP (세션당 1회)
        if (!lateNightXpGiven.current) {
          lateNightXpGiven.current = true;
          invoke<XpResult>("add_xp", { amount: 15, source: "late_night" }).then((res) => {
            setLevel(res.level, res.currentExp, res.expToNext);
          }).catch(() => {});
        }
      }),

      // ── 주기적 상태 보고 (초기 상태 동기화 포함) ──
      listen<ActivityStatus>("activity:status", (event) => {
        const status = event.payload;
        setIdleSeconds(status.idleSeconds);

        // 현재 상태와 IDE 상태 동기화
        const currentState = useCatStore.getState().state;
        const currentIde = useCatStore.getState().activeIde;

        if (status.isIdeRunning && !currentIde) {
          setActiveIde(status.activeIde);
          if (currentState === "idle") {
            setState("coding");
          }
        } else if (!status.isIdeRunning && currentIde) {
          setActiveIde(null);
          if (currentState === "coding") {
            setState("idle");
          }
        }

        // bored 감정: 15분+ 미활동 & IDE 꺼짐
        const currentEmotion = useCatStore.getState().emotion;
        if (!status.isIdeRunning && status.idleSeconds >= 900) {
          if (currentEmotion !== "bored") {
            setEmotion("bored");
            emit("sub-cat:emotion", { emotion: "bored", duration: 0 }).catch(() => {});
          }
        } else if (currentEmotion === "bored") {
          // IDE 감지되거나 활동 재개 → bored 해제
          clearEmotion();
          emit("sub-cat:emotion", { emotion: null, duration: 0 }).catch(() => {});
        }
      }),

      // ── Git 커밋 → celebrating (임시) + XP + 감정 ──
      listen("git:new-commit", () => {
        if (tempStateTimer.current) clearTimeout(tempStateTimer.current);
        setState("celebrating");
        tempStateTimer.current = setTimeout(() => {
          const ide = useCatStore.getState().activeIde;
          setState(ide ? "coding" : "idle");
        }, 3000);

        // 커밋 스트릭 (5분 내 연속 커밋)
        incrementCommitStreak();
        if (commitStreakTimer.current) clearTimeout(commitStreakTimer.current);
        commitStreakTimer.current = setTimeout(() => {
          resetCommitStreak();
        }, 5 * 60_000);
        const commitCount = useCatStore.getState().recentCommitCount;
        if (commitCount >= 3) {
          triggerEmotion("excited", 4000);
        }

        // 커밋 XP +10
        invoke<XpResult>("add_xp", { amount: 10, source: "commit" }).then((res) => {
          setLevel(res.level, res.currentExp, res.expToNext);
        }).catch(() => {});

        notify("CommitCat", "new commit! +10 XP \uD83D\uDC3E");
      }),

      // ── Git push → XP +5 ──
      listen("git:new-push", () => {
        invoke<XpResult>("add_xp", { amount: 5, source: "push" }).then((res) => {
          setLevel(res.level, res.currentExp, res.expToNext);
        }).catch(() => {});
      }),

      // ── XP 레벨업 이벤트 (백엔드에서 emit) ──
      listen<number>("xp:level-up", (event) => {
        triggerLevelUp(event.payload);
        triggerEmotion("proud", 5000);
        notify("CommitCat", `LEVEL UP! You reached Lv.${event.payload}`);
      }),

      // ── GitHub PR 머지 → proud 감정 ──
      listen("github:pr-merged", () => {
        triggerEmotion("proud", 5000);
      }),

      // ── Docker 컨테이너 시작 → coding (IDE 미감지 시) ──
      listen<string>("docker:container-started", () => {
        const current = useCatStore.getState().state;
        if (current === "idle" || current === "sleeping") {
          setState("coding");
        }
      }),

      // ── Docker 빌드 완료 → celebrating ──
      listen("docker:build-complete", () => {
        if (tempStateTimer.current) clearTimeout(tempStateTimer.current);
        setState("celebrating");
        tempStateTimer.current = setTimeout(() => {
          const ide = useCatStore.getState().activeIde;
          setState(ide ? "coding" : "idle");
        }, 3000);

        notify("CommitCat", "docker build complete! +15 XP 🐳");
      }),

      // ── Plugin: 코딩 활동 → coding 상태 유지 ──
      listen("plugin:coding-active", () => {
        const current = useCatStore.getState().state;
        if (current === "idle" || current === "sleeping") {
          setState("coding");
        }
        startCodingTimer();
      }),

      // ── Plugin: 파일 저장 → celebrating (임시) ──
      listen("plugin:save", () => {
        if (tempStateTimer.current) clearTimeout(tempStateTimer.current);
        setState("celebrating");
        tempStateTimer.current = setTimeout(() => {
          const ide = useCatStore.getState().activeIde;
          setState(ide ? "coding" : "idle");
        }, 3000);
      }),

      // ── Plugin: 빌드 성공 → celebrating + 알림 + 빌드실패 리셋 ──
      listen("plugin:build-success", () => {
        if (tempStateTimer.current) clearTimeout(tempStateTimer.current);
        setState("celebrating");
        tempStateTimer.current = setTimeout(() => {
          const ide = useCatStore.getState().activeIde;
          setState(ide ? "coding" : "idle");
        }, 3000);

        resetBuildFails();
        notify("CommitCat", "build succeeded! +15 XP 🔨");
      }),

      // ── Plugin: 빌드 실패 → surprised / angry 감정 ──
      listen("plugin:build-fail", () => {
        incrementBuildFails();
        const failCount = useCatStore.getState().recentBuildFailCount;
        if (failCount >= 3) {
          triggerEmotion("angry", 4000);
        } else {
          triggerEmotion("surprised", 3000);
        }
      }),

      // ── 풀스크린 ──
      listen<boolean>("activity:fullscreen", async (event) => {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const win = getCurrentWindow();
        if (event.payload) {
          await win.hide();
        } else {
          await win.show();
        }
      }),
    ]);

    return () => {
      unlisten.then((fns) => fns.forEach((fn) => fn()));
      if (codingTimer.current) clearInterval(codingTimer.current);
      if (emotionTimer.current) clearTimeout(emotionTimer.current);
      if (commitStreakTimer.current) clearTimeout(commitStreakTimer.current);
    };
  }, [setState, setActiveIde, setIdleSeconds, addCodingMinute, setLevel, triggerLevelUp,
      setEmotion, clearEmotion, incrementCommitStreak, resetCommitStreak, incrementBuildFails, resetBuildFails]);

  // 모자 잠금해제 이벤트
  useEffect(() => {
    const unlisten = listen<string[]>("hat:unlocked", (event) => {
      const hats = event.payload;
      const store = useCatStore.getState();
      hats.forEach(hat => store.addUnlockedHat(hat));
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // 레벨업 시 서브 고양이 동기화
  useEffect(() => {
    const unlisten = listen<number>("xp:level-up", async () => {
      try {
        const settings = await invoke<{ maxCompanions?: number }>("get_settings");
        const companions = settings.maxCompanions ?? 2;
        if (companions > 0) syncSubCats(companions);
      } catch (_) {}
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [syncSubCats]);

  // 활성 프로필 변경/편집 시 메인 고양이 갱신
  useEffect(() => {
    const unlisten = listen<CatProfile>("cat-profile:changed", async (event) => {
      const previousColor = useCatStore.getState().catColor;
      applyActiveProfile(event.payload);

      if (previousColor === event.payload.color) {
        return;
      }

      try {
        const settings = await invoke<{ maxCompanions?: number }>("get_settings");
        const companions = settings.maxCompanions ?? 2;
        setTimeout(() => useCatStore.getState().syncSubCats(companions), 0);
      } catch (_) {
        setTimeout(() => useCatStore.getState().syncSubCats(0), 0);
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [applyActiveProfile]);

  // 동료 고양이 수 변경 (설정에서 0/1/2)
  useEffect(() => {
    const unlisten = listen<number>("companions-change", (event) => {
      const companions = event.payload;
      if (companions > 0) {
        syncSubCats(companions);
      } else {
        // 모든 서브 고양이 제거
        const subs = useCatStore.getState().subCats;
        for (const sub of subs) {
          WebviewWindow.getByLabel(sub.id).then((win) => {
            if (win) win.close();
          });
        }
        useCatStore.setState({ subCats: [] });
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [syncSubCats]);

  // subCats 변경 → WebviewWindow 생성/제거
  const prevSubCatsRef = useRef<string[]>([]);

  useEffect(() => {
    const unsub = useCatStore.subscribe((state) => {
      const currentIds = state.subCats.map((s) => s.id);
      const prevIds = prevSubCatsRef.current;

      // 동일하면 스킵
      const currentKey = state.subCats.map((s) => `${s.id}:${s.color}`).join(",");
      const prevKey = prevIds.join(",");

      // ID만 비교해서 같으면 색상 변경인지 확인
      if (currentKey === prevKey) return;

      // 제거: 이전에 있었지만 현재 없는 윈도우
      for (const id of prevIds) {
        if (!currentIds.includes(id.split(":")[0])) {
          WebviewWindow.getByLabel(id.split(":")[0]).then((win) => {
            if (win) win.close();
          });
        }
      }

      // 색상이 바뀐 기존 윈도우 닫기 (재생성 위해)
      for (const sub of state.subCats) {
        const prev = prevIds.find((p) => p.startsWith(sub.id + ":"));
        if (prev && prev !== `${sub.id}:${sub.color}`) {
          WebviewWindow.getByLabel(sub.id).then((win) => {
            if (win) win.close();
          });
        }
      }

      // 생성: 현재 있지만 이전에 없거나 색상 변경된 윈도우
      for (const sub of state.subCats) {
        const prev = prevIds.find((p) => p.startsWith(sub.id + ":"));
        if (!prev || prev !== `${sub.id}:${sub.color}`) {
          // 약간의 딜레이로 닫힌 후 생성
          setTimeout(async () => {
            const existing = await WebviewWindow.getByLabel(sub.id);
            if (existing) return;
            const randomX = 100 + Math.floor(Math.random() * (window.screen.width - 300));
            const y = window.screen.availHeight - 150;
            const win = new WebviewWindow(sub.id, {
              url: `/?cat=sub&color=${sub.color}`,
              width: 200,
              height: 150,
              x: randomX,
              y,
              resizable: false,
              decorations: false,
              transparent: true,
              alwaysOnTop: true,
              skipTaskbar: true,
              shadow: false,
              visible: true,
            });
            // macOS 투명 설정 적용
            win.once("tauri://created", () => {
              invoke("setup_sub_cat_window", { label: sub.id }).catch(() => {});
            });
          }, prev ? 300 : 0);
        }
      }

      prevSubCatsRef.current = state.subCats.map((s) => `${s.id}:${s.color}`);
    });

    return () => unsub();
  }, []);

  // 풀스크린: 서브 고양이도 함께 숨기기/보이기
  useEffect(() => {
    const unlisten = listen<boolean>("activity:fullscreen", async (event) => {
      const subs = useCatStore.getState().subCats;
      for (const sub of subs) {
        const win = await WebviewWindow.getByLabel(sub.id);
        if (win) {
          if (event.payload) await win.hide();
          else await win.show();
        }
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  return <Cat />;
}

export default App;
