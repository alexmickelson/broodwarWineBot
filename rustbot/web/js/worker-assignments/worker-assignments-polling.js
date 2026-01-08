// Worker assignments polling logic
import * as service from "../service.js";
import * as pollingControls from "../pollingControls.js";
import { update } from "./worker-assignments-ui.js";

let poller = null;

export function startPolling() {
  if (poller) {
    return; // Already registered
  }

  console.log("Registering worker assignments polling...");

  poller = pollingControls.registerPoller("worker-assignments", async () => {
    const data = await service.fetchWorkerStatus();
    update(data.worker_assignments);
  });

  poller.start();
}

export function stopPolling() {
  if (poller) {
    poller.stop();
    console.log("Stopped worker assignments polling");
  }
}
