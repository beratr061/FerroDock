import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";

import { onSignal, startRealtime, useRealtimeConnected } from "@/shared/realtime";

export function RealtimeBridge() {
  const queryClient = useQueryClient();
  const connected = useRealtimeConnected();

  useEffect(() => {
    void queryClient.invalidateQueries({ queryKey: ["deployments"] });
    void queryClient.invalidateQueries({ queryKey: ["docker", "containers"] });
  }, [connected, queryClient]);

  useEffect(() => {
    const stop = startRealtime();
    const unsubscribe = onSignal((signal) => {
      switch (signal.type) {
        case "deployment_status":
          void queryClient.invalidateQueries({ queryKey: ["deployments", signal.application_id] });
          break;
        case "containers_changed":
          void queryClient.invalidateQueries({ queryKey: ["docker", "containers"] });
          void queryClient.invalidateQueries({ queryKey: ["application-containers"] });
          break;
        case "resync":
          void queryClient.invalidateQueries({ queryKey: ["deployments"] });
          void queryClient.invalidateQueries({ queryKey: ["docker", "containers"] });
          break;
        case "deployment_log":
          break;
      }
    });
    return () => {
      unsubscribe();
      stop();
    };
  }, [queryClient]);

  return null;
}
