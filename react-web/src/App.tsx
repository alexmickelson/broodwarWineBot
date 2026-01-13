import { GameSpeed } from './GameSpeed';
import { WorkerAssignments } from './WorkerAssignments';
import { MilitaryAssignments } from './MilitaryAssignments';
import { LarvaeAssignments } from './LarvaeAssignments';
import { UnitOrders } from './UnitOrders';
import { BuildOrder } from './BuildOrder';
import { MapVisualization } from './MapVisualization';

function App() {
  return (
    <div className="min-h-screen bg-bg-primary text-text-primary">
      <div className="max-w-7xl mx-auto h-screen flex flex-col">
        <GameSpeed />
        
        <div className="flex-1 overflow-y-auto p-5">
          <h1 className="text-center text-text-accent text-4xl mb-8 shadow-text">
            RustBot Status
          </h1>
          
          <WorkerAssignments />
          <MilitaryAssignments />
          <LarvaeAssignments />
          <UnitOrders />
          <BuildOrder />
          <MapVisualization />
        </div>
      </div>
    </div>
  );
}

export default App;
