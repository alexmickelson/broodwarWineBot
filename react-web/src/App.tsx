import { GameSpeed } from "./gameSpeed/GameSpeed";
import { WorkerAssignments } from "./workerAssignments/WorkerAssignments";
import { MilitaryAssignments } from "./militaryAssignments/MilitaryAssignments";
import { LarvaeAssignments } from "./larvaeAssignments/LarvaeAssignments";
import { UnitOrders } from "./unitOrders/UnitOrders";
import { BuildOrder } from "./buildOrder/BuildOrder";
import { MapVisualization } from "./mapVisualization/MapVisualization";
import { DebugFlags } from "./debugFlags/DebugFlags";

const App = () => {
  return (
    <div className="min-h-screen bg-abyss-100 text-lavender-400">
      <div className="max-w-7xl mx-auto h-screen flex flex-col">
        <GameSpeed />

        <div className="flex-1 overflow-y-auto p-5">
          <WorkerAssignments />
          <MilitaryAssignments />
          <LarvaeAssignments />
          <UnitOrders />
          <BuildOrder />
          <DebugFlags />
          <MapVisualization />
        </div>
      </div>
    </div>
  );
};

export default App;
