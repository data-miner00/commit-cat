import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import {
  CAT_COLORS,
  CAT_PERSONALITY_OPTIONS,
  type CatProfile,
  type CatProfilesResponse,
} from "../../types/cat";
import "./Settings.css";

interface XpStatus {
  level: number;
  currentExp: number;
  expToNext: number;
}

type AIProvider = "claude" | "openai-api" | "openai-codex-local";

interface CodexProviderStatus {
  available: boolean;
  authenticated: boolean;
  connected: boolean;
  statusMessage: string;
}

interface SettingsPayload {
  pomodoroMinutes?: number;
  breakMinutes?: number;
  githubUsername?: string | null;
  notificationsEnabled?: boolean;
  anthropicApiKey?: string | null;
  openaiApiKey?: string | null;
  aiProvider?: string;
  aiProviderModels?: Record<string, string>;
  aiProviderReasoning?: Record<string, string>;
  maxCompanions?: number;
  birthdayMonth?: number | null;
  birthdayDay?: number | null;
}

interface AIProviderModelOption {
  id: string;
  label: string;
  reasoningEfforts?: AIReasoningOption[];
  defaultReasoning?: string;
}

interface AIReasoningOption {
  id: string;
  label: string;
}

interface AIProviderCatalogEntry {
  id: AIProvider;
  label: string;
  description: string;
  defaultModel: string;
  models: AIProviderModelOption[];
}

interface AIProviderCatalogResponse {
  providers: AIProviderCatalogEntry[];
}

function normalizeAiProvider(value?: string | null): AIProvider {
  if (value === "openai" || value === "openai-api") return "openai-api";
  if (value === "openai-codex-local") return "openai-codex-local";
  return "claude";
}

function normalizeAiProviderModels(value?: Record<string, string> | null): Record<string, string> {
  const next = { ...(value ?? {}) };
  if (next.openai && !next["openai-api"]) {
    next["openai-api"] = next.openai;
  }
  delete next.openai;
  return next;
}

function reasoningStorageKey(providerId: AIProvider, modelId: string): string {
  return `${providerId}::${modelId}`;
}

function migrateLegacyReasoningMap(
  reasoning: Record<string, string>,
  providerModels: Record<string, string>,
  catalog: AIProviderCatalogEntry[],
): Record<string, string> {
  const next: Record<string, string> = {};
  for (const [rawKey, value] of Object.entries(reasoning)) {
    const trimmedValue = value.trim();
    if (!trimmedValue) continue;

    if (rawKey.includes("::")) {
      const [rawProvider, rawModel] = rawKey.split("::", 2);
      if (!rawModel) continue;
      next[reasoningStorageKey(normalizeAiProvider(rawProvider), rawModel)] = trimmedValue;
      continue;
    }

    const provider = normalizeAiProvider(rawKey);
    const providerEntry = catalog.find(candidate => candidate.id === provider);
    const selectedModel = providerModels[provider] ?? providerEntry?.defaultModel ?? "";
    if (!selectedModel) continue;
    next[reasoningStorageKey(provider, selectedModel)] = trimmedValue;
  }
  return next;
}

function resolveSelectedModel(
  providerId: AIProvider,
  providerModels: Record<string, string>,
  provider?: AIProviderCatalogEntry | null,
): string {
  if (!provider) return "";
  const selectedModel = providerModels[providerId];
  if (selectedModel && provider.models.some(model => model.id === selectedModel)) {
    return selectedModel;
  }
  return provider.defaultModel;
}

function resolveSelectedReasoning(
  providerId: AIProvider,
  providerReasoning: Record<string, string>,
  model?: AIProviderModelOption,
): string {
  const options = model?.reasoningEfforts ?? [];
  if (options.length === 0) return "";
  const selectedReasoning = providerReasoning[reasoningStorageKey(providerId, model?.id ?? "")];
  if (!selectedReasoning || selectedReasoning === model?.defaultReasoning) {
    if (model?.defaultReasoning) {
      return "";
    }
    return options[0]?.id ?? "";
  }
  if (selectedReasoning && options.some(option => option.id === selectedReasoning)) {
    return selectedReasoning;
  }
  if (!model?.defaultReasoning) {
    return options[0]?.id ?? "";
  }
  return "";
}

const HAT_DEFINITIONS = [
  { id: "party_hat", name: "Party Hat", image: "/assets/item/party_hat.png", condition: "First commit ever" },
  { id: "wizard", name: "Wizard Hat", image: "/assets/item/wizard.png", condition: "Reach level 5" },
  { id: "crown", name: "Crown", image: "/assets/item/crown.png", condition: "Reach level 10" },
  { id: "tophat", name: "Top Hat", image: "/assets/item/tophat.png", condition: "7-day streak" },
  { id: "santahat", name: "Santa Hat", image: "/assets/item/santahat.png", condition: "Code in December" },
  { id: "sunglass", name: "Sunglasses", image: "/assets/item/sunglass.png", condition: "10 late-night sessions" },
  { id: "tuna", name: "Tuna", image: "/assets/item/tuna.png", condition: "50 total commits" },
  { id: "cornhead", name: "Corn Head", image: "/assets/item/cornhead.png", condition: "30-day streak" },
];

export function Settings() {
  const [repos, setRepos] = useState<string[]>([]);
  const [profiles, setProfiles] = useState<CatProfile[]>([]);
  const [activeProfileId, setActiveProfileId] = useState("");
  const [selectedProfileId, setSelectedProfileId] = useState("");
  const [profileError, setProfileError] = useState("");
  const [focusMinutes, setFocusMinutes] = useState(25);
  const [breakMinutes, setBreakMinutes] = useState(5);
  const [loading, setLoading] = useState(true);
  const [birthdayMonth, setBirthdayMonth] = useState<number | null>(null);
  const [birthdayDay, setBirthdayDay] = useState<number | null>(null);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [cloneModalOpen, setCloneModalOpen] = useState(false);
  const [cloneUrl, setCloneUrl] = useState("");
  const [clonePath, setClonePath] = useState("");
  const [cloning, setCloning] = useState(false);
  const [cloneError, setCloneError] = useState("");
  const dropdownRef = useRef<HTMLDivElement>(null);
  const [xp, setXp] = useState<XpStatus>({ level: 1, currentExp: 0, expToNext: 100 });
  const [githubToken, setGithubToken] = useState("");
  const [githubUsername, setGithubUsername] = useState<string | null>(null);
  const [githubConnecting, setGithubConnecting] = useState(false);
  const [githubError, setGithubError] = useState("");
  const [notificationsEnabled, setNotificationsEnabled] = useState(true);
  const [maxCompanions, setMaxCompanions] = useState(2);
  const [aiKey, setAiKey] = useState("");
  const [aiKeySaved, setAiKeySaved] = useState(false);
  const [aiSaving, setAiSaving] = useState(false);
  const [openaiKey, setOpenaiKey] = useState("");
  const [openaiKeySaved, setOpenaiKeySaved] = useState(false);
  const [aiProvider, setAiProvider] = useState<AIProvider>("claude");
  const [aiProviderModels, setAiProviderModels] = useState<Record<string, string>>({});
  const [aiProviderReasoning, setAiProviderReasoning] = useState<Record<string, string>>({});
  const [aiProviderCatalog, setAiProviderCatalog] = useState<AIProviderCatalogEntry[]>([]);
  const [codexStatus, setCodexStatus] = useState<CodexProviderStatus>({
    available: false,
    authenticated: false,
    connected: false,
    statusMessage: "Checking local Codex provider...",
  });
  const [codexChecking, setCodexChecking] = useState(false);
  const [currentHat, setCurrentHat] = useState<string | null>(null);
  const [unlockedHats, setUnlockedHats] = useState<string[]>([]);
  const [itemDebugMode, setItemDebugMode] = useState(false);
  const [debugLog, setDebugLog] = useState("");
  const [allSavedAnchors, setAllSavedAnchors] = useState<Record<string, { dy: number; dx: number }>>({});

  // 디버그 모드: 키보드 → Cat 윈도우로 전달
  useEffect(() => {
    if (!itemDebugMode) return;
    const handler = (e: KeyboardEvent) => {
      const prefix = e.shiftKey ? "shift_" : "";
      if (e.key === "ArrowUp") { e.preventDefault(); emit("item:debug:key", prefix + "up"); }
      if (e.key === "ArrowDown") { e.preventDefault(); emit("item:debug:key", prefix + "down"); }
      if (e.key === "ArrowLeft") { e.preventDefault(); emit("item:debug:key", prefix + "left"); }
      if (e.key === "ArrowRight") { e.preventDefault(); emit("item:debug:key", prefix + "right"); }
      if (e.key === "Enter") { e.preventDefault(); emit("item:debug:key", "enter"); }
      if (e.key === "r" || e.key === "R") { emit("item:debug:key", "reset"); }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [itemDebugMode]);

  // 디버그 확정 결과 수신
  useEffect(() => {
    const unlisten = listen<string>("item:debug:saved", (event) => {
      try {
        const deltas = JSON.parse(event.payload) as Record<string, { dy: number; dx: number }>;
        setAllSavedAnchors(deltas);
        const entries = Object.entries(deltas).map(([k, v]) => `${k}(${v.dy},${v.dx})`).join(" ");
        setDebugLog(entries || "리셋됨");
      } catch {
        setDebugLog(event.payload);
      }
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  const refreshCodexStatus = useCallback(async () => {
    setCodexChecking(true);
    try {
      const status = await invoke<CodexProviderStatus>("get_codex_provider_status");
      setCodexStatus(status);
    } catch (e) {
      setCodexStatus({
        available: false,
        authenticated: false,
        connected: false,
        statusMessage: String(e),
      });
    } finally {
      setCodexChecking(false);
    }
  }, []);

  // 초기 데이터 로드
  useEffect(() => {
    void refreshCodexStatus();
  }, [refreshCodexStatus]);

  const persistSettingsPatch = useCallback(async (patch: Record<string, unknown>) => {
    await invoke("update_settings_patch", { patch });
  }, []);

  const applyCatProfilesResponse = useCallback((response: CatProfilesResponse, preferredSelectedId?: string) => {
    setProfiles(response.profiles);
    setActiveProfileId(response.activeProfileId);
    const nextSelectedId = preferredSelectedId
      ?? (response.profiles.some(profile => profile.id === selectedProfileId) ? selectedProfileId : response.activeProfileId);
    setSelectedProfileId(nextSelectedId);
    setProfileError("");
  }, [selectedProfileId]);

  useEffect(() => {
    (async () => {
      try {
        const [repoList, settings, xpStatus, catalog, profileResponse] = await Promise.all([
          invoke<string[]>("get_watched_repos"),
          invoke<SettingsPayload>("get_settings"),
          invoke<XpStatus>("get_xp_status"),
          invoke<AIProviderCatalogResponse>("get_ai_provider_catalog"),
          invoke<CatProfilesResponse>("get_cat_profiles"),
        ]);
        setRepos(repoList);
        if (settings.pomodoroMinutes) setFocusMinutes(settings.pomodoroMinutes);
        if (settings.breakMinutes) setBreakMinutes(settings.breakMinutes);
        if (settings.githubUsername) setGithubUsername(settings.githubUsername);
        if (settings.notificationsEnabled !== undefined) setNotificationsEnabled(settings.notificationsEnabled);
        if (settings.maxCompanions !== undefined) setMaxCompanions(settings.maxCompanions);
        if (settings.birthdayMonth) setBirthdayMonth(settings.birthdayMonth);
        if (settings.birthdayDay) setBirthdayDay(settings.birthdayDay);
        if (settings.anthropicApiKey) setAiKeySaved(true);
        if (settings.openaiApiKey) setOpenaiKeySaved(true);
        const normalizedProvider = normalizeAiProvider(settings.aiProvider);
        const normalizedModels = normalizeAiProviderModels(settings.aiProviderModels);
        const normalizedReasoning = migrateLegacyReasoningMap(
          { ...(settings.aiProviderReasoning ?? {}) },
          normalizedModels,
          catalog.providers,
        );
        setAiProvider(normalizedProvider);
        setAiProviderModels(normalizedModels);
        setAiProviderReasoning(normalizedReasoning);
        setAiProviderCatalog(catalog.providers);
        setXp(xpStatus);
        applyCatProfilesResponse(profileResponse, profileResponse.activeProfileId);

        // Load hat info
        invoke<{ currentHat: string | null; unlockedHats: string[] }>("get_hat_info")
          .then(info => {
            setCurrentHat(info.currentHat);
            setUnlockedHats(info.unlockedHats);
          })
          .catch(() => {});
      } catch (e) {
        console.error("Failed to load settings:", e);
      } finally {
        setLoading(false);
      }
    })();
  }, [applyCatProfilesResponse]);

  // 외부 클릭 시 드롭다운 닫기
  useEffect(() => {
    if (!dropdownOpen) return;
    const handleClick = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setDropdownOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [dropdownOpen]);

  // 기존 폴더 선택으로 Repo 추가
  const handleAddExisting = useCallback(async () => {
    setDropdownOpen(false);
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Git Repository",
      });
      if (!selected) return;

      const path = selected as string;
      await invoke("register_repo", { path });
      const updated = await invoke<string[]>("get_watched_repos");
      setRepos(updated);
    } catch (e) {
      console.error("Failed to add repo:", e);
    }
  }, []);

  // Clone 모달 열기
  const handleOpenCloneModal = useCallback(() => {
    setDropdownOpen(false);
    setCloneUrl("");
    setClonePath("");
    setCloneError("");
    setCloneModalOpen(true);
  }, []);

  // Clone 경로 선택
  const handleChooseClonePath = useCallback(async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Clone Destination",
      });
      if (selected) {
        setClonePath(selected as string);
      }
    } catch (e) {
      console.error("Failed to choose path:", e);
    }
  }, []);

  // Clone 실행
  const handleClone = useCallback(async () => {
    if (!cloneUrl.trim() || !clonePath.trim()) return;

    // URL에서 repo 이름 추출하여 하위 폴더로 사용
    const repoName = cloneUrl.trim().replace(/\.git$/, "").split("/").pop() || "repo";
    const fullPath = `${clonePath.replace(/\/$/, "")}/${repoName}`;

    setCloning(true);
    setCloneError("");
    try {
      await invoke("clone_repo", { url: cloneUrl.trim(), path: fullPath });
      const updated = await invoke<string[]>("get_watched_repos");
      setRepos(updated);
      setCloneModalOpen(false);
    } catch (e) {
      setCloneError(String(e));
    } finally {
      setCloning(false);
    }
  }, [cloneUrl, clonePath]);

  // Repo 삭제
  const handleRemoveRepo = useCallback(async (path: string) => {
    try {
      await invoke("remove_repo", { path });
      setRepos(prev => prev.filter(r => r !== path));
    } catch (e) {
      console.error("Failed to remove repo:", e);
    }
  }, []);

  // Timer 설정 변경
  const handleFocusChange = useCallback(async (val: number) => {
    const clamped = Math.max(1, Math.min(120, val));
    setFocusMinutes(clamped);
    try {
      await persistSettingsPatch({ pomodoroMinutes: clamped });
    } catch (e) {
      console.error("Failed to update focus minutes:", e);
    }
  }, [persistSettingsPatch]);

  const handleBreakChange = useCallback(async (val: number) => {
    const clamped = Math.max(1, Math.min(60, val));
    setBreakMinutes(clamped);
    try {
      await persistSettingsPatch({ breakMinutes: clamped });
    } catch (e) {
      console.error("Failed to update break minutes:", e);
    }
  }, [persistSettingsPatch]);

  // GitHub 연결
  const handleGithubConnect = useCallback(async () => {
    if (!githubToken.trim()) return;
    setGithubConnecting(true);
    setGithubError("");
    try {
      const username = await invoke<string>("verify_github_token", { token: githubToken.trim() });
      setGithubUsername(username);
      setGithubToken("");
    } catch (e) {
      setGithubError(String(e));
    } finally {
      setGithubConnecting(false);
    }
  }, [githubToken]);

  // GitHub 연결 해제
  const handleGithubDisconnect = useCallback(async () => {
    try {
      await invoke("disconnect_github");
      setGithubUsername(null);
    } catch (e) {
      console.error("Failed to disconnect GitHub:", e);
    }
  }, []);

  // 알림 토글
  const handleNotificationsToggle = useCallback(async () => {
    const next = !notificationsEnabled;
    setNotificationsEnabled(next);
    try {
      await persistSettingsPatch({ notificationsEnabled: next });
    } catch (e) {
      console.error("Failed to update notifications:", e);
    }
  }, [notificationsEnabled, persistSettingsPatch]);

  // 동료 고양이 수 변경
  const handleCompanionsChange = useCallback(async (value: number) => {
    const clamped = Math.max(0, Math.min(2, value));
    setMaxCompanions(clamped);
    try {
      await persistSettingsPatch({ maxCompanions: clamped });
      await emit("companions-change", clamped);
    } catch (e) {
      console.error("Failed to update companions:", e);
    }
  }, [persistSettingsPatch]);

  // AI API Key 저장
  const handleAiSave = useCallback(async () => {
    if (!aiKey.trim()) return;
    setAiSaving(true);
    try {
      await persistSettingsPatch({ anthropicApiKey: aiKey.trim() });
      setAiKeySaved(true);
      setAiKey("");
    } catch (e) {
      console.error("Failed to save AI key:", e);
    } finally {
      setAiSaving(false);
    }
  }, [aiKey, persistSettingsPatch]);

  // AI API Key 삭제
  const handleAiRemove = useCallback(async () => {
    try {
      await persistSettingsPatch({ anthropicApiKey: null });
      setAiKeySaved(false);
    } catch (e) {
      console.error("Failed to remove AI key:", e);
    }
  }, [persistSettingsPatch]);

  // OpenAI API Key 저장
  const handleOpenaiSave = useCallback(async () => {
    if (!openaiKey.trim()) return;
    try {
      await persistSettingsPatch({ openaiApiKey: openaiKey.trim() });
      setOpenaiKeySaved(true);
      setOpenaiKey("");
    } catch (e) {
      console.error("Failed to save OpenAI key:", e);
    }
  }, [openaiKey, persistSettingsPatch]);

  // OpenAI API Key 삭제
  const handleOpenaiRemove = useCallback(async () => {
    try {
      await persistSettingsPatch({ openaiApiKey: null });
      setOpenaiKeySaved(false);
    } catch (e) {
      console.error("Failed to remove OpenAI key:", e);
    }
  }, [persistSettingsPatch]);

  // AI Provider 변경
  const handleProviderChange = useCallback(async (provider: AIProvider) => {
    setAiProvider(provider);
    try {
      await persistSettingsPatch({ aiProvider: provider });
    } catch (e) {
      console.error("Failed to update AI provider:", e);
    }
  }, [persistSettingsPatch]);

  const handleModelChange = useCallback(async (modelId: string) => {
    const nextModels = {
      ...aiProviderModels,
      [aiProvider]: modelId,
    };
    setAiProviderModels(nextModels);
    try {
      await persistSettingsPatch({ aiProviderModels: nextModels });
    } catch (e) {
      console.error("Failed to update AI model:", e);
      setAiProviderModels(aiProviderModels);
    }
  }, [aiProvider, aiProviderModels, persistSettingsPatch]);

  const handleReasoningChange = useCallback(async (reasoningId: string) => {
    const providerEntry = aiProviderCatalog.find(provider => provider.id === aiProvider) ?? null;
    const modelId = resolveSelectedModel(aiProvider, aiProviderModels, providerEntry);
    if (!modelId) return;
    const nextReasoning = { ...aiProviderReasoning };
    const storageKey = reasoningStorageKey(aiProvider, modelId);
    if (reasoningId) {
      nextReasoning[storageKey] = reasoningId;
    } else {
      delete nextReasoning[storageKey];
    }
    setAiProviderReasoning(nextReasoning);
    try {
      await persistSettingsPatch({ aiProviderReasoning: nextReasoning });
    } catch (e) {
      console.error("Failed to update AI reasoning:", e);
      setAiProviderReasoning(aiProviderReasoning);
    }
  }, [aiProvider, aiProviderCatalog, aiProviderModels, aiProviderReasoning, persistSettingsPatch]);

  const handleCreateProfile = useCallback(async () => {
    try {
      const response = await invoke<CatProfilesResponse>("create_cat_profile");
      applyCatProfilesResponse(response, response.activeProfileId);
    } catch (e) {
      setProfileError(String(e));
    }
  }, [applyCatProfilesResponse]);

  const handleActivateProfile = useCallback(async (profileId: string) => {
    try {
      await invoke("set_active_cat_profile", { profileId });
      setActiveProfileId(profileId);
      setSelectedProfileId(profileId);
      setProfileError("");
    } catch (e) {
      setProfileError(String(e));
    }
  }, []);

  const handleDeleteProfile = useCallback(async (profileId: string) => {
    try {
      const response = await invoke<CatProfilesResponse>("delete_cat_profile", { profileId });
      applyCatProfilesResponse(response);
    } catch (e) {
      setProfileError(String(e));
    }
  }, [applyCatProfilesResponse]);

  const handleSelectedProfilePatch = useCallback(async (patch: Partial<CatProfile>) => {
    const profile = profiles.find(candidate => candidate.id === selectedProfileId);
    if (!profile) return;
    const nextProfile = { ...profile, ...patch };
    setProfiles(current => current.map(candidate => (
      candidate.id === nextProfile.id ? nextProfile : candidate
    )));
    try {
      const response = await invoke<CatProfilesResponse>("update_cat_profile", { profile: nextProfile });
      applyCatProfilesResponse(response, nextProfile.id);
    } catch (e) {
      const latest = await invoke<CatProfilesResponse>("get_cat_profiles").catch(() => null);
      if (latest) {
        applyCatProfilesResponse(latest, nextProfile.id);
      }
      setProfileError(String(e));
    }
  }, [applyCatProfilesResponse, profiles, selectedProfileId]);

  // 모자 장착/해제
  const handleHatToggle = useCallback(async (hatId: string) => {
    const newHat = currentHat === hatId ? null : hatId;
    setCurrentHat(newHat);
    try {
      await invoke("equip_hat", { hatId: newHat });
      await emit("hat:equipped", newHat);
    } catch (e) {
      console.error("Failed to equip hat:", e);
    }
  }, [currentHat]);

  // 경로 축약 (홈 디렉토리 → ~)
  const shortenPath = (path: string) => {
    const home = path.match(/^\/Users\/[^/]+/)?.[0];
    if (home) return path.replace(home, "~");
    return path;
  };

  if (loading) {
    return <div className="settings"><div className="settings__loading">Loading...</div></div>;
  }

  const selectedProfile = profiles.find(profile => profile.id === selectedProfileId) ?? profiles[0] ?? null;
  const selectedProvider = aiProviderCatalog.find(provider => provider.id === aiProvider) ?? aiProviderCatalog[0] ?? null;
  const selectedModel = resolveSelectedModel(aiProvider, aiProviderModels, selectedProvider);
  const selectedModelEntry = selectedProvider?.models.find(model => model.id === selectedModel);
  const selectedReasoning = resolveSelectedReasoning(aiProvider, aiProviderReasoning, selectedModelEntry);
  const codexStatusLabel = codexStatus.connected
    ? "Connected"
    : codexStatus.authenticated
      ? "CLI Missing"
      : codexStatus.available
        ? "Not Connected"
        : "Unavailable";

  const renderProviderConfiguration = () => {
    if (aiProvider === "claude") {
      return (
        <div className="settings__ai-key-section">
          <div className="settings__ai-key-label">Anthropic API Key</div>
          {aiKeySaved ? (
            <div className="settings__github-status">
              <span>Connected</span>
              <button
                className="settings__github-btn settings__github-btn--disconnect"
                onClick={handleAiRemove}
              >
                Remove
              </button>
            </div>
          ) : (
            <div className="settings__github-row">
              <input
                className="settings__github-input"
                type="password"
                placeholder="sk-ant-..."
                value={aiKey}
                onChange={e => setAiKey(e.target.value)}
                disabled={aiSaving}
                onKeyDown={e => e.key === "Enter" && handleAiSave()}
              />
              <button
                className="settings__github-btn settings__github-btn--connect"
                onClick={handleAiSave}
                disabled={aiSaving || !aiKey.trim()}
              >
                {aiSaving ? "..." : "Save"}
              </button>
            </div>
          )}
        </div>
      );
    }

    if (aiProvider === "openai-api") {
      return (
        <div className="settings__ai-key-section">
          <div className="settings__ai-key-label">OpenAI API Key</div>
          {openaiKeySaved ? (
            <div className="settings__github-status">
              <span>Connected</span>
              <button
                className="settings__github-btn settings__github-btn--disconnect"
                onClick={handleOpenaiRemove}
              >
                Remove
              </button>
            </div>
          ) : (
            <div className="settings__github-row">
              <input
                className="settings__github-input"
                type="password"
                placeholder="sk-..."
                value={openaiKey}
                onChange={e => setOpenaiKey(e.target.value)}
                onKeyDown={e => e.key === "Enter" && handleOpenaiSave()}
              />
              <button
                className="settings__github-btn settings__github-btn--connect"
                onClick={handleOpenaiSave}
                disabled={!openaiKey.trim()}
              >
                Save
              </button>
            </div>
          )}
        </div>
      );
    }

    return (
      <div className="settings__provider-card">
        <div className="settings__provider-card-header">
          <div>
            <div className="settings__ai-key-label">Local Provider Status</div>
            <div className={`settings__provider-status ${codexStatus.connected ? "settings__provider-status--connected" : ""}`}>
              {codexStatusLabel}
            </div>
          </div>
          <div className="settings__provider-card-actions">
            <button
              className="settings__github-btn settings__github-btn--disconnect"
              onClick={refreshCodexStatus}
              disabled={codexChecking}
            >
              {codexChecking ? "..." : "Refresh"}
            </button>
          </div>
        </div>

        <div className="settings__provider-note">
          {codexStatus.statusMessage}
        </div>

        {!codexStatus.connected && (
          <div className="settings__provider-steps">
            <div className="settings__ai-key-label">Connection</div>
            <p className="settings__provider-step">
              1. Install the Codex CLI on this machine.
            </p>
            <p className="settings__provider-step">
              2. Run <code>codex login</code> in your terminal and complete the ChatGPT OAuth flow.
            </p>
            <p className="settings__provider-step">
              3. Return here and press Refresh.
            </p>
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="settings">
      <h1 className="settings__title">CommitCat Settings</h1>

      {/* Watched Repositories */}
      <section className="settings__section">
        <h2 className="settings__section-title">Watched Repositories</h2>
        <div className="settings__repo-list">
          {repos.length === 0 ? (
            <div className="settings__repo-empty">
              No repositories registered. Add one to track commits.
            </div>
          ) : (
            repos.map(repo => (
              <div key={repo} className="settings__repo-item">
                <span className="settings__repo-path" title={repo}>
                  {shortenPath(repo)}
                </span>
                <button
                  className="settings__repo-remove"
                  onClick={() => handleRemoveRepo(repo)}
                  title="Remove repository"
                >
                  &times;
                </button>
              </div>
            ))
          )}
        </div>
        <div className="settings__repo-add-wrap" ref={dropdownRef}>
          <button
            className="settings__repo-add"
            onClick={() => setDropdownOpen(prev => !prev)}
          >
            + Add Repository
          </button>
          {dropdownOpen && (
            <div className="settings__dropdown">
              <button className="settings__dropdown-item" onClick={handleOpenCloneModal}>
                Clone Repository...
              </button>
              <button className="settings__dropdown-item" onClick={handleAddExisting}>
                Add Existing Repository...
              </button>
            </div>
          )}
        </div>
      </section>

      {/* Level */}
      <section className="settings__section">
        <h2 className="settings__section-title">Level</h2>
        <div className="settings__level-row">
          <span className="settings__level-label">Lv.{xp.level}</span>
          <div className="settings__xp-bar">
            <div
              className="settings__xp-fill"
              style={{ width: `${xp.expToNext > 0 ? (xp.currentExp / xp.expToNext) * 100 : 0}%` }}
            />
          </div>
          <span className="settings__xp-text">{xp.currentExp} / {xp.expToNext}</span>
        </div>
      </section>

      {/* Cat Settings */}
      <section className="settings__section">
        <h2 className="settings__section-title">Cat</h2>
        <div className="settings__profile-toolbar">
          <span className="settings__color-label">Profiles</span>
          <button className="settings__profile-add" onClick={handleCreateProfile}>
            + New Profile
          </button>
        </div>
        <div className="settings__profile-list">
          {profiles.map(profile => {
            const personalityLabel = CAT_PERSONALITY_OPTIONS.find(option => option.id === profile.personality)?.label ?? profile.personality;
            const isSelected = selectedProfile?.id === profile.id;
            const isActive = activeProfileId === profile.id;
            return (
              <div
                key={profile.id}
                className={`settings__profile-card ${isSelected ? "settings__profile-card--selected" : ""}`}
                onClick={() => setSelectedProfileId(profile.id)}
              >
                <div className="settings__profile-card-main">
                  <div className="settings__profile-card-title">
                    <span>{profile.name}</span>
                    {isActive && <span className="settings__profile-badge">Active</span>}
                  </div>
                  <div className="settings__profile-card-meta">
                    <span className={`settings__profile-color settings__profile-color--${profile.color}`} />
                    <span>{personalityLabel}</span>
                  </div>
                </div>
                <div className="settings__profile-card-actions">
                  {!isActive && (
                    <button
                      className="settings__profile-action"
                      onClick={(e) => {
                        e.stopPropagation();
                        void handleActivateProfile(profile.id);
                      }}
                    >
                      Set Active
                    </button>
                  )}
                  <button
                    className="settings__profile-action settings__profile-action--danger"
                    disabled={profiles.length <= 1}
                    onClick={(e) => {
                      e.stopPropagation();
                      void handleDeleteProfile(profile.id);
                    }}
                  >
                    Delete
                  </button>
                </div>
              </div>
            );
          })}
        </div>
        {profileError && <div className="settings__profile-error">{profileError}</div>}
        {selectedProfile && (
          <div className="settings__profile-editor">
            <label className="settings__profile-field">
              <span className="settings__color-label">Name</span>
              <input
                className="settings__profile-input"
                value={selectedProfile.name}
                onChange={(e) => {
                  const nextName = e.target.value;
                  setProfiles(current => current.map(profile => (
                    profile.id === selectedProfile.id ? { ...profile, name: nextName } : profile
                  )));
                }}
                onBlur={(e) => { void handleSelectedProfilePatch({ name: e.target.value }); }}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.currentTarget.blur();
                  }
                }}
                placeholder="My Cat"
              />
            </label>
            <div className="settings__color-row">
              <span className="settings__color-label">Color</span>
              <div className="settings__color-options">
                {CAT_COLORS.map(color => (
                  <button
                    key={color}
                    className={`settings__color-btn settings__color-btn--${color} ${selectedProfile.color === color ? "settings__color-btn--active" : ""}`}
                    onClick={() => { void handleSelectedProfilePatch({ color }); }}
                    title={color}
                  />
                ))}
              </div>
            </div>
            <label className="settings__profile-field">
              <span className="settings__color-label">Personality</span>
              <select
                className="settings__profile-select"
                value={selectedProfile.personality}
                onChange={(e) => { void handleSelectedProfilePatch({ personality: e.target.value as CatProfile["personality"] }); }}
              >
                {CAT_PERSONALITY_OPTIONS.map(option => (
                  <option key={option.id} value={option.id}>
                    {option.label}
                  </option>
                ))}
              </select>
              <span className="settings__profile-help">
                {CAT_PERSONALITY_OPTIONS.find(option => option.id === selectedProfile.personality)?.description}
              </span>
            </label>
          </div>
        )}
        <div className="settings__toggle-row">
          <span className="settings__toggle-label">Companions</span>
          <div className="settings__stepper">
            <button className="settings__stepper-btn" onClick={() => handleCompanionsChange(maxCompanions - 1)}>-</button>
            <span className="settings__stepper-value">{maxCompanions}</span>
            <button className="settings__stepper-btn" onClick={() => handleCompanionsChange(maxCompanions + 1)}>+</button>
          </div>
        </div>
        {/* Inventory */}
        <div style={{ marginTop: 12 }}>
          <h2 className="settings__section-title">Inventory</h2>
          <div style={{
            display: "grid",
            gridTemplateColumns: "repeat(4, 1fr)",
            gap: 6,
          }}>
            {HAT_DEFINITIONS.map(hat => {
              const unlocked = unlockedHats.includes(hat.id);
              const equipped = currentHat === hat.id;
              return (
                <button
                  key={hat.id}
                  onClick={() => unlocked && handleHatToggle(hat.id)}
                  title={unlocked ? `${hat.name}${equipped ? " (equipped)" : ""}` : `\uD83D\uDD12 ${hat.condition}`}
                  style={{
                    width: "100%",
                    aspectRatio: "1",
                    borderRadius: 8,
                    border: equipped ? "2px solid #ffd700" : "2px solid transparent",
                    background: unlocked ? "#2a2a3a" : "#1a1a2e",
                    cursor: unlocked ? "pointer" : "default",
                    padding: 4,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    opacity: unlocked ? 1 : 0.4,
                    position: "relative",
                  }}
                >
                  {unlocked ? (
                    <img
                      src={hat.image}
                      alt={hat.name}
                      style={{ width: "80%", height: "80%", objectFit: "contain", imageRendering: "pixelated" }}
                    />
                  ) : (
                    <span style={{ fontSize: 16 }}>{"\uD83D\uDD12"}</span>
                  )}
                </button>
              );
            })}
          </div>
          {/* Item Debug Mode */}
          <button
            onClick={() => {
              const next = !itemDebugMode;
              setItemDebugMode(next);
              setDebugLog("");
              emit("item:debug", next);
            }}
            style={{
              marginTop: 8,
              padding: "6px 10px",
              fontSize: 11,
              borderRadius: 6,
              border: itemDebugMode ? "1px solid #ffd700" : "1px solid #555",
              background: itemDebugMode ? "#3a3a1a" : "#1a1a2e",
              color: itemDebugMode ? "#ffd700" : "#888",
              cursor: "pointer",
              width: "100%",
            }}
          >
            {itemDebugMode ? "🔴 Debug ON — ↑↓←→: 이동 (Shift=5px) / Enter: 저장 / R: 리셋" : "Item Debug Mode"}
          </button>
          {itemDebugMode && (
            <div style={{ marginTop: 6, display: "flex", alignItems: "center", gap: 6, flexWrap: "wrap" }}>
              <button
                onClick={async () => {
                  await invoke("unlock_all_hats");
                  const info = await invoke<{ currentHat: string | null; unlockedHats: string[] }>("get_hat_info");
                  setUnlockedHats(info.unlockedHats);
                }}
                style={{
                  fontSize: 10, padding: "3px 8px", borderRadius: 4,
                  border: "1px solid #f55", background: "#2a1a1a", color: "#f88", cursor: "pointer",
                }}
              >
                Unlock All
              </button>
              <span style={{ fontSize: 11, color: "#aaa" }}>강제 상태:</span>
              <select
                onChange={(e) => emit("item:debug:forceState", e.target.value)}
                style={{
                  fontSize: 11, background: "#1a1a2e", color: "#fff",
                  border: "1px solid #555", borderRadius: 4, padding: "2px 4px",
                }}
              >
                <option value="">자동</option>
                <option value="stand">stand</option>
                <option value="walk">walk</option>
                <option value="sit">sit</option>
                <option value="sleep">sleep</option>
                <option value="grab">grab</option>
                <option value="petting">petting</option>
              </select>
            </div>
          )}
          {itemDebugMode && (
            <div style={{ marginTop: 6, fontSize: 10, fontFamily: "monospace" }}>
              {debugLog && (
                <div style={{ color: "#4f4", marginBottom: 4 }}>✅ {debugLog}</div>
              )}
              {Object.keys(allSavedAnchors).length > 0 && (
                <div style={{ background: "#1a1a2e", borderRadius: 4, padding: 6, border: "1px solid #333" }}>
                  <div style={{ color: "#aaa", marginBottom: 3 }}>확정된 delta (Enter로 코드 반영 요청):</div>
                  {["brown", "orange", "white"].map(color => {
                    const entries = Object.entries(allSavedAnchors).filter(([k]) => k.startsWith(color + "/"));
                    if (entries.length === 0) return null;
                    return (
                      <div key={color} style={{ marginBottom: 2 }}>
                        <span style={{ color: color === "brown" ? "#c87" : color === "orange" ? "#fa5" : "#ccc" }}>{color}</span>
                        {": "}
                        {entries.map(([k, v]) => (
                          <span key={k} style={{ color: "#8f8", marginRight: 6 }}>
                            {k.split("/")[1]}(dy:{v.dy},dx:{v.dx})
                          </span>
                        ))}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          )}
        </div>
      </section>

      {/* Birthday */}
      <section className="settings__section">
        <h2 className="settings__section-title">Birthday</h2>
        <p style={{ fontSize: 11, color: "#888", marginBottom: 8 }}>
          Set your birthday for a special surprise!
        </p>
        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
          <select
            value={birthdayMonth ?? ""}
            onChange={(e) => {
              const m = e.target.value ? Number(e.target.value) : null;
              setBirthdayMonth(m);
              persistSettingsPatch({ birthdayMonth: m, birthdayDay: birthdayDay });
            }}
            style={{
              flex: 1, padding: "6px 8px", fontSize: 13,
              background: "#f5f5f5", color: "#333", border: "1px solid #ddd",
              borderRadius: 6,
            }}
          >
            <option value="">Month</option>
            {["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"].map((name, i) => (
              <option key={i + 1} value={i + 1}>{name}</option>
            ))}
          </select>
          <select
            value={birthdayDay ?? ""}
            onChange={(e) => {
              const d = e.target.value ? Number(e.target.value) : null;
              setBirthdayDay(d);
              persistSettingsPatch({ birthdayMonth: birthdayMonth, birthdayDay: d });
            }}
            style={{
              flex: 1, padding: "6px 8px", fontSize: 13,
              background: "#f5f5f5", color: "#333", border: "1px solid #ddd",
              borderRadius: 6,
            }}
          >
            <option value="">Day</option>
            {Array.from({ length: birthdayMonth === 2 ? 29 : [4,6,9,11].includes(birthdayMonth ?? 0) ? 30 : 31 }, (_, i) => (
              <option key={i + 1} value={i + 1}>{i + 1}</option>
            ))}
          </select>
        </div>
      </section>

      {/* Timer */}
      <section className="settings__section">
        <h2 className="settings__section-title">Timer</h2>
        <div className="settings__timer-row">
          <label className="settings__timer-label">Focus Duration</label>
          <div className="settings__stepper">
            <button className="settings__stepper-btn" onClick={() => handleFocusChange(focusMinutes - 5)}>-</button>
            <span className="settings__stepper-value">{focusMinutes} min</span>
            <button className="settings__stepper-btn" onClick={() => handleFocusChange(focusMinutes + 5)}>+</button>
          </div>
        </div>
        <div className="settings__timer-row">
          <label className="settings__timer-label">Break Duration</label>
          <div className="settings__stepper">
            <button className="settings__stepper-btn" onClick={() => handleBreakChange(breakMinutes - 1)}>-</button>
            <span className="settings__stepper-value">{breakMinutes} min</span>
            <button className="settings__stepper-btn" onClick={() => handleBreakChange(breakMinutes + 1)}>+</button>
          </div>
        </div>
      </section>

      {/* AI */}
      <section className="settings__section">
        <h2 className="settings__section-title">AI</h2>

        <div className="settings__provider-row">
          <div className="settings__provider-field">
            <label className="settings__ai-key-label" htmlFor="ai-provider-select">Provider</label>
            <div className="settings__provider-select-wrap">
              <select
                id="ai-provider-select"
                className="settings__provider-select"
                value={aiProvider}
                onChange={e => handleProviderChange(e.target.value as AIProvider)}
              >
                {aiProviderCatalog.map(provider => (
                  <option key={provider.id} value={provider.id}>
                    {provider.label}
                  </option>
                ))}
              </select>
              <span className="settings__provider-caret">▾</span>
            </div>
          </div>
          {selectedProvider && (
            <div className="settings__provider-note">
              {selectedProvider.description}
            </div>
          )}
          <div className="settings__provider-field">
            <label className="settings__ai-key-label" htmlFor="ai-model-select">Model</label>
            <div className="settings__provider-select-wrap">
              <select
                id="ai-model-select"
                className="settings__provider-select"
                value={selectedModel}
                onChange={e => handleModelChange(e.target.value)}
                disabled={!selectedProvider}
              >
                {selectedProvider?.models.map(model => (
                  <option key={model.id} value={model.id}>
                    {model.label}
                  </option>
                ))}
              </select>
              <span className="settings__provider-caret">▾</span>
            </div>
          </div>
          {(selectedModelEntry?.reasoningEfforts?.length ?? 0) > 0 && (
            <div className="settings__provider-field">
              <label className="settings__ai-key-label" htmlFor="ai-reasoning-select">Reasoning</label>
              <div className="settings__provider-select-wrap">
                <select
                  id="ai-reasoning-select"
                  className="settings__provider-select"
                  value={selectedReasoning}
                  onChange={e => handleReasoningChange(e.target.value)}
                >
                  {selectedModelEntry?.defaultReasoning && (
                    <option value="">
                      {`(default) ${selectedModelEntry.defaultReasoning}`}
                    </option>
                  )}
                  {selectedModelEntry?.reasoningEfforts
                    ?.filter(option => option.id !== selectedModelEntry.defaultReasoning)
                    .map(option => (
                    <option key={option.id} value={option.id}>
                      {option.label}
                    </option>
                  ))}
                </select>
                <span className="settings__provider-caret">▾</span>
              </div>
            </div>
          )}
        </div>
        {renderProviderConfiguration()}
      </section>

      {/* Notifications */}
      <section className="settings__section">
        <h2 className="settings__section-title">Notifications</h2>
        <div className="settings__toggle-row">
          <span className="settings__toggle-label">System Notifications</span>
          <button
            className={`settings__toggle ${notificationsEnabled ? "settings__toggle--on" : ""}`}
            onClick={handleNotificationsToggle}
          >
            <span className="settings__toggle-knob" />
          </button>
        </div>
      </section>

      {/* GitHub */}
      <section className="settings__section">
        <h2 className="settings__section-title">GitHub</h2>
        {githubUsername ? (
          <div className="settings__github-status">
            <span>Connected as <strong>@{githubUsername}</strong></span>
            <button
              className="settings__github-btn settings__github-btn--disconnect"
              onClick={handleGithubDisconnect}
            >
              Disconnect
            </button>
          </div>
        ) : (
          <>
            <div className="settings__github-row">
              <input
                className="settings__github-input"
                type="password"
                placeholder="GitHub Personal Access Token"
                value={githubToken}
                onChange={e => setGithubToken(e.target.value)}
                disabled={githubConnecting}
                onKeyDown={e => e.key === "Enter" && handleGithubConnect()}
              />
              <button
                className="settings__github-btn settings__github-btn--connect"
                onClick={handleGithubConnect}
                disabled={githubConnecting || !githubToken.trim()}
              >
                {githubConnecting ? "..." : "Connect"}
              </button>
            </div>
            {githubError && (
              <div className="settings__github-error">{githubError}</div>
            )}
          </>
        )}
      </section>

      {/* Clone Modal */}
      {cloneModalOpen && (
        <div className="settings__modal-overlay" onClick={() => !cloning && setCloneModalOpen(false)}>
          <div className="settings__modal" onClick={e => e.stopPropagation()}>
            <h3 className="settings__modal-title">Clone Repository</h3>

            <label className="settings__modal-label">Repository URL</label>
            <input
              className="settings__modal-input"
              type="text"
              placeholder="https://github.com/user/repo.git"
              value={cloneUrl}
              onChange={e => setCloneUrl(e.target.value)}
              disabled={cloning}
            />

            <label className="settings__modal-label">Local Path</label>
            <div className="settings__modal-path-row">
              <input
                className="settings__modal-input"
                type="text"
                placeholder="/Users/.../projects"
                value={clonePath}
                onChange={e => setClonePath(e.target.value)}
                disabled={cloning}
              />
              <button
                className="settings__modal-choose"
                onClick={handleChooseClonePath}
                disabled={cloning}
              >
                Choose
              </button>
            </div>

            {cloneError && (
              <div className="settings__modal-error">{cloneError}</div>
            )}

            <div className="settings__modal-actions">
              <button
                className="settings__modal-btn settings__modal-btn--cancel"
                onClick={() => setCloneModalOpen(false)}
                disabled={cloning}
              >
                Cancel
              </button>
              <button
                className="settings__modal-btn settings__modal-btn--primary"
                onClick={handleClone}
                disabled={cloning || !cloneUrl.trim() || !clonePath.trim()}
              >
                {cloning ? "Cloning..." : "Clone"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
