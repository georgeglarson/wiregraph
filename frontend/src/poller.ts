import { TopologyResponse, PacketEvent, Stats } from "./types";

const BASE_URL = "http://127.0.0.1:9877";

export class Poller {
  private lastEventTs = 0;
  private aborted = false;

  onTopology?: (data: TopologyResponse) => void;
  onEvents?: (events: PacketEvent[]) => void;
  onStats?: (stats: Stats) => void;

  start(): void {
    this.pollTopology();
    this.pollEvents();
    this.pollStats();
  }

  stop(): void {
    this.aborted = true;
  }

  private async pollTopology(): Promise<void> {
    while (!this.aborted) {
      try {
        const res = await fetch(`${BASE_URL}/api/topology`);
        if (res.ok) {
          const data: TopologyResponse = await res.json();
          this.onTopology?.(data);
        }
      } catch {
        // Backend not ready yet
      }
      await sleep(1000);
    }
  }

  private async pollEvents(): Promise<void> {
    while (!this.aborted) {
      try {
        const res = await fetch(`${BASE_URL}/api/events?since=${this.lastEventTs}`);
        if (res.ok) {
          const events: PacketEvent[] = await res.json();
          if (events.length > 0) {
            this.lastEventTs = events[events.length - 1].timestamp;
            this.onEvents?.(events);
          }
        }
      } catch {
        // Backend not ready yet
      }
      await sleep(200);
    }
  }

  private async pollStats(): Promise<void> {
    while (!this.aborted) {
      try {
        const res = await fetch(`${BASE_URL}/api/stats`);
        if (res.ok) {
          const stats: Stats = await res.json();
          this.onStats?.(stats);
        }
      } catch {
        // Backend not ready yet
      }
      await sleep(1000);
    }
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
