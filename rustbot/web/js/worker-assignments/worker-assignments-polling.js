// Worker assignments polling logic
import * as service from "../service.js";
import * as pollingControls from "../pollingControls.js";
import * as expandableSection from "../expandable-section.js";
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

export function init() {
  expandableSection.registerSection("worker-assignments", {
    onExpand: () => {
      console.log("Worker assignments expanded - starting polling");
      startPolling();
    },
    onCollapse: () => {
      console.log("Worker assignments collapsed - stopping polling");
      stopPolling();
    },
    defaultExpanded: false,
  });
}
