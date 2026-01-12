// Military assignments polling logic
import * as state from "../state.js";
import * as service from "../service.js";
import * as pollingControls from "../pollingControls.js";
import * as expandableSection from "../expandable-section.js";
import * as ui from "./military-assignments-ui.js";

let poller = null;

export function startPolling() {
  if (poller) {
    return; // Already registered
  }

  console.log("Registering military assignments polling...");

  poller = pollingControls.registerPoller("military-assignments", async () => {
    const data = await service.fetchMilitaryAssignments();
    ui.update(data.military_assignments);
  });

  poller.start();
}

export function stopPolling() {
  if (poller) {
    poller.stop();
    console.log("Stopped military assignments polling");
  }
}

export function init() {
  expandableSection.registerSection("military-assignments", {
    onExpand: () => {
      console.log("Military assignments expanded - starting polling");
      startPolling();
    },
    onCollapse: () => {
      console.log("Military assignments collapsed - stopping polling");
      stopPolling();
    },
    defaultExpanded: false,
  });
}
