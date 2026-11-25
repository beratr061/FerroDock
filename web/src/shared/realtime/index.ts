import { useSyncExternalStore } from "react";

import { isRealtimeConnected, onConnectionChange } from "./connection";

export {
  isRealtimeConnected,
  onSignal,
  startRealtime,
  waitForSignal,
  type Signal,
} from "./connection";

export function useRealtimeConnected(): boolean {
  return useSyncExternalStore(onConnectionChange, isRealtimeConnected, () => false);
}
