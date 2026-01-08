// Unit orders polling logic
import * as service from "../service.js";
import * as pollingControls from "../pollingControls.js";
import * as expandableSection from "../expandable-section.js";
import { update } from "./unit-orders-ui.js";

let poller = null;

export function startPolling() {
  if (poller) {
    return; // Already registered
  }

  console.log("Registering unit orders polling...");

  poller = pollingControls.registerPoller("unit-orders", async () => {
    const data = await service.fetchUnitOrders();
    update(data.unit_orders);
  });

  poller.start();
}

export function stopPolling() {
  if (poller) {
    poller.stop();
    console.log("Stopped unit orders polling");
  }
}

export function init() {
  expandableSection.registerSection("unit-orders", {
    onExpand: () => {
      console.log("Unit orders expanded - starting polling");
      startPolling();
    },
    onCollapse: () => {
      console.log("Unit orders collapsed - stopping polling");
      stopPolling();
    },
    defaultExpanded: false,
  });
}
