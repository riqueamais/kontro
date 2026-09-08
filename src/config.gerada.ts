export type CloseAction = "MinimizeToTray" | "Exit";
export type OverlayMode = "Desligada" | "EmJogo" | "Sempre";
export type Theme = "Sistema" | "Noite" | "Preto" | "Ardosia" | "Brasa" | "Dia";

export interface Config {
  StartWithWindows: boolean;
  StartMinimized: boolean;
  CloseAction: CloseAction;
  NotificationsEnabled: boolean;
  WarnThreshold: number;
  CriticalThreshold: number;
  ConnectToastEnabled: boolean;
  OverlayMode: OverlayMode;
  OverlayX: number;
  OverlayY: number;
  OverlayMonitor: number;
  OverlayScale: number;
  OverlayOpacity: number;
  AutoCheckUpdates: boolean;
  OverlayShortcutEnabled: boolean;
  OverlayShortcut: string;
  OverlayMoveShortcut: string;
  FirstRunDone: boolean;
  Theme: Theme;
}
