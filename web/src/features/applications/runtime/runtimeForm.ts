import type { MountKind, Protocol, RuntimeConfig } from "./api";

export interface MountRow {
  kind: MountKind;
  source: string;
  target: string;
  readOnly: boolean;
}

export interface PortRow {
  published: string;
  target: string;
  protocol: Protocol;
}

export interface RuntimeForm {
  mounts: MountRow[];
  ports: PortRow[];
}

export const NEW_MOUNT: MountRow = { kind: "volume", source: "", target: "", readOnly: false };
export const NEW_PORT: PortRow = { published: "", target: "", protocol: "tcp" };
export const PORT_PATTERN = /^\d{1,5}$/;

export function toForm(config: RuntimeConfig): RuntimeForm {
  return {
    mounts: config.mounts.map((mount) => ({
      kind: mount.kind,
      source: mount.source,
      target: mount.target,
      readOnly: mount.read_only ?? false,
    })),
    ports: config.ports.map((port) => ({
      published: String(port.published),
      target: String(port.target),
      protocol: port.protocol,
    })),
  };
}

export function toRequest(form: RuntimeForm): RuntimeConfig {
  return {
    mounts: form.mounts.map((mount) => ({
      kind: mount.kind,
      source: mount.source.trim(),
      target: mount.target.trim(),
      read_only: mount.readOnly,
    })),
    ports: form.ports.map((port) => ({
      published: Number(port.published),
      target: Number(port.target),
      protocol: port.protocol,
    })),
  };
}

export function validPort(value: string): boolean {
  const port = Number(value);
  return PORT_PATTERN.test(value) && port >= 1 && port <= 65535;
}
