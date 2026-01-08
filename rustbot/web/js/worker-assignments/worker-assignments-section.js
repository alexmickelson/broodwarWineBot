// Worker assignments expandable section setup
import * as expandableSection from "../expandable-section.js";
import * as workerAssignmentsPolling from "./worker-assignments-polling.js";

export function init() {
  expandableSection.registerSection("worker-assignments", {
    onExpand: () => {
      console.log("Worker assignments expanded - starting polling");
      workerAssignmentsPolling.startPolling();
    },
    onCollapse: () => {
      console.log("Worker assignments collapsed - stopping polling");
      workerAssignmentsPolling.stopPolling();
    },
    defaultExpanded: true,
  });
}
