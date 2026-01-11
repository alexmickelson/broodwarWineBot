// Map polling logic
import * as service from "../service.js";
import * as pollingControls from "../pollingControls.js";
import * as expandableSection from "../expandable-section.js";
import { update } from "./map-ui.js";

let poller = null;

export function startPolling() {
  if (poller) {
    return; // Already registered
  }

  console.log("Registering map polling...");

  poller = pollingControls.registerPoller("map", async () => {
    const data = await service.fetchMap();
    update(data.map_data);
  });

  poller.start();
}

export function stopPolling() {
  if (poller) {
    poller.stop();
    console.log("Stopped map polling");
  }
}

export function init() {
  expandableSection.registerSection("map-visualization", {
    onExpand: () => {
      console.log("Map expanded - starting polling");
      startPolling();
    },
    onCollapse: () => {
      console.log("Map collapsed - stopping polling");
      stopPolling();
    },
    defaultExpanded: false,
  });
}
