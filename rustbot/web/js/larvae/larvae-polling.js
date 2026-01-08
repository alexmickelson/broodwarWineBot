// Larvae assignments polling logic
import * as service from "../service.js";
import * as pollingControls from "../pollingControls.js";
import * as expandableSection from "../expandable-section.js";
import { update } from "./larvae-ui.js";

let poller = null;

export function startPolling() {
  if (poller) {
    return; // Already registered
  }

  console.log("Registering larvae assignments polling...");

  poller = pollingControls.registerPoller("larvae-assignments", async () => {
    const data = await service.fetchLarvae();
    update(data.larva_responsibilities);
  });

  poller.start();
}

export function stopPolling() {
  if (poller) {
    poller.stop();
    console.log("Stopped larvae assignments polling");
  }
}

export function init() {
  expandableSection.registerSection("larvae-assignments", {
    onExpand: () => {
      console.log("Larvae assignments expanded - starting polling");
      startPolling();
    },
    onCollapse: () => {
      console.log("Larvae assignments collapsed - stopping polling");
      stopPolling();
    },
    defaultExpanded: true,
  });
}
