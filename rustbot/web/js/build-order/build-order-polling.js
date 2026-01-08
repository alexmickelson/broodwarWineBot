// Build order polling logic
import * as service from "../service.js";
import * as pollingControls from "../pollingControls.js";
import * as expandableSection from "../expandable-section.js";
import { update } from "./build-order-ui.js";

let poller = null;

export function startPolling() {
  if (poller) {
    return; // Already registered
  }

  console.log("Registering build order polling...");

  poller = pollingControls.registerPoller("build-order", async () => {
    const data = await service.fetchBuildOrder();
    update(data.build_order, data.build_order_index);
  });

  poller.start();
}

export function stopPolling() {
  if (poller) {
    poller.stop();
    console.log("Stopped build order polling");
  }
}

export function init() {
  expandableSection.registerSection("build-order", {
    onExpand: () => {
      console.log("Build order expanded - starting polling");
      startPolling();
    },
    onCollapse: () => {
      console.log("Build order collapsed - stopping polling");
      stopPolling();
    },
    defaultExpanded: false,
  });
}
