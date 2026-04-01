import { useState, useRef, useEffect, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { LogicalPosition } from "@tauri-apps/api/dpi";
import type { CatEmotion } from "../../stores/catStore";
import type { CatColor } from "../../types/cat";
import "./Cat.css";

const WIN_W = 200;
const meowMessages = ["meow~", "nya~", "mew!", "purr~", "mrrp!", "nyaa~"];

// 감정별 메시지 (메인과 다르게 간단한 리액션)
const emotionMessages: Record<string, string[]> = {
  surprised: ["huh?!", "what?!", "eek!"],
  excited: ["yay~!", "woohoo!", "amazing!"],
  proud: ["we did it!", "so cool~", "nice!"],
  bored: ["so bored...", "zzz...", "hmm..."],
  angry: ["grr!", "no way!", "hmph!"],
};

type Behavior = "walk" | "stand" | "sit" | "sleep";

export function SubCat({ color }: { color: CatColor }) {
  const appWindow = useRef(getCurrentWindow());
  const winPosRef = useRef({ x: 0, y: 0 });
  const [direction, setDirection] = useState<"left" | "right">(
    Math.random() > 0.5 ? "right" : "left"
  );
  const [behavior, setBehavior] = useState<Behavior>("walk");
  const [frame, setFrame] = useState(0);
  const [bubble, setBubble] = useState<string | null>(null);
  const [bubbleKey, setBubbleKey] = useState(0);
  const bubbleTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const screenW = useRef(window.screen.width);
  const maxY = useRef(window.screen.availHeight - 150);

  // 감정
  const [emotion, setEmotion] = useState<CatEmotion>(null);

  // 드래그
  const [isDragging, setIsDragging] = useState(false);
  const isDraggingRef = useRef(false);
  const didDrag = useRef(false);
  const dragStartMouse = useRef({ x: 0, y: 0 });
  const dragStartWin = useRef({ x: 0, y: 0 });

  // 투명 영역 클릭 통과
  const ignoreRef = useRef(false);
  const catRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const win = appWindow.current;
    const PAD = 30;
    const IGNORE_DELAY = 200;
    let busy = false;
    let queued: boolean | null = null;
    let ignoreTimer: ReturnType<typeof setTimeout> | null = null;

    const applyIgnore = async (ignore: boolean) => {
      if (busy) { queued = ignore; return; }
      busy = true;
      try { await win.setIgnoreCursorEvents(ignore); } catch (_) {}
      busy = false;
      if (queued !== null) {
        const next = queued;
        queued = null;
        applyIgnore(next);
      }
    };

    const onMove = (e: MouseEvent) => {
      if (isDraggingRef.current) return;
      const { clientX: mx, clientY: my } = e;
      let hit = false;
      if (catRef.current) {
        const r = catRef.current.getBoundingClientRect();
        if (mx >= r.left - PAD && mx <= r.right + PAD && my >= r.top - PAD && my <= r.bottom + PAD) {
          hit = true;
        }
      }
      if (!hit) {
        const el = document.elementFromPoint(mx, my) as HTMLElement | null;
        if (el?.closest(".cat__bubble")) hit = true;
      }
      const shouldIgnore = !hit;
      if (shouldIgnore === ignoreRef.current) return;

      if (shouldIgnore) {
        if (!ignoreTimer) {
          ignoreTimer = setTimeout(() => {
            ignoreTimer = null;
            ignoreRef.current = true;
            applyIgnore(true);
          }, IGNORE_DELAY);
        }
      } else {
        if (ignoreTimer) { clearTimeout(ignoreTimer); ignoreTimer = null; }
        ignoreRef.current = false;
        applyIgnore(false);
      }
    };
    document.addEventListener("mousemove", onMove);
    return () => document.removeEventListener("mousemove", onMove);
  }, []);

  // 초기 위치 설정
  useEffect(() => {
    (async () => {
      try {
        const pos = await appWindow.current.outerPosition();
        const scale = await appWindow.current.scaleFactor();
        winPosRef.current = { x: pos.x / scale, y: Math.min(pos.y / scale, maxY.current) };
      } catch (_) {}
    })();
  }, []);

  // 브라우저 기본 컨텍스트 메뉴 방지
  useEffect(() => {
    const prevent = (e: MouseEvent) => e.preventDefault();
    document.addEventListener("contextmenu", prevent);
    return () => document.removeEventListener("contextmenu", prevent);
  }, []);

  // 행동 전환 사이클
  useEffect(() => {
    let duration: number;
    let next: Behavior;

    if (behavior === "walk") {
      duration = 5000 + Math.random() * 5000;
      next = "stand";
    } else if (behavior === "stand") {
      if (Math.random() < 0.5) {
        duration = 2000 + Math.random() * 3000;
        next = "walk";
      } else {
        duration = 2000 + Math.random() * 2000;
        next = "sit";
      }
    } else if (behavior === "sit") {
      if (Math.random() < 0.5) {
        duration = 2000 + Math.random() * 2000;
        next = "sleep";
      } else {
        duration = 5000 + Math.random() * 3000;
        next = "stand";
      }
    } else {
      duration = 10000 + Math.random() * 5000;
      next = "stand";
    }

    const id = setTimeout(() => {
      setBehavior(next);
      if (next === "walk") {
        setDirection(Math.random() > 0.5 ? "right" : "left");
      }
    }, duration);
    return () => clearTimeout(id);
  }, [behavior]);

  // 걷기 프레임
  useEffect(() => {
    if (behavior !== "walk") { setFrame(0); return; }
    const id = setInterval(() => setFrame((prev) => (prev === 0 ? 1 : 0)), 1000);
    return () => clearInterval(id);
  }, [behavior]);

  // 감정 이벤트 수신 (메인 고양이에서 브로드캐스트)
  useEffect(() => {
    const unlisten = listen<{ emotion: CatEmotion; duration: number }>("sub-cat:emotion", (event) => {
      const { emotion: newEmotion } = event.payload;
      setEmotion(newEmotion);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // 윈도우 이동
  const moveWindow = useCallback(async (x: number, y: number) => {
    winPosRef.current = { x, y };
    try {
      await appWindow.current.setPosition(new LogicalPosition(Math.round(x), Math.round(y)));
    } catch (_) {}
  }, []);

  // 걸어다니기
  useEffect(() => {
    if (isDragging || behavior !== "walk" || frame === 0) return;
    const id = setInterval(() => {
      const pos = winPosRef.current;
      const speed = 0.75;
      let newX = pos.x + (direction === "right" ? speed : -speed);
      const maxX = screenW.current - WIN_W;
      if (newX > maxX) { setDirection("left"); newX = maxX; }
      else if (newX < 0) { setDirection("right"); newX = 0; }
      moveWindow(newX, pos.y);
    }, 30);
    return () => clearInterval(id);
  }, [direction, isDragging, moveWindow, frame, behavior]);

  // 말풍선
  const showBubble = useCallback((msg: string, duration = 2000) => {
    setBubble(msg);
    setBubbleKey((k) => k + 1);
    if (bubbleTimer.current) clearTimeout(bubbleTimer.current);
    bubbleTimer.current = setTimeout(() => setBubble(null), duration);
  }, []);

  // 감정 변경 시 말풍선 표시
  useEffect(() => {
    if (!emotion) return;
    const msgs = emotionMessages[emotion];
    if (msgs) {
      // 약간의 랜덤 딜레이로 메인과 동시에 안 뜨게
      const delay = 300 + Math.random() * 700;
      const timer = setTimeout(() => {
        showBubble(msgs[Math.floor(Math.random() * msgs.length)], emotion === "bored" ? 4000 : 3000);
      }, delay);
      return () => clearTimeout(timer);
    }
  }, [emotion, showBubble]);

  // 드래그
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return;
    isDraggingRef.current = true;
    ignoreRef.current = false;
    setIsDragging(true);
    appWindow.current.setIgnoreCursorEvents(false).catch(() => {});
    didDrag.current = false;
    dragStartMouse.current = { x: e.screenX, y: e.screenY };
    dragStartWin.current = { ...winPosRef.current };
  }, []);

  useEffect(() => {
    if (!isDragging) return;
    const handleMove = (e: MouseEvent) => {
      didDrag.current = true;
      const dx = e.screenX - dragStartMouse.current.x;
      const dy = e.screenY - dragStartMouse.current.y;
      moveWindow(dragStartWin.current.x + dx, dragStartWin.current.y + dy);
    };
    const handleUp = () => {
      setIsDragging(false);
      isDraggingRef.current = false;
      if (didDrag.current) showBubble("wheee~!", 1500);
    };
    window.addEventListener("mousemove", handleMove);
    window.addEventListener("mouseup", handleUp);
    return () => {
      window.removeEventListener("mousemove", handleMove);
      window.removeEventListener("mouseup", handleUp);
    };
  }, [isDragging, moveWindow, showBubble]);

  // 클릭 → meow
  const handleClick = () => {
    if (didDrag.current) return;
    const msg = meowMessages[Math.floor(Math.random() * meowMessages.length)];
    showBubble(msg);
  };

  // 이미지 경로
  const getImageSrc = () => {
    if (behavior === "sleep") return `/assets/cat/${color}_sit2.png`;
    if (behavior === "sit") return `/assets/cat/${color}_sit.png`;
    if (behavior === "stand") return `/assets/cat/${color}_stand.png`;
    return frame === 0
      ? `/assets/cat/${color}_stand.png`
      : `/assets/cat/${color}_walk.png`;
  };

  const isFlipped = direction === "right";

  return (
    <div className="cat-window">
      <div className="bubble-area">
        {bubble && (
          <div className="cat__bubble" key={bubbleKey}>{bubble}</div>
        )}
      </div>
      <div
        ref={catRef}
        className={`cat ${isDragging ? "cat--dragging" : ""} ${emotion ? `cat--emotion-${emotion}` : ""}`}
        onMouseDown={handleMouseDown}
        onClick={handleClick}
      >
        <div
          className="cat__sprite"
          style={{ transform: isFlipped ? "scaleX(-1)" : "scaleX(1)" }}
        >
          <img
            className={`cat__image cat__image--${color} ${behavior === "walk" && frame === 1 ? "cat__image--walk" : ""} ${behavior === "sit" ? "cat__image--sit" : ""} ${behavior === "sleep" ? "cat__image--sleep" : ""}`}
            src={getImageSrc()}
            alt="sub cat"
            draggable={false}
          />
        </div>
        {behavior === "sleep" && (
          <div className="cat__zzz" style={direction === "right" ? { left: "auto", right: "5px" } : undefined}>z z z</div>
        )}
      </div>
    </div>
  );
}
