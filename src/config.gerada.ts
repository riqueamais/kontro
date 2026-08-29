export type CloseAction = "MinimizeToTray" | "Exit";
export type OverlayMode = "Desligada" | "EmJogo" | "Sempre";
export type OverlayCorner = "SuperiorEsquerdo" | "SuperiorDireito" | "InferiorEsquerdo" | "InferiorDireito";

export interface Config {
  StartWithWindows: boolean;
  StartMinimized: boolean;
  CloseAction: CloseAction;
  NotificationsEnabled: boolean;
  WarnThreshold: number;
  CriticalThreshold: number;
  ConnectToastEnabled: boolean;
  OverlayMode: OverlayMode;
  OverlayCorner: OverlayCorner;
  OverlayMonitor: number;
  OverlayScale: number;
  OverlayOpacity: number;
  AutoCheckUpdates: boolean;
  OverlayShortcutEnabled: boolean;
  FirstRunDone: boolean;
}
