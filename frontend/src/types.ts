export interface Tab {
  id: string;
  title: string;
  sessionId: string;
  type: "local" | "ssh";
}
